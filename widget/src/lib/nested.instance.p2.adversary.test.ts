import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary pass 2 on story:essui-app-widget (d825897). The server now sends a widget instance
// holding its body, nested instances' bodies included, each at its own path and as written in its
// widget's declaration (`outline::rendered`). ESS renders a nested instance's body with the nested
// instance's own args (`instance_body`: the outer body is substituted first, then the inner widget
// is expanded with the inner args). Here both widgets name their param `loan`, as card and line
// widgets of one entity do:
//
//   loan_card(loan): title: args.loan.title; renewal: loan_line(loan: args.loan.renewal)
//   loan_line(loan): line: args.loan.title
//
// With `latest: loan_card(loan: rows.first)` over the row {title: Dune, renewal: {title: Dune
// Messiah}}, ESS renders `latest/body/renewal/body/line` as `text: rows.first.renewal.title`,
// i.e. "Dune Messiah". The canvas is compiled with vue/compiler-sfc and rendered to a string, as
// canvas.inherited.test.ts does.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=NestedP2' },
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
  const script = compileScript(descriptor, { id: 'nestedp2', inlineTemplate: true });
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

const LOAN = { type: 'Loan', required: true, note: 'the loan' };
const ROWS = [{ title: 'Dune', renewal: { title: 'Dune Messiah' } }];

/** The outline as `rendered` sends it: each body node's fields as its widget's declaration writes
 *  them, nested bodies included, inherited. */
function outline(): OutlineNode {
  const at = 'page:overview/section:latest';
  return n('/', 'root', 'document', [
    n(
      'component:loan_card',
      'component',
      'widget',
      [
        n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title' } }),
        n('component:loan_card/node:renewal', 'node', 'loan_line', [], { props: { args: { loan: 'args.loan.renewal' } } }),
      ],
      { title: 'A loan as a card.', props: { params: { loan: LOAN }, arrange: 'column', uses: [{ path: at }] } },
    ),
    n(
      'component:loan_line',
      'component',
      'widget',
      [n('component:loan_line/node:line', 'node', 'text', [], { props: { text: 'args.loan.title' } })],
      { title: 'A loan on one line.', props: { params: { loan: LOAN }, arrange: 'row', uses: [] } },
    ),
    n(
      'page:overview',
      'page',
      'dashboard_page',
      [
        n(
          at,
          'section',
          'loan_card',
          [
            n(`${at}/node:title`, 'node', 'text', [], { inherited: true, props: { text: 'args.loan.title' } }),
            n(
              `${at}/node:renewal`,
              'node',
              'loan_line',
              [n(`${at}/node:renewal/node:line`, 'node', 'text', [], { inherited: true, props: { text: 'args.loan.title' } })],
              { inherited: true, props: { args: { loan: 'args.loan.renewal' } } },
            ),
          ],
          { view: 'loans.All', props: { args: { loan: 'rows.first' } } },
        ),
      ],
      { title: 'Overview' },
    ),
  ]);
}

async function render(mode: CanvasMode): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = { 'loans.All': { view: 'loans.All', rows: ROWS } as Rows };
  state.doc = { outline: outline(), selected: '/' } as StoreApi['state']['doc'];
  state.currentPage = 'page:overview';
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The markup of the element whose `data-path` is `path`, up to the next `data-path`. */
function drawnAt(html: string, path: string): string | null {
  const start = html.indexOf(`data-path="${path}"`);
  if (start < 0) return null;
  const next = html.indexOf('data-path="', start + 1);
  return html.slice(start, next < 0 ? undefined : next);
}

for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: a nested instance's body reads its own args, as ESS renders it (p2)`, async () => {
    const html = await render(mode);
    const line = drawnAt(html, 'page:overview/section:latest/node:renewal/node:line');
    assert.ok(line !== null, 'the nested body node is drawn');
    assert.match(line, /Dune Messiah/, `ESS renders the renewal's title here; the canvas draws: ${line}`);
  });
}
