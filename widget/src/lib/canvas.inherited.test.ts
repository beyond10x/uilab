import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// story:essui-app-widget, `canvas.inherited.test.ts`. The canvas draws what ESS renders: the
// sections a page kind contributes and the body of a widget instance, each marked as inherited
// (epic:ess-ui-adoption D6). CanvasView, CompositeView and PrimitiveView are compiled with
// vue/compiler-sfc, as instance.canvas.test.ts does, and rendered to a string.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Inherited' },
  localStorage: { getItem: () => null, setItem: () => undefined },
};
const storeModule: string = '../store.ts';
const { state } = (await import(storeModule)) as StoreApi;
const vueUrl = import.meta.resolve('vue');
const rendererModule: string = 'vue/server-renderer';
const { renderToString } = (await import(rendererModule)) as { renderToString(app: unknown): Promise<string> };
const vueModule: string = vueUrl;
const { createSSRApp, h } = (await import(vueModule)) as {
  createSSRApp(root: unknown): unknown;
  h(component: unknown, props?: Record<string, unknown>): unknown;
};

function compileSfc(file: URL, vueImports: Record<string, string> = {}): string {
  const { descriptor } = parse(readFileSync(file, 'utf8'), { filename: file.pathname });
  const script = compileScript(descriptor, { id: 'inherited', inlineTemplate: true });
  const code = stripTypeScriptTypes(script.content, { mode: 'strip' }).replace(
    /from\s+(['"])([^'"]+)\1/g,
    (_m, _q: string, spec: string) => {
      if (spec === 'vue') return `from '${vueUrl}'`;
      if (spec.endsWith('.vue')) return `from '${vueImports[spec]}'`;
      return `from '${new URL(spec, file).href}'`;
    },
  );
  return `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`;
}

const primitiveUrl = compileSfc(new URL('../components/PrimitiveView.vue', import.meta.url));
const compositeUrl = compileSfc(new URL('../components/CompositeView.vue', import.meta.url), { './PrimitiveView.vue': primitiveUrl });
const canvasUrl = compileSfc(new URL('../components/CanvasView.vue', import.meta.url), { './CompositeView.vue': compositeUrl });
const CanvasView = ((await import(canvasUrl)) as { default: unknown }).default;

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

const LOANS = [{ title: 'Dune', member: 'Ada', due: '2026-10-03', state: 'open' }];

/**
 * The library as the server sends it: the Loans page (`list_page`) with the `filters` section its
 * kind contributes, inherited, before the `list` the author wrote; the overview with an instance of
 * `loan_card`, holding the widget's body nodes, inherited.
 */
function library(): OutlineNode {
  return n('/', 'root', 'document', [
    n(
      'component:loan_card',
      'component',
      'widget',
      [
        n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title', style: 'heading' } }),
        n('component:loan_card/node:due', 'node', 'badge', [], { props: { text: 'args.loan.due' } }),
      ],
      { title: 'A loan as a card.', props: { params: { loan: { type: 'Loan', required: true, note: 'the loan' } }, arrange: 'column', uses: [] } },
    ),
    n('page:overview', 'page', 'dashboard_page', [
      n(
        'page:overview/section:latest',
        'section',
        'loan_card',
        [
          n('page:overview/section:latest/node:title', 'node', 'text', [], { inherited: true, props: { text: 'args.loan.title', style: 'heading' } }),
          n('page:overview/section:latest/node:due', 'node', 'badge', [], { inherited: true, props: { text: 'args.loan.due' } }),
        ],
        { props: { args: { loan: 'rows.first' } }, view: 'loans.All' },
      ),
    ], { title: 'Overview' }),
    n('page:loans', 'page', 'list_page', [
      n('page:loans/section:filters', 'section', 'filter_bar', [], { inherited: true, props: { binds: ['state.search'], search: { binds: 'state.search' } } }),
      n('page:loans/section:list', 'section', 'collection', [], { view: 'loans.All', props: { columns: [{ field: 'title' }] } }),
    ], { title: 'Loans' }),
  ]);
}

async function render(page: string, mode: CanvasMode): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = { 'loans.All': { view: 'loans.All', rows: LOANS } as Rows };
  state.doc = { outline: library(), selected: '/' } as StoreApi['state']['doc'];
  state.currentPage = page;
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The class attribute of the element whose `data-path` is `path`, or `null` when none is drawn. */
function classOf(html: string, path: string): string | null {
  const escaped = path.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const tag = new RegExp(`<[a-z0-9]+\\s[^>]*data-path="${escaped}"[^>]*>`).exec(html)?.[0];
  if (!tag) return null;
  return /class="([^"]*)"/.exec(tag)?.[1] ?? '';
}

/** Every `data-path` drawn, in order. */
function drawn(html: string): string[] {
  return [...html.matchAll(/data-path="([^"]+)"/g)].map((m) => m[1]);
}

for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: the Loans page draws the filters section its kind contributes, inherited, before the list`, async () => {
    const html = await render('page:loans', mode);
    const sections = drawn(html).filter((p) => p.startsWith('page:loans/section:'));
    assert.deepEqual(sections, ['page:loans/section:filters', 'page:loans/section:list']);
    assert.ok(classOf(html, 'page:loans/section:filters')!.split(' ').includes('inherited'), 'filters carries the inherited class');
    assert.ok(!classOf(html, 'page:loans/section:list')!.split(' ').includes('inherited'), 'list is the author\'s');
  });

  test(`${mode}: a widget instance draws its body nodes at the instance, each inherited, with its args bound`, async () => {
    const html = await render('page:overview', mode);
    for (const body of ['page:overview/section:latest/node:title', 'page:overview/section:latest/node:due']) {
      const cls = classOf(html, body);
      assert.ok(cls !== null, `${body} is drawn`);
      assert.ok(cls.split(' ').includes('inherited'), `${body} carries the inherited class`);
    }
    assert.ok(!classOf(html, 'page:overview/section:latest')!.split(' ').includes('inherited'), 'the instance is the author\'s');
    assert.equal(drawn(html).filter((p) => p === 'page:overview/section:latest/node:title').length, 1, 'the body is drawn once');
    assert.ok(!drawn(html).some((p) => p.startsWith('component:')), 'the body is drawn at the instance, not at the declaration');
    assert.match(html, /Dune/);
    assert.doesNotMatch(html, /args\.loan/);
  });
}
