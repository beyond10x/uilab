import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { EssJsonValue, UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { markClasses, outlineMarks, withRemoved } from './marks.ts';

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
