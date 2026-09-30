import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// story:canvas-widget-instances. A widget instance on a page draws its widget's body with its args
// bound, on the canvas itself: CanvasView, CompositeView and PrimitiveView are compiled with
// vue/compiler-sfc as canvasmode.adversary.test.ts does, and rendered to a string.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Instances' },
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
  const script = compileScript(descriptor, { id: 'instances', inlineTemplate: true });
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

const MEMBERS = [
  { id: 'm-1', name: 'Ada Lovelace', standing: 'good' },
  { id: 'm-2', name: 'Grace Hopper', standing: 'late' },
];
const LOANS = [{ id: 'l-1', title: 'Dune', state: 'open' }];

/** One page holding a `member_card` instance in a section, a `loan_card` instance in a board
 *  widget and a `member_card` instance as a collection item. */
function doc(): OutlineNode {
  return n('/', 'root', 'document', [
    n(
      'component:member_card',
      'component',
      'widget',
      [
        n('component:member_card/node:name', 'node', 'text', [], { props: { text: 'args.member.name', style: 'heading' } }),
        n('component:member_card/node:standing', 'node', 'badge', [], { props: { text: 'args.member.standing' } }),
      ],
      { title: 'A member', props: { params: { member: { type: 'Member', required: true } }, arrange: 'column', uses: [] } },
    ),
    n(
      'component:loan_card',
      'component',
      'widget',
      [n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title', style: 'heading' } })],
      { title: 'A loan', props: { params: { loan: { type: 'Loan', required: true } }, uses: [] } },
    ),
    n('page:members', 'page', 'list_page', [
      n('page:members/section:spotlight', 'section', 'member_card', [], { props: { args: { member: 'rows.1' } }, view: 'members.All' }),
      n(
        'page:members/section:kpis',
        'section',
        'board',
        [n('page:members/section:kpis/widget:top', 'widget', 'loan_card', [], { props: { args: { loan: 'rows.first' } } })],
        { view: 'loans.All' },
      ),
      n(
        'page:members/section:list',
        'section',
        'collection',
        [n('page:members/section:list/item:card', 'item', 'member_card', [], { props: { args: { member: 'row' } } })],
        { view: 'members.All' },
      ),
    ], { title: 'Members' }),
  ]);
}

async function renderCanvas(mode: CanvasMode, selected = '/'): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = {
    'members.All': { view: 'members.All', rows: MEMBERS } as Rows,
    'loans.All': { view: 'loans.All', rows: LOANS } as Rows,
  };
  state.doc = { outline: doc(), selected } as StoreApi['state']['doc'];
  state.currentPage = 'page:members';
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The HTML of the element at `path`, up to the next element carrying a `data-path` of a node that
 *  is not below it. */
function nodeHtml(html: string, path: string): string {
  const start = html.indexOf(`data-path="${path}"`);
  assert.ok(start >= 0, `nothing on the canvas has data-path ${path}`);
  const rest = html.slice(start);
  const next = [...rest.matchAll(/data-path="([^"]+)"/g)].find((m) => m.index > 0 && !m[1].startsWith(path) && !m[1].startsWith('component:'));
  return next ? rest.slice(0, next.index) : rest;
}

function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
}

for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: a section instance draws its widget body with rows.<n> bound, not a placeholder`, async () => {
    const html = nodeHtml(await renderCanvas(mode), 'page:members/section:spotlight');
    assert.doesNotMatch(html, /class="placeholder"/, 'the instance is still a hatched placeholder');
    assert.match(text(html), /Grace Hopper/);
    assert.match(text(html), /late/);
  });

  test(`${mode}: a board widget instance draws its body with rows.first of the board`, async () => {
    const html = nodeHtml(await renderCanvas(mode), 'page:members/section:kpis/widget:top');
    assert.doesNotMatch(html, /class="placeholder"/);
    assert.match(text(html), /Dune/);
  });

  test(`${mode}: a collection item instance draws once per row, each with its row`, async () => {
    const html = nodeHtml(await renderCanvas(mode), 'page:members/section:list');
    const items = [...html.matchAll(/data-path="page:members\/section:list\/item:card"/g)];
    assert.equal(items.length, MEMBERS.length, 'one instance per row');
    assert.match(text(html), /Ada Lovelace good .*Grace Hopper late/);
  });
}

test('structure labels the instance with its name and widget; preview does not', async () => {
  assert.match(text(nodeHtml(await renderCanvas('structure'), 'page:members/section:spotlight')), /spotlight · member_card/);
  assert.doesNotMatch(text(nodeHtml(await renderCanvas('preview'), 'page:members/section:spotlight')), /spotlight · member_card/);
});

test('the selected instance carries the selection; its body is drawn inside a body that takes no clicks', async () => {
  const html = await renderCanvas('structure', 'page:members/section:spotlight');
  assert.match(html, /class="card node selected"[^>]*data-path="page:members\/section:spotlight"/);
  const spotlight = nodeHtml(html, 'page:members/section:spotlight');
  assert.match(spotlight, /class="instance-body[^"]*"/, 'the body sits in the instance body');
});
