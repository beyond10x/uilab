import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { primitiveShape } from './components.ts';

// story:essui-app-widget, `divider_renders_as_rule`: an ESS `primitive: divider` is drawn on the
// canvas as a horizontal rule, not as the hatched placeholder box. PrimitiveView is compiled with
// vue/compiler-sfc and rendered to a string.

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Divider' },
  localStorage: { getItem: () => null, setItem: () => undefined },
};
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
  const script = compileScript(descriptor, { id: 'divider', inlineTemplate: true });
  const code = stripTypeScriptTypes(script.content, { mode: 'strip' }).replace(
    /from\s+(['"])([^'"]+)\1/g,
    (_m, _q: string, spec: string) => (spec === 'vue' ? `from '${vueUrl}'` : `from '${new URL(spec, file).href}'`),
  );
  return `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`;
}

const PrimitiveView = ((await import(compileSfc(new URL('../components/PrimitiveView.vue', import.meta.url)))) as { default: unknown }).default;

function prim(kind: string, props?: Record<string, unknown>): OutlineNode {
  return { path: `component:card/node:${kind}`, layer: 'node', name: kind, kind, children: [], ...(props ? { props } : {}) } as OutlineNode;
}

test('a divider is a rule; the other drawn primitives keep their shapes', () => {
  assert.equal(primitiveShape(prim('divider')), 'rule');
  assert.equal(primitiveShape(prim('text')), 'text');
  assert.equal(primitiveShape(prim('text', { style: 'heading' })), 'heading');
  assert.equal(primitiveShape(prim('badge')), 'badge');
  assert.equal(primitiveShape(prim('image')), 'image');
  assert.equal(primitiveShape(prim('link')), 'link');
  assert.equal(primitiveShape(prim('button')), 'button');
  assert.equal(primitiveShape(prim('toggle')), 'toggle');
});

test('the canvas draws a divider as a horizontal rule, not the placeholder box', async () => {
  const html = await renderToString(createSSRApp({ render: () => h(PrimitiveView, { node: prim('divider') }) }));
  assert.match(html, /<hr[^>]*class="[^"]*prim-rule/);
  assert.doesNotMatch(html, /placeholder/);
});
