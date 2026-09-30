import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary cases for story:canvas-widget-instances, on the canvas itself: CanvasView,
// CompositeView and PrimitiveView compiled with vue/compiler-sfc and rendered to a string, as
// instance.canvas.test.ts does.

interface StoreApi {
  state: {
    conn: string;
    doc: { outline: OutlineNode; selected: string } | null;
    rows: Record<string, Rows>;
    currentPage: string | null;
    proposal: unknown;
    proposalBase: OutlineNode | null;
  };
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=InstancesAdversary' },
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
  const script = compileScript(descriptor, { id: 'instances-adversary', inlineTemplate: true });
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

const MEMBERS = [
  { id: 'm-1', name: 'Ada Lovelace', standing: 'good' },
  { id: 'm-2', name: 'Grace Hopper', standing: 'late' },
];

function memberCard(text = 'args.member.name'): OutlineNode {
  return n(
    'component:member_card',
    'component',
    'widget',
    [
      n('component:member_card/node:name', 'node', 'text', [], { props: { text, style: 'heading' } }),
      n('component:member_card/node:standing', 'node', 'badge', [], { props: { text: 'args.member.standing' } }),
    ],
    { title: 'A member', props: { params: { member: { type: 'Member', required: true } }, arrange: 'column', uses: [] } },
  );
}

function page(sections: OutlineNode[], widgets: OutlineNode[] = [memberCard()]): OutlineNode {
  return n('/', 'root', 'document', [...widgets, n('page:members', 'page', 'list_page', sections, { title: 'Members' })]);
}

async function render(
  outline: OutlineNode,
  mode: CanvasMode = 'structure',
  rows: Record<string, unknown[]> = { 'members.All': MEMBERS },
  proposal: { outline: OutlineNode } | null = null,
): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = 'closed';
  state.rows = Object.fromEntries(Object.entries(rows).map(([view, r]) => [view, { view, rows: r } as Rows]));
  state.doc = { outline, selected: '/' } as StoreApi['state']['doc'];
  state.proposal = proposal ? { proposal_id: 'p-1', outline: proposal.outline } : null;
  state.proposalBase = proposal ? outline : null;
  state.currentPage = 'page:members';
  try {
    return await renderToString(createSSRApp({ render: () => h(CanvasView) }));
  } finally {
    state.proposal = null;
    state.proposalBase = null;
  }
}

function text(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
}

function count(html: string, re: RegExp): number {
  return [...html.matchAll(re)].length;
}

const listOfCards = (view: string) =>
  n(
    'page:members/section:list',
    'section',
    'collection',
    [n('page:members/section:list/item:card', 'item', 'member_card', [], { props: { args: { member: 'row' } } })],
    { view },
  );

test('a collection item instance over a loaded view with no rows draws no item: one per row is none', async () => {
  const html = await render(page([listOfCards('members.All')]), 'preview', { 'members.All': [] });
  assert.equal(count(html, /data-widget="member_card"/g), 0, 'an item is drawn for a row that does not exist');
});

test('depth 3: each instance passes args.<param> through to the next, the innermost shows the bound row', async () => {
  const outer = n('component:outer', 'component', 'widget', [n('component:outer/node:mid', 'node', 'middle', [], { props: { args: { member: 'args.member' } } })], {
    props: { params: { member: { type: 'Member', required: true } }, uses: [] },
  });
  const middle = n('component:middle', 'component', 'widget', [n('component:middle/node:in', 'node', 'inner', [], { props: { args: { m: 'args.member' } } })], {
    props: { params: { member: { type: 'Member', required: true } }, uses: [] },
  });
  const inner = n('component:inner', 'component', 'widget', [n('component:inner/node:t', 'node', 'text', [], { props: { text: 'args.m.name' } })], {
    props: { params: { m: { type: 'Member', required: true } }, uses: [] },
  });
  const outline = page([n('page:members/section:spot', 'section', 'outer', [], { props: { args: { member: 'rows.1' } }, view: 'members.All' })], [outer, middle, inner]);
  const html = await render(outline, 'preview');
  assert.equal(count(html, /data-widget="outer"/g), 1);
  assert.equal(count(html, /data-widget="middle"/g), 1);
  assert.equal(count(html, /data-widget="inner"/g), 1);
  assert.match(text(html), /Grace Hopper/);
});

