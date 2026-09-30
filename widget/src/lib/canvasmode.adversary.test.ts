import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { accountName, canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary cases for story:canvas-preview-mode. The preview is drawn by CanvasView and
// CompositeView, which no lib test reaches, so these render the real components server-side: each
// SFC is compiled with vue/compiler-sfc, its types stripped, its imports pointed at the same module
// files this test imports (one `vue`, one store, one canvas mode), and rendered to a string.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Adversary' },
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

/** An SFC compiled to a `data:` module whose imports resolve to the files this test uses. */
function compileSfc(file: URL, vueImports: Record<string, string> = {}): string {
  const { descriptor } = parse(readFileSync(file, 'utf8'), { filename: file.pathname });
  const script = compileScript(descriptor, { id: 'adversary', inlineTemplate: true });
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

const compositeUrl = compileSfc(new URL('../components/CompositeView.vue', import.meta.url));
const canvasUrl = compileSfc(new URL('../components/CanvasView.vue', import.meta.url), { './CompositeView.vue': compositeUrl });
const CompositeView = ((await import(compositeUrl)) as { default: unknown }).default;
const CanvasView = ((await import(canvasUrl)) as { default: unknown }).default;

function setMode(m: CanvasMode): void {
  canvasMode.mode.value = m;
}

function node(layer: string, name: string, kind: string, extra: Partial<OutlineNode> = {}, children: OutlineNode[] = []): OutlineNode {
  const path = extra.path ?? `${layer}:${name}`;
  return { layer, name, kind, path, children, ...extra };
}

/** Visible text of rendered HTML: tags and comments dropped, whitespace collapsed. */
function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
}

async function renderComposite(n: OutlineNode): Promise<string> {
  return renderToString(createSSRApp({ render: () => h(CompositeView, { node: n }) }));
}

async function renderCanvas(outline: OutlineNode, selected = '/'): Promise<string> {
  state.doc = { outline, selected } as StoreApi['state']['doc'];
  state.currentPage = null;
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** A document with one shell (account menu, notifications, overlay outlet) and one page. */
function app(accountView: string): OutlineNode {
  const shell = node('shell', 'main', 'shell', {}, [
    node('region', 'account', 'account_menu', { path: 'shell:main/account', view: accountView }),
    node('region', 'notify', 'notifications', { path: 'shell:main/notify' }),
    node('region', 'overlay', 'overlay_outlet', { path: 'shell:main/overlay' }),
    node('region', 'body', 'page_outlet', { path: 'shell:main/body' }),
  ]);
  const page = node('page', 'overview', 'page', { title: 'Overview' });
  return node('root', 'library', 'app', { path: '/', title: 'Lending library' }, [shell, page]);
}

test('preview hides the view a composite reads, including in its empty line', async () => {
  state.conn = 'closed';
  state.rows = {};
  const members = node('section', 'members', 'collection', { path: 'page:p/members', view: 'members.All', props: { columns: ['name'] } });
  setMode('structure');
  const structure = text(await renderComposite(members));
  assert.match(structure, /members · collection · members\.All/, 'structure keeps the label');
  setMode('preview');
  const preview = text(await renderComposite(members));
  assert.doesNotMatch(preview, /members · collection/, 'preview drops the label line');
  assert.doesNotMatch(preview, /members\.All/, `preview still names the view: ${JSON.stringify(preview)}`);
});

test('preview keeps a marked overlay outlet and its selection, and hides an unmarked one', async () => {
  state.rows = {};
  setMode('preview');
  const plain = await renderCanvas(app('staff.Me'));
  assert.doesNotMatch(text(plain), /overlay_outlet/, 'an unmarked outlet is invisible in preview');
  const selected = await renderCanvas(app('staff.Me'), 'shell:main/overlay');
  assert.match(selected, /class="chip node selected"[^>]*>\s*overlay\s*<span class="muted">overlay_outlet<\/span>/);
});

test('preview draws the account menu as the name from its rows', async () => {
  state.rows = { 'staff.Me': { view: 'staff.Me', rows: [{ id: 's-1', name: 'Example Librarian' }] } };
  setMode('preview');
  const html = await renderCanvas(app('staff.Me'), 'shell:main/account');
  assert.match(html, /class="chrome-account node selected"/);
  assert.match(text(html), /E Example Librarian ▾/);
});

test('an account menu on a draft view says its name is sample data, as every other draft read does', async () => {
  state.rows = { 'draft.Me': { view: 'draft.Me', rows: [{ id: 's-1', name: 'Sample Person' }] } };
  setMode('preview');
  const html = await renderCanvas(app('draft.Me'));
  assert.match(text(html), /Sample Person/, 'the made-up name is drawn');
  assert.match(html, /sample data|sample-tag/, `the made-up name carries no sample marker: ${JSON.stringify(text(html))}`);
});

test('accountName falls back to full_name, and trims what it returns', () => {
  assert.equal(accountName([{ id: 's-1', full_name: 'Full Name' }]), 'Full Name');
  assert.equal(accountName([{ id: 's-1', name: '  Padded  ' }]), 'Padded');
  assert.equal(accountName([{ id: 's-1', name: ' ', display_name: 'Shown' }]), 'Shown');
  assert.equal(accountName([{ id: 's-1', name: 42, email: 'a@example.com' }]), 'a@example.com');
});

// Round 2 (implementor): the rest of the classes the findings above are instances of.

test('the account chrome is announced by the name alone', async () => {
  state.rows = { 'staff.Me': { view: 'staff.Me', rows: [{ id: 's-1', name: 'Example Librarian' }] } };
  setMode('preview');
  const html = await renderCanvas(app('staff.Me'));
  const button = /<button[^>]*class="chrome-account[^"]*"[^>]*>[\s\S]*?<\/button>/.exec(html)?.[0] ?? '';
  assert.match(button, /aria-label="Example Librarian"/);
  assert.match(button, /<span[^>]*class="avatar"[^>]*aria-hidden="true"|<span[^>]*aria-hidden="true"[^>]*class="avatar"/, button);
  assert.match(button, /<span[^>]*class="caret"[^>]*aria-hidden="true"|<span[^>]*aria-hidden="true"[^>]*class="caret"/, button);
});

test('preview hides the view a menu entry opens per row of', async () => {
  state.rows = {};
  const nav = node('nav', 'menu', 'nav', { path: 'nav:menu' }, [
    node('nav_section', 'people', 'nav_section', { path: 'nav:menu/people', props: { page: 'member', from_view: 'members.All' } }),
  ]);
  const member = node('page', 'member', 'page', { title: 'Member' });
  const root = node('root', 'library', 'app', { path: '/' }, [nav, member]);
  setMode('structure');
  assert.match(text(await renderCanvas(root)), /per row of members\.All/, 'structure names the view');
  setMode('preview');
  assert.doesNotMatch(text(await renderCanvas(root)), /members\.All/);
});

// Pass 2 (adversary): the correction, attacked.

/** The accessible name a button gets: its aria-label when it has one, else its visible text. */
function accessibleNames(html: string, cls: string): string[] {
  const out: string[] = [];
  for (const m of html.matchAll(/<button([^>]*)>([\s\S]*?)<\/button>/g)) {
    const attrs = m[1];
    if (!new RegExp(`class="[^"]*\\b${cls}\\b`).test(attrs)) continue;
    const label = /aria-label="([^"]*)"/.exec(attrs)?.[1];
    const visible = text(m[2].replace(/<span[^>]*aria-hidden="true"[^>]*>[\s\S]*?<\/span>/g, ''));
    out.push(label ?? visible);
  }
  return out;
}

