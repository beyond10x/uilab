import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, nodeActions, removeInstruction, type CanvasMode } from './canvasmode.ts';

// story:essui-app-widget, `canvasmode.inherited.test.ts`. A node a page kind or a widget
// contributes is not written in the document, so the canvas offers no remove on it; a node the
// author wrote keeps the offer.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Remove' },
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
  const script = compileScript(descriptor, { id: 'remove', inlineTemplate: true });
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

const filters = n('page:loans/section:filters', 'section', 'filter_bar', [], { inherited: true });
const list = n('page:loans/section:list', 'section', 'collection', [], { view: 'loans.All' });

function library(): OutlineNode {
  return n('/', 'root', 'document', [n('page:loans', 'page', 'list_page', [filters, list], { title: 'Loans' })]);
}

async function render(selected: string, mode: CanvasMode = 'structure'): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = { 'loans.All': { view: 'loans.All', rows: [] } as Rows };
  state.doc = { outline: library(), selected } as StoreApi['state']['doc'];
  state.currentPage = 'page:loans';
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

/** The HTML of the card at `path`, up to the next card. */
function card(html: string, path: string): string {
  const start = html.indexOf(`data-path="${path}"`);
  assert.ok(start >= 0, `nothing on the canvas has data-path ${path}`);
  const rest = html.slice(start);
  const next = [...rest.matchAll(/data-path="([^"]+)"/g)].find((m) => m.index > 0 && !m[1].startsWith(path));
  return next ? rest.slice(0, next.index) : rest;
}

const REMOVE = /class="[^"]*card-remove[^"]*"/;

test('an inherited node is offered no remove; a node the author wrote is', () => {
  assert.deepEqual(nodeActions(filters, 'structure'), []);
  assert.deepEqual(nodeActions(list, 'structure'), ['remove']);
});

test('the canvas draws no remove on a selected inherited section', async () => {
  assert.doesNotMatch(card(await render(filters.path), filters.path), REMOVE);
});

test('the canvas draws a remove on a selected section the author wrote, and none on one not selected', async () => {
  const html = await render(list.path);
  assert.match(card(html, list.path), REMOVE);
  assert.doesNotMatch(card(html, filters.path), REMOVE);
});

test('preview offers no remove on any node', async () => {
  assert.deepEqual(nodeActions(list, 'preview'), []);
  assert.doesNotMatch(card(await render(list.path, 'preview'), list.path), REMOVE);
});

test('the root and the menu are not removable', () => {
  assert.deepEqual(nodeActions(n('/', 'root', 'document'), 'structure'), []);
  assert.deepEqual(nodeActions({ path: 'nav', layer: 'nav', name: '', kind: 'navigation', children: [] }, 'structure'), []);
});

test('a remove asks the agent to remove that node by layer and name', () => {
  assert.equal(removeInstruction(list), 'remove the section list');
});
