import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { outlineMarks, withRemoved } from './marks.ts';
import { navLayout } from './outline.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

function asObject(m: Map<string, string>): Record<string, string> {
  return Object.fromEntries([...m.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

/**
 * The shapes below are the server's (`uilab_doc::outline`, read from `/api/state` over a copy of the
 * operator's document): a widget node carries `uses`, its instances' paths, in its props; a nav
 * section carries its pages as `props.pages`; a page carries `props.shell`.
 */
function widget(uses: { path: string }[]): OutlineNode {
  return n(
    'component:member_card',
    'component',
    'widget',
    [n('component:member_card/node:name', 'node', 'text', [], { props: { text: 'args.member.name' } })],
    { title: "Card showing a member's name", props: { params: { member: { type: 'Member', required: true } }, arrange: 'column', uses } },
  );
}

function page(name: string, children: OutlineNode[] = []): OutlineNode {
  return n(`page:${name}`, 'page', 'list_page', children, { props: { shell: 'app' } });
}

function nav(pages: string[]): OutlineNode {
  return n('nav', 'nav', 'navigation', [n('nav/nav_section:circulation', 'nav_section', 'nav_section', [], { title: 'Circulation', props: { pages } })], {
    props: { home: 'overview' },
  });
}

const list = n('page:members/section:list', 'section', 'collection', [], { view: 'members.All', props: { columns: ['name'] } });

/**
 * Acceptance: "an Insert marks only the new subtree". Inserting an instance of a widget changes
 * nothing about the widget, yet the server recomputes the widget node's `uses` from its instances,
 * so the widget's props differ and it is marked changed in the tree and the Components tab.
 */
test('inserting a widget instance marks only the new instance, not the widget it instantiates', () => {
  const doc = n('/', 'root', 'document', [nav(['overview', 'members']), widget([]), page('overview'), page('members', [list])]);
  const card = n('page:members/section:card', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } } });
  const proposal = n('/', 'root', 'document', [
    nav(['overview', 'members']),
    widget([{ path: 'page:members/section:card' }]),
    page('overview'),
    page('members', [list, card]),
  ]);
  assert.deepEqual(asObject(outlineMarks(doc, proposal)), { 'page:members/section:card': 'added' });
});

/**
 * Brief: "the canvas shows the proposal's outline with removed nodes drawn from the document's (as
 * today for a Remove)". Before the unit a Remove showed the document's outline, so the menu listed
 * the removed page, struck through. The server's Remove of a page also drops it from its nav
 * section's `pages` (`uilab-doc/src/patch.rs` `remove`), and `withRemoved` puts the page node back
 * but keeps the proposal's section, so the menu the canvas draws from the shown outline loses it.
 */
test('a page removal still lists the removed page in the canvas menu', () => {
  const doc = n('/', 'root', 'document', [nav(['overview', 'members', 'loans']), page('overview'), page('members'), page('loans')]);
  const proposal = n('/', 'root', 'document', [nav(['overview', 'members']), page('overview'), page('members')]);
  const shown = withRemoved(doc, proposal);
  const menu = navLayout(shown).flatMap((g) => g.entries.map((e) => e.page.path));
  assert.deepEqual(menu, ['page:overview', 'page:members', 'page:loans']);
});

/**
 * The unit's own test says a remove marks "the removed node and its whole subtree removed, and
 * nothing else", on a fixture whose nav has no section listing the page. With one (the operator's
 * document lists every page in a section), the section is marked changed as well.
 */
test('a page removal marks nothing but the removed page, also when a nav section lists it', () => {
  const doc = n('/', 'root', 'document', [nav(['overview', 'loans']), page('overview'), page('loans')]);
  const proposal = n('/', 'root', 'document', [nav(['overview']), page('overview')]);
  assert.deepEqual(asObject(outlineMarks(doc, proposal)), { 'page:loans': 'removed' });
});

test('a rename is the old path removed and the new one added, the old one after its document predecessor', () => {
  const doc = n('/', 'root', 'document', [page('a'), page('b'), page('c')]);
  const proposal = n('/', 'root', 'document', [page('a'), page('bee'), page('c')]);
  assert.deepEqual(asObject(outlineMarks(doc, proposal)), { 'page:b': 'removed', 'page:bee': 'added' });
  assert.deepEqual(
    withRemoved(doc, proposal).children.map((c) => c.path),
    ['page:a', 'page:b', 'page:bee', 'page:c'],
  );
});

test('a widget body node removed marks that node only; its widget and the ancestors stay unmarked', () => {
  const two = n('component:member_card', 'component', 'widget', [
    n('component:member_card/node:name', 'node', 'text'),
    n('component:member_card/node:joined', 'node', 'text'),
  ]);
  const one = n('component:member_card', 'component', 'widget', [n('component:member_card/node:name', 'node', 'text')]);
  const doc = n('/', 'root', 'document', [two]);
  const proposal = n('/', 'root', 'document', [one]);
  assert.deepEqual(asObject(outlineMarks(doc, proposal)), { 'component:member_card/node:joined': 'removed' });
  assert.deepEqual(
    withRemoved(doc, proposal).children[0].children.map((c) => c.path),
    ['component:member_card/node:name', 'component:member_card/node:joined'],
  );
});

test('an array prop in another order is changed; an object prop in another key order is not', () => {
  const a = n('/', 'root', 'document', [
    n('page:p', 'page', 'list_page', [
      n('page:p/section:s', 'section', 'collection', [], { props: { columns: ['name', 'email'] } }),
      n('page:p/section:t', 'section', 'collection', [], { props: { a: { x: 1, y: 2 }, b: null } }),
    ]),
  ]);
  const b = n('/', 'root', 'document', [
    n('page:p', 'page', 'list_page', [
      n('page:p/section:s', 'section', 'collection', [], { props: { columns: ['email', 'name'] } }),
      n('page:p/section:t', 'section', 'collection', [], { props: { b: null, a: { y: 2, x: 1 } } }),
    ]),
  ]);
  assert.deepEqual(asObject(outlineMarks(a, b)), { 'page:p/section:s': 'changed' });
});

test('a proposal that reorders and removes keeps every node exactly once', () => {
  const kids = (names: string[]) => names.map((x) => n(`page:p/section:${x}`, 'section', 'collection'));
  const doc = n('/', 'root', 'document', [n('page:p', 'page', 'list_page', kids(['a', 'b', 'c', 'd']))]);
  const proposal = n('/', 'root', 'document', [n('page:p', 'page', 'list_page', kids(['d', 'a']))]);
  const shown = withRemoved(doc, proposal).children[0].children.map((c) => c.name);
  assert.deepEqual([...shown].sort(), ['a', 'b', 'c', 'd']);
  assert.deepEqual(shown, ['d', 'a', 'b', 'c']);
});

test('a thousand-node outline with every leaf renamed marks each once', () => {
  const pages = (suffix: string) =>
    Array.from({ length: 10 }, (_, i) =>
      n(`page:p${i}`, 'page', 'list_page', Array.from({ length: 100 }, (_, j) => n(`page:p${i}/section:s${j}${suffix}`, 'section', 'collection'))),
    );
  const doc = n('/', 'root', 'document', pages(''));
  const proposal = n('/', 'root', 'document', pages('x'));
  const marks = outlineMarks(doc, proposal);
  assert.equal(marks.size, 2000);
  assert.equal([...marks.values()].filter((m) => m === 'removed').length, 1000);
  const shown = withRemoved(doc, proposal);
  assert.equal(shown.children.reduce((k, p) => k + p.children.length, 0), 2000);
});
