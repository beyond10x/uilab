import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary pass 2, story:canvas-shows-labels. After pass 1's fix the server sends every node as
// ESS renders it (crates/uilab-doc/tests/adversary_canvas_labels_p2.rs holds the props to ESS's
// loader), so what is left is what the canvas does with them. Each case is a text ess-ui/1 0.48.0
// lets the author write and the canvas does not draw, built as `outline::rendered` sends it for a
// document `ess ui check` (0.48.0) passes with 0 errors:
//
//   pages.overview (static_page): header {help: {link}, metrics: [{name: due, component: due_tag}]},
//   section body: collection with user-selectable columns {binds: state.cols, all: [Book, Due back]},
//   overlay member: form with `record` (read-only fields above the inputs: "Library card") and a
//   group whose action is "Send a check mail".

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
  location: { search: '?name=LabelsAdversaryP2' },
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
  const script = compileScript(descriptor, { id: 'labels-adversary-p2', inlineTemplate: true });
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
const OVERLAY = `${P}/overlay:member`;

function doc(): OutlineNode {
  return n('/', 'root', 'document', {
    title: 'Lending library',
    children: [
      n('component:due_tag', 'component', 'widget', {
        title: 'A due-back tag.',
        props: { params: {}, arrange: 'column', uses: [{ path: P, trail: 'header/metrics/due' }] },
        children: [n('component:due_tag/node:tag', 'node', 'badge', { props: { text: 'Due this week' } })],
      }),
      n('nav', 'nav', 'navigation', {
        props: { home: 'overview' },
        children: [n('nav/nav_section:desk', 'nav_section', 'nav_section', { title: 'Desk', props: { pages: ['overview'] } })],
      }),
      n(P, 'page', 'static_page', {
        title: 'Overview',
        props: {
          shell: 'app',
          header: { help: { link: 'https://library.example/help' }, metrics: [{ name: 'due', component: 'due_tag' }] },
        },
        children: [
          n(`${P}/section:body`, 'section', 'collection', {
            view: 'loans.All',
            props: { columns: { binds: 'state.cols', all: [{ field: 'title', label: 'Book' }, { field: 'due', label: 'Due back' }] } },
          }),
          n(OVERLAY, 'overlay', 'drawer form', {
            title: 'Member settings',
            props: {
              title: 'Member settings',
              does: 'members.Update',
              record: { component: 'record', reads: { view: 'members.Me' }, fields: [{ field: 'card_no', label: 'Library card' }] },
              groups: [
                {
                  name: 'contact',
                  label: 'Contact',
                  fields: [{ field: 'email', label: 'Email' }],
                  actions: [{ name: 'verify', does: 'members.VerifyEmail', label: 'Send a check mail' }],
                },
              ],
            },
          }),
        ],
      }),
    ],
  });
}

async function render(mode: CanvasMode, open: string | null = null): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = {
    'loans.All': { view: 'loans.All', rows: [{ title: 'Dune', due: '2026-10-08' }] } as unknown as Rows,
    'members.Me': { view: 'members.Me', rows: [{ card_no: 'L-0042' }] } as unknown as Rows,
  };
  state.doc = { outline: doc(), selected: '/' };
  state.currentPage = P;
  state.openOverlay = open;
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The HTML inside the element at `path`, up to the next element of a node not below it. */
function nodeHtml(html: string, path: string, from = 0): string {
  const at = html.indexOf(`data-path="${path}"`, from);
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

/** The open overlay's HTML: the modal, after the page. */
function modalHtml(html: string): string {
  const at = html.indexOf('class="modal"');
  assert.ok(at >= 0, 'no overlay is open on the canvas');
  return html.slice(at);
}

for (const mode of ['preview', 'structure'] as const) {
  test(`${mode}: a collection's user-selectable columns show their labels`, async () => {
    const shown = text(nodeHtml(await render(mode), `${P}/section:body`));
    assert.ok(shown.includes('Book') && shown.includes('Due back'), `the selectable columns' labels are not on the canvas: ${shown}`);
  });

  test(`${mode}: a form's read-only record shows its field labels`, async () => {
    const shown = text(modalHtml(await render(mode, OVERLAY)));
    assert.ok(shown.includes('Library card'), `the form's record field label is not on the canvas: ${shown}`);
  });

  test(`${mode}: a form group's action shows its label`, async () => {
    const shown = text(modalHtml(await render(mode, OVERLAY)));
    assert.ok(shown.includes('Contact'), `control: the group heading is drawn: ${shown}`);
    assert.ok(shown.includes('Send a check mail'), `the form group's action label is not on the canvas: ${shown}`);
  });

  test(`${mode}: a header metric that is a widget instance shows the widget's text`, async () => {
    const shown = text(headerHtml(await render(mode)));
    assert.ok(shown.includes('Due this week'), `the header metric's widget body is not on the canvas: ${shown}`);
  });

  test(`${mode}: a header help written as a link is on the canvas`, async () => {
    const html = headerHtml(await render(mode));
    assert.ok(html.includes('https://library.example/help'), `the header's help link is not on the canvas: ${text(html)}`);
  });
}
