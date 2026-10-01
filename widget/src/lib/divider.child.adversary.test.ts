import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary pass 1, story:essui-app-widget, `divider_renders_as_rule`. ESS's Section construct:
// "`children` adds widgets or primitives rendered with the section (a caption, a button)". A
// `primitive: divider` among a section's children reaches the canvas at layer `child`, kind
// `divider` (`uilab-doc` outline). The unit's own case renders PrimitiveView directly; this one
// renders the canvas, which decides whether PrimitiveView is reached at all.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=DividerChild' },
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
  const script = compileScript(descriptor, { id: 'dividerchild', inlineTemplate: true });
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

/** The Loans page with a list section that holds a divider among its `children`. */
function library(): OutlineNode {
  return n('/', 'root', 'document', [
    n('page:loans', 'page', 'list_page', [
      n('page:loans/section:list', 'section', 'collection', [n('page:loans/section:list/child:rule', 'child', 'divider')], {
        view: 'loans.All',
        props: { columns: [{ field: 'title' }] },
      }),
    ], { title: 'Loans' }),
  ]);
}

async function render(mode: CanvasMode): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = { 'loans.All': { view: 'loans.All', rows: [{ title: 'Dune' }] } as Rows };
  state.doc = { outline: library(), selected: '/' } as StoreApi['state']['doc'];
  state.currentPage = 'page:loans';
  return renderToString(createSSRApp({ render: () => h(CanvasView) }));
}

for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: a divider among a section's children is drawn as a rule, not the placeholder box`, async () => {
    const html = await render(mode);
    assert.match(html, /data-path="page:loans\/section:list\/child:rule"/, 'the divider is drawn');
    assert.match(html, /<hr[^>]*class="[^"]*prim-rule/, `no rule in: ${html}`);
    assert.doesNotMatch(html, /class="placeholder">divider/, 'drawn as the placeholder box');
  });
}
