import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// story:canvas-shows-labels, `metric.label.test.ts`. The canvas shows the text ESS lets an author
// write on a composite: a metric's `label` above its value, and every other visible text of
// ess-ui/1 0.48.0's composites (`label` on a field, a column, an action, a tab, a form group and
// the submit button; `title` on a page and an overlay; a choice's option labels; a confirm's body,
// consequences, input label and confirm label; a filter bar's search placeholder). Each node is
// built as the server sends it (`crates/uilab-doc/src/outline.rs`, `describe`: the written keys
// other than `NOT_PROPS` are the node's props, `title` is lifted), in preview and structure mode.

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
  location: { search: '?name=Labels' },
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
  const script = compileScript(descriptor, { id: 'labels', inlineTemplate: true });
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

/** The library overview (examples/library/library.ui.yaml) with one section or overlay per
 *  composite that carries author-written text. */
function doc(): OutlineNode {
  const sections = [
    n(`${P}/section:on_loan`, 'section', 'metric', { view: 'loans.Summary', props: { label: 'Copies on loan', from: 'on_loan' } }),
    n(`${P}/section:bare`, 'section', 'metric', { view: 'loans.Summary', props: { from: 'on_loan' } }),
    n(`${P}/section:recent`, 'section', 'collection', {
      view: 'loans.All',
      props: {
        columns: [{ field: 'title', label: 'Book' }, { field: 'due' }],
        row_actions: [{ opens: 'extend', label: 'Extend' }],
        bulk_actions: [{ does: 'loans.ReturnMany', label: 'Return selected' }],
        actions: [{ name: 'export', export: { reads: 'loans.All', as: 'csv' }, label: 'Export CSV' }],
      },
    }),
    n(`${P}/section:detail`, 'section', 'record', {
      view: 'loans.All',
      props: {
        fields: [{ field: 'member', label: 'Borrower' }],
        tabs: [{ name: 'more', label: 'More', fields: [{ field: 'due', label: 'Due back' }] }],
        actions: [{ does: 'loans.Renew', label: 'Renew now' }],
      },
    }),
    n(`${P}/section:lend`, 'section', 'form', {
      props: {
        does: 'loans.Lend',
        fields: [{ field: 'member', label: 'Lend to' }],
        groups: [{ name: 'notes', label: 'Notes for the desk', fields: [{ field: 'note', label: 'Remark' }] }],
        tabs: [{ name: 'when', label: 'When', fields: [{ field: 'due', label: 'Bring back by' }] }],
        actions: [{ does: 'loans.Cancel', label: 'Cancel lending' }],
        submit: { label: 'Lend the copy' },
      },
    }),
    n(`${P}/section:filters`, 'section', 'filter_bar', {
      props: {
        binds: ['state.q', 'state.min'],
        search: { binds: 'state.q', placeholder: 'Find a book' },
        inputs: [{ field: 'min', label: 'Minimum copies', binds: 'state.min' }],
        actions: [{ sets: { 'state.q': "''" }, label: 'Clear filters' }],
      },
    }),
    n(`${P}/section:pick`, 'section', 'choice', { props: { options: [{ value: 'open', label: 'Out on loan' }, 'returned'] } }),
    n(`${P}/section:used`, 'section', 'references', { view: 'loans.All', props: { columns: [{ field: 'title', label: 'Used by' }] } }),
    n(`${P}/section:tiles`, 'section', 'board', { view: 'loans.All', props: { item_actions: [{ does: 'loans.Unpin', label: 'Unpin tile' }] } }),
    n(`${P}/section:flow`, 'section', 'graph_editor', {
      view: 'loans.All',
      props: { node_actions: [{ opens: 'extend', label: 'Edit step' }], edge_actions: [{ does: 'flows.Insert', label: 'Insert step' }] },
    }),
  ];
  const overlays = [
    n(`${P}/overlay:extend`, 'overlay', 'drawer form', { title: 'Extend loan', props: { title: 'Extend loan', does: 'loans.ExtendLoan', fields: ['due'] } }),
    n(`${P}/overlay:return`, 'overlay', 'dialog confirm', {
      title: 'Return this copy?',
      props: {
        title: 'Return this copy?',
        does: 'loans.Return',
        body: 'The copy goes back on the shelf.',
        consequences: ['The member is notified'],
        confirm_label: 'Return it',
        input: { label: 'Type RETURN to confirm', must_equal: 'RETURN' },
        alternatives: [{ does: 'loans.Snooze', label: 'Remind me tomorrow' }],
      },
    }),
  ];
  return n('/', 'root', 'document', {
    title: 'Lending library',
    children: [n(P, 'page', 'dashboard_page', { title: 'Overview', props: { shell: 'app' }, children: [...sections, ...overlays] })],
  });
}

