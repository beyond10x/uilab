import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import {
  columnsOf,
  fieldsOf,
  findNode,
  homePage,
  isDraftView,
  lineage,
  navLayout,
  normalize,
  overlayOf,
  pageOf,
  pagesOf,
  parentPath,
  shellOf,
} from './outline.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

const doc = n('/', 'root', 'document', [
  n('shell:app', 'shell', 'shell', [n('shell:app/region:nav', 'region', 'navigation')]),
  n('nav', 'nav', 'navigation', [n('nav/nav_section:people', 'nav_section', 'nav_section')]),
  n('page:loan', 'page', 'form_page'),
  n('page:loans', 'page', 'list_page', [
    n('page:loans/section:list', 'section', 'collection', [n('page:loans/section:list/item:status', 'item', 'record')], {
      view: 'loans.All',
      props: { columns: [{ field: 'title' }, { field: 'state', as: 'tag' }, 'due'] },
    }),
    n('page:loans/overlay:edit', 'overlay', 'drawer form', [], { props: { fields: ['due', { field: 'note', label: 'Note' }] } }),
  ]),
]);

test('findNode resolves every layer by full path', () => {
  assert.equal(findNode(doc, '/')?.layer, 'root');
  assert.equal(findNode(doc, '')?.layer, 'root');
  assert.equal(findNode(doc, 'nav')?.kind, 'navigation');
  assert.equal(findNode(doc, 'nav/nav_section:people')?.name, 'people');
  assert.equal(findNode(doc, 'shell:app/region:nav')?.kind, 'navigation');
  assert.equal(findNode(doc, 'page:loans/section:list/item:status')?.kind, 'record');
  assert.equal(findNode(doc, '/page:loans/overlay:edit/')?.kind, 'drawer form');
});

test('findNode does not confuse a name that prefixes another', () => {
  assert.equal(findNode(doc, 'page:loan')?.kind, 'form_page');
  assert.equal(findNode(doc, 'page:loans')?.kind, 'list_page');
  assert.equal(findNode(doc, 'page:lo'), null);
  assert.equal(findNode(doc, 'page:loans/section:missing'), null);
});

test('lineage runs from the root to the node', () => {
  assert.deepEqual(lineage(doc, 'page:loans/section:list/item:status')?.map((x) => x.path), [
    '/',
    'page:loans',
    'page:loans/section:list',
    'page:loans/section:list/item:status',
  ]);
  assert.equal(lineage(doc, 'page:nope'), null);
});

test('pageOf, overlayOf and parentPath read the path alone', () => {
  assert.equal(pageOf('page:loans/section:list/item:status'), 'page:loans');
  assert.equal(pageOf('page:loans'), 'page:loans');
  assert.equal(pageOf('/page:loans/'), 'page:loans');
  assert.equal(pageOf('nav/nav_section:people'), null);
  assert.equal(pageOf('/'), null);
  assert.equal(overlayOf('page:loans/overlay:edit/widget:x'), 'page:loans/overlay:edit');
  assert.equal(overlayOf('shell:app/overlay:help'), 'shell:app/overlay:help');
  assert.equal(overlayOf('page:loans/section:list'), null);
  assert.equal(parentPath('page:loans/section:list'), 'page:loans');
  assert.equal(parentPath('page:loans'), '/');
  assert.equal(parentPath('/'), null);
  assert.equal(normalize(' /nav/ '), 'nav');
});

function placedDoc(): OutlineNode {
  return n('/', 'root', 'document', [
    n('shell:app', 'shell', 'shell'),
    n('shell:print', 'shell', 'shell'),
    n('nav', 'nav', 'navigation', [
      n('nav/nav_section:circ', 'nav_section', 'nav_section', [], { props: { pages: ['loans', 'overview', 'ghost'] } }),
      n('nav/nav_section:people', 'nav_section', 'nav_section', [], { props: { from_view: 'members.All', page: 'member' } }),
    ], { props: { home: 'overview' } }),
    n('page:loans', 'page', 'list_page', [], { props: { shell: 'app' } }),
    n('page:overview', 'page', 'dashboard_page', [], { props: { shell: 'app' } }),
    n('page:member', 'page', 'record_page', [], { props: { shell: 'print' } }),
    n('page:hidden', 'page', 'form_page'),
  ]);
}

test('navLayout groups pages under their sections and hides unplaced pages', () => {
  const groups = navLayout(placedDoc());
  assert.deepEqual(
    groups.map((g) => [g.section?.name, g.entries.map((e) => `${e.page.name}${e.fromView ? `<${e.fromView}` : ''}`)]),
    [
      ['circ', ['loans', 'overview']],
      ['people', ['member<members.All']],
    ],
  );
});

test('navLayout without placement lists every page after the headings', () => {
  const groups = navLayout(doc);
  assert.deepEqual(
    groups.map((g) => [g.section?.name ?? null, g.entries.map((e) => e.page.name)]),
    [
      ['people', []],
      [null, ['loan', 'loans']],
    ],
  );
});

test('homePage and shellOf read props, with fallbacks', () => {
  const placed = placedDoc();
  assert.equal(homePage(placed)?.name, 'overview');
  assert.equal(homePage(doc)?.name, 'loan');
  assert.equal(shellOf(placed, findNode(placed, 'page:member'))?.name, 'print');
  assert.equal(shellOf(placed, findNode(placed, 'page:hidden'))?.name, 'app');
  assert.equal(shellOf(placed, null)?.name, 'app');
});

test('pages, columns, fields and draft views', () => {
  assert.deepEqual(pagesOf(doc).map((p) => p.name), ['loan', 'loans']);
  assert.deepEqual(columnsOf(findNode(doc, 'page:loans/section:list')!), [
    { field: 'title', as: undefined },
    { field: 'state', as: 'tag' },
    { field: 'due' },
  ]);
  assert.deepEqual(fieldsOf(findNode(doc, 'page:loans/overlay:edit')!), [
    { field: 'due', label: 'due' },
    { field: 'note', label: 'Note' },
  ]);
  assert.equal(isDraftView('draft.loans.Overdue'), true);
  assert.equal(isDraftView('loans.All'), false);
  assert.equal(isDraftView(undefined), false);
});
