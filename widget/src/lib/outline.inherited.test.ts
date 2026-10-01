import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { inheritedMark } from './outline.ts';

// story:essui-app-widget, `outline.inherited.test.ts`. The outline tree marks a node a page kind or
// a widget contributes: it is shown, but the author did not write it. TreeNode is compiled with
// vue/compiler-sfc and rendered to a string.

interface StoreApi {
  state: { expanded: Set<string> };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Tree' },
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

function compileSfc(file: URL): string {
  const { descriptor } = parse(readFileSync(file, 'utf8'), { filename: file.pathname });
  const script = compileScript(descriptor, { id: 'tree', inlineTemplate: true });
  const code = stripTypeScriptTypes(script.content, { mode: 'strip' }).replace(
    /from\s+(['"])([^'"]+)\1/g,
    (_m, _q: string, spec: string) => (spec === 'vue' ? `from '${vueUrl}'` : `from '${new URL(spec, file).href}'`),
  );
  return `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`;
}

const TreeNode = ((await import(compileSfc(new URL('../components/TreeNode.vue', import.meta.url)))) as { default: unknown }).default;

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

const filters = n('page:loans/section:filters', 'section', 'filter_bar', [], { inherited: true });
const list = n('page:loans/section:list', 'section', 'collection');
const loans = n('page:loans', 'page', 'list_page', [filters, list]);

/** The tree row of `path`: from its `title` to the end of its row. */
function row(html: string, path: string): string {
  const start = html.indexOf(`title="${path}"`);
  assert.ok(start >= 0, `no tree row for ${path}`);
  const rowStart = html.lastIndexOf('<div', start);
  return html.slice(rowStart, html.indexOf('</div>', start) + '</div>'.length);
}

test('an inherited node has the inherited mark; a node the author wrote has none', () => {
  assert.ok(inheritedMark(filters));
  assert.equal(inheritedMark(list), null);
  assert.equal(inheritedMark({ ...list, inherited: false }), null);
});

test('the tree renders the inherited mark on the inherited row only', async () => {
  state.expanded.add(loans.path);
  const html = await renderToString(createSSRApp({ render: () => h(TreeNode, { node: loans, depth: 0 }) }));
  const mark = inheritedMark(filters)!;
  const inherited = row(html, filters.path);
  assert.match(inherited, /class="[^"]*\binherited-mark\b[^"]*"/);
  assert.ok(inherited.includes(mark), `the row shows ${mark}`);
  assert.match(inherited, /class="tree-row[^"]*\binherited\b/);
  const own = row(html, list.path);
  assert.doesNotMatch(own, /inherited/);
});