test('mutual recursion across two widgets draws each once and stops at the one already drawn', async () => {
  const a = n('component:a', 'component', 'widget', [n('component:a/node:b', 'node', 'b', [], { props: { args: {} } })], { props: { uses: [] } });
  const b = n('component:b', 'component', 'widget', [n('component:b/node:a', 'node', 'a', [], { props: { args: {} } })], { props: { uses: [] } });
  const html = await render(page([n('page:members/section:s', 'section', 'a', [], { props: { args: {} } })], [a, b]));
  assert.equal(count(html, /data-widget="a"/g), 1);
  assert.equal(count(html, /data-widget="b"/g), 1);
  assert.match(html, /class="placeholder">a</);
});

test('a proposal changing the widget body shows on the page instance, marked as changed', async () => {
  const before = page([n('page:members/section:spot', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } }, view: 'members.All' })]);
  const after = page(
    [n('page:members/section:spot', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } }, view: 'members.All' })],
    [memberCard('Member: args.member.name')],
  );
  const html = await render(before, 'structure', { 'members.All': MEMBERS }, { outline: after });
  assert.match(text(html), /Member: Ada Lovelace/);
  assert.match(html, /class="prim node hl-replace[^"]*"[^>]*data-path="component:member_card\/node:name"/);
});

test('preview drops the name · kind label of a composite inside an instance body', async () => {
  const board = n(
    'component:roster',
    'component',
    'widget',
    [n('component:roster/node:list', 'node', 'collection', [], { props: { columns: [{ field: 'name' }] }, view: 'members.All' })],
    { props: { uses: [] } },
  );
  const outline = page([n('page:members/section:r', 'section', 'roster', [], { props: { args: {} } })], [board]);
  assert.match(text(await render(outline, 'structure')), /list · collection/);
  const shown = text(await render(outline, 'preview'));
  assert.doesNotMatch(shown, /list · collection/);
  assert.match(shown, /Grace Hopper/);
});

test('an instance item of a collection reading a draft view keeps the sample-data tag and draws per row', async () => {
  const html = await render(page([listOfCards('draft.members')]), 'preview', { 'draft.members': MEMBERS });
  assert.match(html, /sample-tag/);
  assert.equal(count(html, /data-widget="member_card"/g), 2);
});

test('an instance of a widget no longer declared draws the placeholder named after it', async () => {
  const html = await render(page([n('page:members/section:gone', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } } })], []));
  assert.match(html, /class="placeholder">member_card</);
  assert.equal(count(html, /data-widget=/g), 0);
});

test('row in a section outside any collection takes the sample row', async () => {
  const reader = n('page:members/section:all', 'section', 'metric', [], { props: { from: 'name' }, view: 'members.All' });
  const html = await render(page([n('page:members/section:s', 'section', 'member_card', [], { props: { args: { member: 'row' } } }), reader]), 'preview');
  assert.match(text(html), /Ada Lovelace good/);
});

// A page with no widget instance at all: a collection whose items are primitives writing `row.<field>`,
// the documented item form. Main drew each item once as a `tag · badge` card; the unit draws it
// once per row as a primitive, and must not print the unresolved reference as the item's words.
for (const mode of ['structure', 'preview'] as const) {
  test(`${mode}: a primitive collection item on a page with no instances never shows the raw row.<field> text`, async () => {
    const list = n(
      'page:members/section:list',
      'section',
      'collection',
      [n('page:members/section:list/item:tag', 'item', 'badge', [], { props: { text: 'row.standing' } })],
      { view: 'members.All', props: { columns: [{ field: 'name' }] } },
    );
    const shown = text(await render(page([list], []), mode));
    assert.doesNotMatch(shown, /row\.standing/, shown);
  });
}

test('50 rows of a collection item whose widget nests another instance: every row drawn, in bounded time', async () => {
  const many = Array.from({ length: 50 }, (_, i) => ({ id: `m-${i}`, name: `Member ${i}`, standing: i % 2 ? 'late' : 'good' }));
  const wrap = n('component:wrap', 'component', 'widget', [n('component:wrap/node:card', 'node', 'member_card', [], { props: { args: { member: 'args.member' } } })], {
    props: { params: { member: { type: 'Member', required: true } }, uses: [] },
  });
  const list = n(
    'page:members/section:list',
    'section',
    'collection',
    [n('page:members/section:list/item:w', 'item', 'wrap', [], { props: { args: { member: 'row' } } })],
    { view: 'members.All' },
  );
  const started = performance.now();
  const html = await render(page([list], [memberCard(), wrap]), 'preview', { 'members.All': many });
  const took = performance.now() - started;
  assert.equal(count(html, /data-widget="wrap"/g), 50);
  assert.equal(count(html, /data-widget="member_card"/g), 50);
  assert.match(text(html), /Member 0 good .*Member 49 late/);
  assert.ok(took < 2000, `took ${Math.round(took)} ms`);
});
