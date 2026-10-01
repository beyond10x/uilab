import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';
import { isSample, rowsDue } from './rows.ts';

// story:essui-app-widget. A placeholder read (`reads: {placeholder, fixture}`) shows its fixture's
// rows; with no fixture file the server makes rows up and says so (`Rows.sample`), and the canvas
// tags them as sample data. Whether rows are made up is the server's answer, not a guess from the
// view's name.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Samples' },
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
  const script = compileScript(descriptor, { id: 'samples', inlineTemplate: true });
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
const CompositeView = ((await import(compileSfc(new URL('../components/CompositeView.vue', import.meta.url), { './PrimitiveView.vue': primitiveUrl }))) as { default: unknown }).default;

const section: OutlineNode = {
  path: 'page:loans/section:history',
  layer: 'section',
  name: 'history',
  kind: 'collection',
  view: 'LoanHistory',
  props: { columns: [{ field: 'title' }] },
  children: [],
};

async function render(rows: Rows, mode: CanvasMode): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = { LoanHistory: rows };
  state.doc = { outline: { path: '/', layer: 'root', name: '', kind: 'document', children: [] }, selected: '/' } as StoreApi['state']['doc'];
  return renderToString(createSSRApp({ render: () => h(CompositeView, { node: section }) }));
}

const ROWS = [{ title: 'Dune' }];

test('rows the server made up are samples; fixture rows and an empty answer are not', () => {
  assert.equal(isSample({ view: 'LoanHistory', rows: ROWS, sample: true }), true);
  assert.equal(isSample({ view: 'LoanHistory', rows: ROWS, sample: false }), false);
  assert.equal(isSample({ view: 'LoanHistory', rows: ROWS }), false);
  assert.equal(isSample({ view: 'LoanHistory', rows: [], sample: true }), false);
  assert.equal(isSample(undefined), false);
});

for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: made-up rows of a placeholder read are tagged as sample data; fixture rows are not`, async () => {
    assert.match(await render({ view: 'LoanHistory', rows: ROWS, sample: true }, mode), /sample-tag/);
    assert.doesNotMatch(await render({ view: 'LoanHistory', rows: ROWS, sample: false }, mode), /sample-tag/);
  });
}

test('made-up rows are asked for again when the document moves on; fixture rows once', () => {
  assert.equal(rowsDue('LoanHistory', true, 1, 2, true), true);
  assert.equal(rowsDue('LoanHistory', true, 2, 2, true), false);
  assert.equal(rowsDue('LoanHistory', true, 1, 2, false), false);
  assert.equal(rowsDue('LoanHistory', true, 1, 2), false);
});
