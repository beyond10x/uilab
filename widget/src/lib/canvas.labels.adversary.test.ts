import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary pass 1, story:canvas-shows-labels. The canvas shows every text ess-ui/1 0.48.0 lets an
// author write as visible text, in preview and structure mode. These cases are the texts the
// unit's own `metric.label.test.ts` does not reach: a page header's headline `metrics` (their
// `label` is a metric's caption, the story's own subject), a section's `states.empty` message and
// action, and the `label` / `placeholder` of the `toggle` and `input` primitives. Each node is
// built as the server sends it (`crates/uilab-doc/src/outline.rs`: a page's `header` as ESS renders
// it, a composite's written keys other than `NOT_PROPS` as its props, a primitive's as its props).

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
    openOverlay: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=LabelsAdversary' },
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
  const script = compileScript(descriptor, { id: 'labels-adversary', inlineTemplate: true });
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

function n(path: string, layer: string, kind: string, extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children: [], ...extra };
}

const P = 'page:overview';

/** The library overview with a header whose headline metric is labelled (as ESS renders it: the
 *  server's `header_json` drops `metrics` and puts back the page's as written), a collection whose
 *  read returns no rows and whose `states.empty` the author wrote, and a section carrying a toggle
 *  and an input primitive among its `children`. */
function doc(): OutlineNode {
  const sections = [
    n(`${P}/section:recent`, 'section', 'collection', {
      view: 'loans.None',
      props: {
        columns: [{ field: 'title' }],
        states: { empty: { message: 'Nothing is out on loan', action: { name: 'lend', opens: 'lend', label: 'Lend a copy' } } },
      },
    }),
    n(`${P}/section:prefs`, 'section', 'record', {
      view: 'loans.Summary',
      props: { fields: ['on_loan'] },
      children: [
        n(`${P}/section:prefs/child:notify`, 'child', 'toggle', { props: { label: 'Email me when a copy is due', binds: 'draft.notify' } }),
        n(`${P}/section:prefs/child:find`, 'child', 'input', { props: { as: 'search', placeholder: 'Search the shelves', binds: 'state.q' } }),
      ],
    }),
  ];
  return n('/', 'root', 'document', {
    title: 'Lending library',
    children: [
      n('nav', 'nav', 'navigation', {
        props: { home: 'overview' },
        children: [n('nav/nav_section:circulation', 'nav_section', 'nav_section', { title: 'Circulation', props: { pages: ['overview'] } })],
      }),
      n(P, 'page', 'dashboard_page', {
        title: 'Overview',
        props: {
          shell: 'app',
          header: {
            title: 'Overview',
            metrics: [{ name: 'out', component: 'metric', label: 'Copies out', reads: { view: 'loans.Summary' }, from: 'on_loan' }],
          },
        },
        children: sections,
      }),
    ],
  });
}

async function render(mode: CanvasMode): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = {
    'loans.Summary': { view: 'loans.Summary', rows: [{ on_loan: 12 }] } as Rows,
    'loans.None': { view: 'loans.None', rows: [] } as unknown as Rows,
  };
  state.doc = { outline: doc(), selected: '/' };
  state.currentPage = P;
  state.openOverlay = null;
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The HTML inside the element at `path`, up to the next element of a node not below it. */
function nodeHtml(html: string, path: string): string {
  const at = html.indexOf(`data-path="${path}"`);
  assert.ok(at >= 0, `nothing on the canvas has data-path ${path}`);
  const rest = html.slice(html.indexOf('>', at) + 1);
  const next = [...rest.matchAll(/data-path="([^"]+)"/g)].find((m) => m[1] !== path && !m[1].startsWith(`${path}/`));
  return next ? rest.slice(0, rest.lastIndexOf('<', next.index)) : rest;
}

function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
}

/** The page's own HTML before its first section: its header. */
function headerHtml(html: string): string {
  const page = nodeHtml(html, P);
  const first = page.indexOf(`data-path="${P}/section:`);
  return first >= 0 ? page.slice(0, page.lastIndexOf('<', first)) : page;
}

for (const mode of ['preview', 'structure'] as const) {
  test(`${mode}: a page header's headline metric shows its label`, async () => {
    const shown = text(headerHtml(await render(mode)));
    assert.ok(shown.includes('Copies out'), `the header metric's label is not on the canvas: ${shown}`);
  });

  test(`${mode}: a section whose read returns no rows shows the author's empty message and action`, async () => {
    const shown = text(nodeHtml(await render(mode), `${P}/section:recent`));
    assert.ok(shown.includes('Nothing is out on loan'), `the states.empty message is not on the canvas: ${shown}`);
    assert.ok(shown.includes('Lend a copy'), `the states.empty action label is not on the canvas: ${shown}`);
  });

  test(`${mode}: a toggle primitive shows its label`, async () => {
    const shown = text(nodeHtml(await render(mode), `${P}/section:prefs/child:notify`));
    assert.ok(shown.includes('Email me when a copy is due'), `the toggle's label is not on the canvas: ${shown}`);
  });

  test(`${mode}: an input primitive shows its placeholder`, async () => {
    const html = nodeHtml(await render(mode), `${P}/section:prefs/child:find`);
    assert.ok(html.includes('placeholder="Search the shelves"'), `no input hints "Search the shelves": ${text(html)}`);
  });
}
