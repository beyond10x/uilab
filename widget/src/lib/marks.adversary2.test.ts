import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { EssJsonValue, UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { outlineMarks, withRemoved } from './marks.ts';
import { navLayout } from './outline.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

function asObject(m: Map<string, string>): Record<string, string> {
  return Object.fromEntries([...m.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

/** A menu section as the server outlines it: a fixed list as `props.pages`, a dynamic one as
 *  `props.from_view` + `props.page`; its label is the node's title. */
type Listing = string[] | { from_view: string; page: string };

function section(name: string, listing: Listing, title?: string): OutlineNode {
  const props: { [key: string]: EssJsonValue } = Array.isArray(listing) ? { pages: listing } : { ...listing };
  return n(`nav/nav_section:${name}`, 'nav_section', 'nav_section', [], { props, ...(title ? { title } : {}) });
}

function page(name: string, children: OutlineNode[] = []): OutlineNode {
  return n(`page:${name}`, 'page', 'list_page', children, { props: { shell: 'app' } });
}

function doc(sections: OutlineNode[], pages: OutlineNode[]): OutlineNode {
  return n('/', 'root', 'document', [n('nav', 'nav', 'navigation', sections, { props: { home: 'overview' } }), ...pages]);
}

/** The shown menu: each section's name with the page names it lists, in order. */
function menu(root: OutlineNode): [string, string[]][] {
  return navLayout(root).map((g) => [g.section?.name ?? '', g.entries.map((e) => e.page.name)]);
}

const pages = (...names: string[]) => names.map((p) => page(p));

/**
 * Acceptance: "in both with different props ... is changed". A menu section's `pages` is the
 * section's own authored list (`NavSection.pages`, replaced whole by a nav_section Replace,
 * `crates/uilab-doc/src/patch.rs:489`); only a page Remove or a page Insert touches it from
 * elsewhere. DERIVED_PROPS drops the key outright, so a proposal whose whole effect is to move a
 * page from one menu section to another marks nothing at all.
 */
test('a Batch that moves an existing page into another menu section marks both sections changed', () => {
  const before = doc([section('circulation', ['overview', 'loans']), section('people', ['members'])], pages('overview', 'loans', 'members'));
  const after = doc([section('circulation', ['overview']), section('people', ['members', 'loans'])], pages('overview', 'loans', 'members'));
  assert.deepEqual(asObject(outlineMarks(before, after)), {
    'nav/nav_section:circulation': 'changed',
    'nav/nav_section:people': 'changed',
  });
});

test('a Replace that reorders a menu section marks the section changed', () => {
  const before = doc([section('circulation', ['overview', 'loans', 'members'])], pages('overview', 'loans', 'members'));
  const after = doc([section('circulation', ['members', 'overview', 'loans'])], pages('overview', 'loans', 'members'));
  assert.deepEqual(asObject(outlineMarks(before, after)), { 'nav/nav_section:circulation': 'changed' });
});

test('a section relabelled by the same Batch that removes one of its pages is still changed', () => {
  const before = doc([section('circulation', ['overview', 'loans', 'members'], 'Circulation')], pages('overview', 'loans', 'members'));
  const after = doc([section('circulation', ['overview', 'members'], 'Lending')], pages('overview', 'members'));
  assert.deepEqual(asObject(outlineMarks(before, after)), { 'nav/nav_section:circulation': 'changed', 'page:loans': 'removed' });
  assert.deepEqual(menu(withRemoved(before, after)), [['circulation', ['overview', 'loans', 'members']]]);
});

test('a Batch removing two pages from one section keeps them where they were, wherever they sat', () => {
  const all = ['a', 'b', 'c', 'd', 'e'];
  const before = doc([section('main', all)], pages(...all));
  for (const gone of [['b', 'c'], ['a', 'b'], ['d', 'e'], ['a', 'e'], ['c', 'b']]) {
    const left = all.filter((p) => !gone.includes(p));
    const after = doc([section('main', left)], pages(...left));
    assert.deepEqual(menu(withRemoved(before, after)), [['main', all]], `removing ${gone.join(', ')}`);
    assert.deepEqual(asObject(outlineMarks(before, after)), Object.fromEntries([...gone].sort().map((p) => [`page:${p}`, 'removed'])));
  }
});

test('a page removed and inserted again under another section is neither removed nor listed twice', () => {
  const before = doc([section('circulation', ['overview', 'loans']), section('people', ['members'])], pages('overview', 'loans', 'members'));
  const moved = page('loans', [n('page:loans/section:list', 'section', 'collection', [], { view: 'loans.All' })]);
  const after = doc([section('circulation', ['overview']), section('people', ['members', 'loans'])], [page('overview'), page('members'), moved]);
  const marks = asObject(outlineMarks(before, after));
  assert.equal(marks['page:loans'], undefined);
  assert.equal(marks['page:loans/section:list'], 'added');
  assert.deepEqual(menu(withRemoved(before, after)), [
    ['circulation', ['overview']],
    ['people', ['members', 'loans']],
  ]);
});

test('a dynamic menu section is left as it is when a page listed elsewhere is removed', () => {
  const dynamic = section('members', { from_view: 'members.All', page: 'member' });
  const before = doc([section('circulation', ['overview', 'loans']), dynamic], pages('overview', 'loans', 'member'));
  const after = doc([section('circulation', ['overview']), dynamic], pages('overview', 'member'));
  assert.deepEqual(asObject(outlineMarks(before, after)), { 'page:loans': 'removed' });
  assert.deepEqual(menu(withRemoved(before, after)), [
    ['circulation', ['overview', 'loans']],
    ['members', ['member']],
  ]);
  const retargeted = doc([section('circulation', ['overview', 'loans']), section('members', { from_view: 'members.Active', page: 'member' })], pages('overview', 'loans', 'member'));
  assert.deepEqual(asObject(outlineMarks(before, retargeted)), { 'nav/nav_section:members': 'changed' });
});
