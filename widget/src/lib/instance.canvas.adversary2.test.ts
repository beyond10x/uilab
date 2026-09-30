import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { compileScript, parse } from 'vue/compiler-sfc';
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../generated/types.ts';
import { canvasMode, type CanvasMode } from './canvasmode.ts';

// Adversary pass 2 on story:canvas-widget-instances, on the canvas itself: CanvasView,
// CompositeView and PrimitiveView compiled with vue/compiler-sfc and rendered to a string, as
// instance.canvas.test.ts does. The subject is the round-2 correction: primitive items read their
// row, an item list over no rows draws no item, args.* on a page is unbound.

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
  location: { search: '?name=InstancesAdversary2' },
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
  const script = compileScript(descriptor, { id: 'instances-adversary2', inlineTemplate: true });
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

function memberCard(): OutlineNode {
  return n(
    'component:member_card',
    'component',
    'widget',
    [
      n('component:member_card/node:name', 'node', 'text', [], { props: { text: 'args.member.name', style: 'heading' } }),
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
  conn = 'closed',
): Promise<string> {
  canvasMode.mode.value = mode;
  state.conn = conn;
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

function list(kind: string, items: OutlineNode[], view: string | undefined = 'members.All'): OutlineNode {
  return n('page:members/section:list', 'section', kind, items, view ? { view } : {});
}

for (const mode of ['structure', 'preview'] as const) {
  // A composite item is drawn once per row since this unit (main drew it once), so a row reference
  // in its title is now printed raw once per row, on a page with no widget instance at all.
  test(`${mode}: a composite item writing row.<field> in its title never shows the raw reference, one per row`, async () => {
    const head = n('page:members/section:list/item:head', 'item', 'header', [], { title: 'row.name' });
    const html = await render(page([list('collection', [head])], []), mode);
    assert.equal(count(html, /data-path="page:members\/section:list\/item:head"/g), MEMBERS.length);
    assert.doesNotMatch(text(html), /row\.name/, text(html));
  });

  test(`${mode}: an instance item writing row.<field> in its title never shows the raw reference`, async () => {
    const card = n('page:members/section:list/item:card', 'item', 'member_card', [], { title: 'row.name', props: { args: { member: 'row' } } });
    const html = await render(page([list('collection', [card])]), mode);
    assert.doesNotMatch(text(html), /row\.name/, text(html));
  });

  // `rows` is a reference for the server (check.rs is_reference) as `row` is, and bindArg reads it
  // for an instance item; a primitive item in the same list prints it as written.
  test(`${mode}: a primitive item writing rows.first.<field> never shows the raw reference`, async () => {
    const top = n('page:members/section:list/item:top', 'item', 'text', [], { props: { text: 'first: rows.first.name' } });
    const html = await render(page([list('collection', [top])], []), mode);
    assert.doesNotMatch(text(html), /rows\.first/, text(html));
  });

  // Over a loaded view with no rows an item list draws no item, but a section instance and a board
  // widget instance reading rows.first draw their body from made-up sample args, with neither the
  // empty line nor the sample-data tag: on the page that reads as data the view holds.
  test(`${mode}: a section instance reading rows.first of a loaded empty view says so, not sample data unmarked`, async () => {
    const spot = n('page:members/section:spot', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } }, view: 'members.All' });
    const html = await render(page([spot]), mode, { 'members.All': [] });
    assert.match(text(html), /no data yet|sample data/, text(html));
  });

  test(`${mode}: a board widget instance reading rows.first of a loaded empty board view says so`, async () => {
    const board = n(
      'page:members/section:kpis',
      'section',
      'board',
      [n('page:members/section:kpis/widget:top', 'widget', 'member_card', [], { props: { args: { member: 'rows.first' } } })],
      { view: 'members.All' },
    );
    const html = await render(page([board]), mode, { 'members.All': [] });
    assert.match(text(html), /no data yet|sample data/, text(html));
  });

  test(`${mode}: a record over zero rows draws no item and keeps its empty line; over two rows its items read the first`, async () => {
    const who = n('page:members/section:list/item:who', 'item', 'badge', [], { props: { text: 'row.standing' } });
    const empty = await render(page([list('record', [who])], []), mode, { 'members.All': [] });
    assert.equal(count(empty, /item:who/g), 0);
    assert.match(text(empty), /no data yet/);
    const full = await render(page([list('record', [who])], []), mode);
    assert.equal(count(full, /data-path="page:members\/section:list\/item:who"/g), 1);
    assert.match(text(full), /good/);
    assert.doesNotMatch(text(full), /late/);
  });

  test(`${mode}: a collection whose rows never loaded on a closed connection draws no item and says no data`, async () => {
    const tag = n('page:members/section:list/item:tag', 'item', 'badge', [], { props: { text: 'row.standing' } });
    const html = await render(page([list('collection', [tag])], []), mode, {}, null, 'closed');
    assert.equal(count(html, /item:tag/g), 0);
    assert.match(text(html), /no data yet/);
    assert.doesNotMatch(text(html), /loading/);
  });

  test(`${mode}: a primitive item over a draft view is drawn per row under the sample-data tag`, async () => {
    const tag = n('page:members/section:list/item:tag', 'item', 'badge', [], { props: { text: 'row.standing' } });
    const html = await render(page([list('collection', [tag], 'draft.members')], []), mode, { 'draft.members': MEMBERS });
    assert.match(html, /sample-tag/);
    assert.equal(count(html, /data-path="page:members\/section:list\/item:tag"/g), 2);
    assert.match(text(html), /good .*late/);
  });

  test(`${mode}: a chart over no rows keeps its empty line beside an instance`, async () => {
    const chart = n('page:members/section:chart', 'section', 'chart', [], { props: { x: 'name' }, view: 'members.All' });
    const spot = n('page:members/section:spot', 'section', 'member_card', [], { props: { args: { member: 'row' } } });
    const html = await render(page([chart, spot]), mode, { 'members.All': [] });
    assert.match(text(html), /no data yet/);
    assert.doesNotMatch(html, /class="placeholder">chart/);
  });
}

test('a proposal adding a primitive item to a loaded collection marks each row copy as added, read from its row', async () => {
  const before = page([list('collection', [], 'members.All')], []);
  const tag = n('page:members/section:list/item:tag', 'item', 'badge', [], { props: { text: 'row.standing' } });
  const after = page([list('collection', [tag], 'members.All')], []);
  const html = await render(before, 'structure', { 'members.All': MEMBERS }, { outline: after });
  assert.equal(count(html, /class="prim node hl-insert[^"]*"[^>]*data-path="page:members\/section:list\/item:tag"/g), 2);
  assert.match(text(html), /good .*late/);
});

test('a page with no instance and no item draws neither an instance body nor an item row', async () => {
  const table = n('page:members/section:table', 'section', 'collection', [], { props: { columns: [{ field: 'name' }] }, view: 'members.All' });
  const board = n(
    'page:members/section:kpis',
    'section',
    'board',
    [n('page:members/section:kpis/widget:count', 'widget', 'metric', [], { props: { from: 'name' }, view: 'members.All' })],
  );
  for (const mode of ['structure', 'preview'] as const) {
    const html = await render(page([table, board], []), mode);
    assert.doesNotMatch(html, /instance-body|item-rows|item-row/);
    assert.match(html, /class="children grid"/);
    assert.match(text(html), /Ada Lovelace .*Grace Hopper/);
  }
});
