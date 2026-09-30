import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { EssJsonValue, UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { DERIVED_PROPS, markClasses, outlineMarks, previewBase, withRemoved } from './marks.ts';
import { navLayout } from './outline.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

function members(sectionProps: { [key: string]: EssJsonValue }, overlays: OutlineNode[] = []): OutlineNode {
  return n('page:members', 'page', 'list_page', [
    n('page:members/section:list', 'section', 'collection', [n('page:members/section:list/item:name', 'item', 'field')], {
      view: 'members.All',
      props: sectionProps,
    }),
    ...overlays,
  ]);
}

function loans(): OutlineNode {
  return n('page:loans', 'page', 'list_page', [
    n('page:loans/section:list', 'section', 'collection', [n('page:loans/section:list/item:status', 'item', 'record')], {
      view: 'loans.All',
    }),
  ]);
}

function doc(pages: OutlineNode[]): OutlineNode {
  return n('/', 'root', 'document', [n('nav', 'nav', 'navigation', [], { props: { home: 'members' } }), ...pages]);
}

const base = doc([members({ columns: ['name', 'email'] }), loans()]);

function asObject(m: Map<string, string>): Record<string, string> {
  return Object.fromEntries([...m.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

test('a batch of insert and replace marks the new node added and the replaced one changed, nothing removed', () => {
  const dialog = n('page:members/overlay:extend_loan', 'overlay', 'dialog form', [], { title: 'Extend loan' });
  const proposal = doc([
    members({ columns: ['name', 'email'], row_actions: [{ opens: 'extend_loan', label: 'Extend' }] }, [dialog]),
    loans(),
  ]);
  assert.deepEqual(asObject(outlineMarks(base, proposal)), {
    'page:members/overlay:extend_loan': 'added',
    'page:members/section:list': 'changed',
  });
});

test('a remove marks the removed node and its whole subtree removed, and nothing else', () => {
  const proposal = doc([members({ columns: ['name', 'email'] })]);
  assert.deepEqual(asObject(outlineMarks(base, proposal)), {
    'page:loans': 'removed',
    'page:loans/section:list': 'removed',
    'page:loans/section:list/item:status': 'removed',
  });
});

test('an insert marks only the new subtree added; its parent is not changed', () => {
  const inserted = n('page:members/section:stats', 'section', 'stats', [n('page:members/section:stats/item:count', 'item', 'metric')]);
  const proposal = doc([
    n('page:members', 'page', 'list_page', [...members({ columns: ['name', 'email'] }).children, inserted]),
    loans(),
  ]);
  assert.deepEqual(asObject(outlineMarks(base, proposal)), {
    'page:members/section:stats': 'added',
    'page:members/section:stats/item:count': 'added',
  });
});

test('title, kind and view each make a node changed; props equal up to key order do not', () => {
  const a = n('/', 'root', 'document', [
    n('page:a', 'page', 'list_page', [], { title: 'A' }),
    n('page:b', 'page', 'list_page'),
    n('page:c', 'page', 'list_page', [n('page:c/section:s', 'section', 'collection', [], { view: 'c.All' })]),
    n('page:d', 'page', 'list_page', [n('page:d/section:s', 'section', 'collection', [], { props: { x: 1, y: [1, { z: 2 }] } })]),
  ]);
  const b = n('/', 'root', 'document', [
    n('page:a', 'page', 'list_page', [], { title: 'A2' }),
    n('page:b', 'page', 'form_page'),
    n('page:c', 'page', 'list_page', [n('page:c/section:s', 'section', 'collection', [], { view: 'c.Open' })]),
    n('page:d', 'page', 'list_page', [n('page:d/section:s', 'section', 'collection', [], { props: { y: [1, { z: 2 }], x: 1 } })]),
  ]);
  assert.deepEqual(asObject(outlineMarks(a, b)), {
    'page:a': 'changed',
    'page:b': 'changed',
    'page:c/section:s': 'changed',
  });
});

test('identical outlines carry no marks', () => {
  assert.equal(outlineMarks(base, structuredClone(base)).size, 0);
});

test('marks are keyed by normalized path', () => {
  const proposal = doc([members({ columns: ['name', 'email'] }), loans(), n('/page:extra/', 'page', 'list_page')]);
  assert.deepEqual([...outlineMarks(base, proposal).keys()], ['page:extra']);
});

test('the shown outline is the proposal with removed nodes kept at their document position', () => {
  const proposal = doc([loans()]);
  const shown = withRemoved(base, proposal);
  assert.deepEqual(
    shown.children.map((c) => c.path),
    ['nav', 'page:members', 'page:loans'],
  );
  assert.deepEqual(shown.children[1], base.children[1]);
});

test('removed children interleave with the proposal children by document order', () => {
  const d = n('/', 'root', 'document', [
    n('page:p', 'page', 'list_page', [
      n('page:p/section:a', 'section', 'collection'),
      n('page:p/section:b', 'section', 'collection'),
      n('page:p/section:c', 'section', 'collection'),
    ]),
  ]);
  const p = n('/', 'root', 'document', [
    n('page:p', 'page', 'list_page', [n('page:p/section:new', 'section', 'collection'), n('page:p/section:b', 'section', 'collection')]),
  ]);
  assert.deepEqual(
    withRemoved(d, p).children[0].children.map((c) => c.name),
    ['a', 'new', 'b', 'c'],
  );
});

test('with nothing removed the shown outline is the proposal itself', () => {
  const proposal = doc([members({ columns: ['name'] }), loans()]);
  assert.equal(withRemoved(base, proposal), proposal);
});

test('each mark has its own class, and no mark sets none', () => {
  assert.deepEqual(markClasses('added'), { 'hl-insert': true, 'hl-replace': false, removed: false });
  assert.deepEqual(markClasses('changed'), { 'hl-insert': false, 'hl-replace': true, removed: false });
  assert.deepEqual(markClasses('removed'), { 'hl-insert': false, 'hl-replace': false, removed: true });
  assert.deepEqual(markClasses(undefined), { 'hl-insert': false, 'hl-replace': false, removed: false });
});

test('marks diff against the document the proposal arrived with, not a later one', () => {
  const p = { proposal_id: 'p1' };
  const proposal = doc([members({ columns: ['name'] }), loans()]);
  const arrived = previewBase(null, p, null, base);
  assert.equal(arrived, base);
  // Another client's undo drops the loans page; the card stays open.
  const later = doc([members({ columns: ['name', 'email'] })]);
  const kept = previewBase(p, p, arrived, later);
  assert.equal(kept, base);
  assert.deepEqual(asObject(outlineMarks(kept!, proposal)), { 'page:members/section:list': 'changed' });
  assert.deepEqual(withRemoved(kept!, proposal).children.map((c) => c.path), ['nav', 'page:members', 'page:loans']);
});

test('a new proposal takes the document current when it arrives; no proposal has no base', () => {
  const later = doc([loans()]);
  assert.equal(previewBase({ proposal_id: 'p1' }, { proposal_id: 'p2' }, base, later), later);
  assert.equal(previewBase({ proposal_id: 'p1' }, null, base, later), null);
  assert.equal(previewBase(null, { proposal_id: 'p1' }, null, null), null);
  assert.equal(previewBase({ proposal_id: 'p1' }, { proposal_id: 'p1' }, null, later), later);
});

test('a derived prop is left out of the comparison; every other prop still counts', () => {
  assert.deepEqual(DERIVED_PROPS, { component: ['uses'], nav_section: ['pages'] });
  const widget = (props: { [key: string]: EssJsonValue }) => n('component:card', 'component', 'widget', [], { props });
  const section = (props: { [key: string]: EssJsonValue }) =>
    n('nav', 'nav', 'navigation', [n('nav/nav_section:main', 'nav_section', 'nav_section', [], { props })]);
  const a = n('/', 'root', 'document', [section({ pages: ['a'] }), widget({ arrange: 'column', uses: [] })]);
  const b = n('/', 'root', 'document', [section({ pages: ['a', 'b'] }), widget({ arrange: 'column', uses: [{ path: 'page:a/section:x' }] })]);
  assert.equal(outlineMarks(a, b).size, 0);
  const c = n('/', 'root', 'document', [section({ pages: ['a'], from_view: 'v' }), widget({ arrange: 'row', uses: [] })]);
  assert.deepEqual(asObject(outlineMarks(a, c)), { 'component:card': 'changed', 'nav/nav_section:main': 'changed' });
  // The same key on another layer is the document's own prop.
  const s = (props: { [key: string]: EssJsonValue }) =>
    n('/', 'root', 'document', [n('page:a', 'page', 'list_page', [n('page:a/section:x', 'section', 'collection', [], { props })])]);
  assert.deepEqual(asObject(outlineMarks(s({ uses: 1, pages: 1 }), s({ uses: 2, pages: 1 }))), { 'page:a/section:x': 'changed' });
});

function menu(root: OutlineNode): string[][] {
  return navLayout(root).map((g) => g.entries.map((e) => e.page.name));
}

function navDoc(sections: [string, string[]][], pages: string[]): OutlineNode {
  return n('/', 'root', 'document', [
    n(
      'nav',
      'nav',
      'navigation',
      sections.map(([s, ps]) => n(`nav/nav_section:${s}`, 'nav_section', 'nav_section', [], { props: { pages: ps } })),
    ),
    ...pages.map((p) => n(`page:${p}`, 'page', 'list_page')),
  ]);
}

test('a removed page stays in its section of the shown menu next to the pages the proposal adds', () => {
  const d = navDoc([['main', ['overview', 'loans']]], ['overview', 'loans']);
  const p = navDoc([['main', ['overview', 'fines']]], ['overview', 'fines']);
  assert.deepEqual(menu(withRemoved(d, p)), [['overview', 'loans', 'fines']]);
});

test('only removed pages come back into a section; a page the proposal moves elsewhere does not', () => {
  const d = navDoc([['a', ['overview', 'loans']], ['b', ['members']]], ['overview', 'loans', 'members']);
  const p = navDoc([['a', ['overview']], ['b', ['loans']]], ['overview', 'loans']);
  assert.deepEqual(menu(withRemoved(d, p)), [['overview'], ['members', 'loans']]);
  assert.deepEqual(asObject(outlineMarks(d, p)), { 'page:members': 'removed' });
});