test('a draft account menu tells a screen reader its name is sample data, as it tells the eye', async () => {
  state.rows = { 'draft.Me': { view: 'draft.Me', rows: [{ id: 's-1', name: 'Pat Doe' }] } };
  setMode('preview');
  const html = await renderCanvas(app('draft.Me'), 'shell:main/account');
  assert.match(html, /class="chrome-account node selected"/, 'selection outline and sample tag together');
  assert.match(text(html), /Pat Doe sample data/, 'the eye sees the marker');
  const [name] = accessibleNames(html, 'chrome-account');
  assert.match(name ?? '', /sample data/i, `the accessible name drops the sample marker: ${JSON.stringify(name)}`);
});

test('preview names no view in the empty line of any row-reading composite, in any rows state', async () => {
  const kinds = ['collection', 'metric', 'record', 'chart'];
  const states: { label: string; conn: string; view: string | undefined; rows?: Rows['rows']; want: [string, string] }[] = [
    { label: 'connected, not loaded', conn: 'open', view: 'members.All', want: ['loading members.All…', 'loading…'] },
    { label: 'disconnected, not loaded', conn: 'closed', view: 'members.All', want: ['no data yet (members.All)', 'no data yet'] },
    { label: 'loaded, zero rows', conn: 'open', view: 'members.All', rows: [], want: ['no data yet (members.All)', 'no data yet'] },
    { label: 'draft, loading', conn: 'open', view: 'draft.Stats', want: ['loading draft.Stats…', 'loading…'] },
    { label: 'draft, zero rows', conn: 'open', view: 'draft.Stats', rows: [], want: ['no data yet (draft.Stats)', 'no data yet'] },
    { label: 'no view', conn: 'open', view: undefined, want: ['no data yet (no view)', 'no data yet'] },
  ];
  for (const s of states) {
    for (const kind of kinds) {
      state.conn = s.conn;
      state.rows = s.view && s.rows ? { [s.view]: { view: s.view, rows: s.rows } } : {};
      const n = node('section', 'sec', kind, { path: `page:p/${kind}`, ...(s.view ? { view: s.view } : {}), props: { columns: ['name'], from: 'n' } });
      const where = `${kind}, ${s.label}`;
      setMode('structure');
      const structure = text(await renderComposite(n));
      assert.ok(structure.includes(s.want[0]), `${where}: structure ${JSON.stringify(structure)}`);
      setMode('preview');
      const preview = text(await renderComposite(n));
      assert.ok(preview.includes(s.want[1]), `${where}: preview ${JSON.stringify(preview)}`);
      if (s.view) assert.ok(!preview.includes(s.view), `${where}: preview names the view ${JSON.stringify(preview)}`);
      assert.doesNotMatch(preview, /sample data/, `${where}: nothing made up is shown, so no sample tag`);
    }
  }
});

test('the bell is announced by the title its tooltip shows', async () => {
  state.rows = {};
  const doc = app('staff.Me');
  const bell = doc.children[0].children[1];
  bell.title = 'Alerts';
  setMode('preview');
  const html = await renderCanvas(doc);
  assert.match(html, /class="chrome-bell[^"]*"[^>]*title="Alerts"|title="Alerts"[^>]*class="chrome-bell/, 'the tooltip reads the title');
  assert.deepEqual(accessibleNames(html, 'chrome-bell'), ['Alerts']);
});