async function render(mode: CanvasMode, overlay: string | null = null): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = {
    'loans.Summary': { view: 'loans.Summary', rows: [{ on_loan: 12 }] } as Rows,
    'loans.All': { view: 'loans.All', rows: [{ title: 'Dune', member: 'Ada Lovelace', due: '2026-10-09' }] } as Rows,
  };
  state.doc = { outline: doc(), selected: '/' };
  state.currentPage = P;
  state.openOverlay = overlay;
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The HTML inside the element at `path` (from the end of its opening tag), up to the opening tag
 *  of the next element carrying a `data-path` of a node that is not below it. */
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

/** Each composite that carries author-written text, and the text the canvas must show for it:
 *  `shown` as visible text, `hint` as an input's placeholder. */
const CASES: { what: string; path: string; overlay?: boolean; shown: string[]; hint?: string[] }[] = [
  { what: 'a metric label', path: `${P}/section:on_loan`, shown: ['Copies on loan'] },
  { what: 'a column label', path: `${P}/section:recent`, shown: ['Book'] },
  { what: 'a row action label', path: `${P}/section:recent`, shown: ['Extend'] },
  { what: 'a collection action and bulk action label', path: `${P}/section:recent`, shown: ['Export CSV', 'Return selected'] },
  { what: 'a record field, tab and action label', path: `${P}/section:detail`, shown: ['Borrower', 'More', 'Due back', 'Renew now'] },
  {
    what: 'a form field, group, tab, action and submit label',
    path: `${P}/section:lend`,
    shown: ['Lend to', 'Notes for the desk', 'Remark', 'When', 'Bring back by', 'Cancel lending', 'Lend the copy'],
  },
  { what: 'a filter bar input label, search placeholder and action label', path: `${P}/section:filters`, shown: ['Clear filters'], hint: ['Minimum copies', 'Find a book'] },
  { what: 'a choice option label', path: `${P}/section:pick`, shown: ['Out on loan', 'returned'] },
  { what: 'a references column label', path: `${P}/section:used`, shown: ['Used by'] },
  { what: 'a board item action label', path: `${P}/section:tiles`, shown: ['Unpin tile'] },
  { what: 'a graph editor node and edge action label', path: `${P}/section:flow`, shown: ['Edit step', 'Insert step'] },
  { what: 'an overlay title', path: `${P}/overlay:extend`, overlay: true, shown: ['Extend loan'] },
  {
    what: 'a confirm title, body, consequence, input label, confirm label and alternative label',
    path: `${P}/overlay:return`,
    overlay: true,
    shown: ['Return this copy?', 'The copy goes back on the shelf.', 'The member is notified', 'Type RETURN to confirm', 'Return it', 'Remind me tomorrow'],
  },
];

for (const mode of ['preview', 'structure'] as const) {
  test(`${mode}: the library overview's metric shows its label above its value`, async () => {
    const html = nodeHtml(await render(mode), `${P}/section:on_loan`);
    const label = html.indexOf('Copies on loan');
    const value = html.indexOf('>12<');
    assert.ok(label >= 0, `the label is not on the canvas: ${text(html)}`);
    assert.ok(value >= 0, `the value is not on the canvas: ${text(html)}`);
    assert.ok(label < value, 'the label is drawn below the value');
  });

  test(`${mode}: a metric without a label shows its value and no invented caption`, async () => {
    const shown = text(nodeHtml(await render(mode), `${P}/section:bare`));
    assert.equal(shown, mode === 'preview' ? '12' : 'bare · metric · loans.Summary 12');
  });

  test(`${mode}: the page title is shown`, async () => {
    assert.match(text(nodeHtml(await render(mode), P)), /^(overview · dashboard_page )?Overview\b/);
  });

  for (const c of CASES) {
    test(`${mode}: ${c.what} is shown`, async () => {
      const html = nodeHtml(await render(mode, c.overlay ? c.path : null), c.path);
      for (const s of c.shown) assert.ok(text(html).includes(s), `${JSON.stringify(s)} is not on the canvas: ${text(html)}`);
      for (const s of c.hint ?? []) assert.ok(html.includes(`placeholder="${s}"`), `no input hints ${JSON.stringify(s)}`);
    });
  }

  test(`${mode}: a column without a label is headed by its field, nothing else`, async () => {
    const html = nodeHtml(await render(mode), `${P}/section:recent`);
    const heads = [...html.matchAll(/<th[^>]*>([^<]*)<\/th>/g)].map((m) => m[1]);
    assert.deepEqual(heads.slice(0, 2), ['Book', 'due']);
  });
}
