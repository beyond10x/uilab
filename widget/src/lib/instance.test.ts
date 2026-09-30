import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { fixtureRowOf } from './components.ts';
import { bindArg, compositeKind, drawsAsPrimitive, instanceArgs, instanceBody, itemScopes, rowNode, widgetOfInstance } from './instance.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

const MEMBERS = [
  { name: 'Ada Lovelace', standing: 'good', joined: '2024-01-02' },
  { name: 'Grace Hopper', standing: 'late', joined: '2023-05-06' },
];

const LOANS = [
  { title: 'Dune', member: 'Ada Lovelace', due: '2026-10-01', state: 'open' },
  { title: 'Emma', member: 'Grace Hopper', due: '2026-09-01', state: 'late' },
];

/** The library outline with the eval document's widgets and instances of them on a page: a
 *  section, a board widget, a collection item and a drawer. */
function doc(): OutlineNode {
  return n('/', 'root', 'document', [
    n(
      'component:member_card',
      'component',
      'widget',
      [
        n('component:member_card/node:name', 'node', 'text', [], { props: { text: 'args.member.name', style: 'heading' } }),
        n('component:member_card/node:standing', 'node', 'badge', [], { props: { text: 'args.member.standing' } }),
      ],
      { title: 'A member', props: { params: { member: { type: 'Member', required: true } }, arrange: 'column', uses: [] } },
    ),
    n(
      'component:loan_card',
      'component',
      'widget',
      [
        n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title', style: 'heading' } }),
        n('component:loan_card/node:state', 'node', 'badge', [], { props: { text: 'args.loan.state' } }),
      ],
      { title: 'A loan', props: { params: { loan: { type: 'Loan', required: true } }, arrange: 'row', uses: [] } },
    ),
    n(
      'component:page_intro',
      'component',
      'widget',
      [
        n('component:page_intro/node:title', 'node', 'text', [], { props: { text: 'args.title', style: 'heading' } }),
        n('component:page_intro/node:subtitle', 'node', 'text', [], { props: { text: 'args.subtitle' } }),
      ],
      {
        title: 'Intro',
        props: { params: { title: { type: 'string', required: true }, subtitle: { type: 'string', default: '' } }, uses: [] },
      },
    ),
    n('component:badge', 'component', 'widget', [n('component:badge/node:tag', 'node', 'badge', [], { props: { text: 'args.label' } })], {
      props: { params: { label: { type: 'string', required: true } }, uses: [] },
    }),
    n('page:members', 'page', 'list_page', [
      n('page:members/section:spotlight', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } } }),
      n('page:members/section:intro', 'section', 'page_intro', [], { props: { args: { title: 'Members' } } }),
      n('page:members/section:list', 'section', 'collection', [
        n('page:members/section:list/item:card', 'item', 'member_card', [], { props: { args: { member: 'row' } } }),
        n('page:members/section:list/item:tag', 'item', 'badge', [], { props: { text: 'row.standing' } }),
      ], { view: 'members.All' }),
      n('page:members/overlay:quick', 'overlay', 'drawer loan_card', [], { props: { args: { loan: 'rows.first' } } }),
    ]),
  ]);
}

function at(root: OutlineNode, path: string): OutlineNode {
  const walk = (node: OutlineNode): OutlineNode | null => (node.path === path ? node : node.children.map(walk).find(Boolean) ?? null);
  const found = walk(root);
  assert.ok(found, `no node ${path}`);
  return found;
}

const rowOf = fixtureRowOf(['members.All', 'loans.All'], { 'members.All': { rows: MEMBERS }, 'loans.All': { rows: LOANS } });

test('bindArg: row binds the item row, row.<field> a field of it', () => {
  assert.deepEqual(bindArg('row', { row: MEMBERS[1] }), { bound: true, value: MEMBERS[1] });
  assert.deepEqual(bindArg('row.standing', { row: MEMBERS[1] }), { bound: true, value: 'late' });
});

test('bindArg: rows.first and rows.<n> bind rows of the composite, rows the whole list', () => {
  assert.deepEqual(bindArg('rows.first', { rows: MEMBERS }), { bound: true, value: MEMBERS[0] });
  assert.deepEqual(bindArg('rows.1', { rows: MEMBERS }), { bound: true, value: MEMBERS[1] });
  assert.deepEqual(bindArg('rows.first.name', { rows: MEMBERS }), { bound: true, value: 'Ada Lovelace' });
  assert.deepEqual(bindArg('rows', { rows: MEMBERS }), { bound: true, value: MEMBERS });
});

test('bindArg: a literal binds as written', () => {
  assert.deepEqual(bindArg('Showcase', {}), { bound: true, value: 'Showcase' });
  assert.deepEqual(bindArg(3, {}), { bound: true, value: 3 });
  assert.deepEqual(bindArg({ name: 'x' }, {}), { bound: true, value: { name: 'x' } });
  assert.deepEqual(bindArg('rowdy', { row: MEMBERS[0] }), { bound: true, value: 'rowdy' });
});

test('bindArg: a row reference with nothing to read is unbound', () => {
  assert.deepEqual(bindArg('row', {}), { bound: false });
  assert.deepEqual(bindArg('row.name', {}), { bound: false });
  assert.deepEqual(bindArg('rows.first', {}), { bound: false });
  assert.deepEqual(bindArg('rows.first', { rows: [] }), { bound: false });
  assert.deepEqual(bindArg('rows.9', { rows: MEMBERS }), { bound: false });
});

test('widgetOfInstance: a section, an item and an overlay naming a widget are instances', () => {
  const root = doc();
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:spotlight'))?.name, 'member_card');
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:list/item:card'))?.name, 'member_card');
  assert.equal(widgetOfInstance(root, at(root, 'page:members/overlay:quick'))?.name, 'loan_card');
  assert.equal(compositeKind(at(root, 'page:members/overlay:quick')), 'loan_card');
});

test('widgetOfInstance: a built-in composite and a primitive named like a widget are not instances', () => {
  const root = doc();
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:list')), null);
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:list/item:tag')), null);
  const instance = n('page:members/section:list/item:tag2', 'item', 'badge', [], { props: { args: { label: 'row.standing' } } });
  assert.equal(widgetOfInstance(root, instance)?.name, 'badge');
});

test('drawsAsPrimitive: a primitive item or body node, not an instance of a widget named like one', () => {
  const root = doc();
  assert.equal(drawsAsPrimitive(at(root, 'page:members/section:list/item:tag')), true);
  assert.equal(drawsAsPrimitive(at(root, 'component:member_card/node:name')), true);
  assert.equal(drawsAsPrimitive(at(root, 'page:members/section:list/item:card')), false);
  assert.equal(drawsAsPrimitive(n('page:members/section:list/item:t', 'item', 'badge', [], { props: { args: { label: 'row' } } })), false);
});

test('widgetOfInstance: an instance of a widget it sits in is not expanded again', () => {
  const root = doc();
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:spotlight'), ['member_card']), null);
  assert.equal(widgetOfInstance(root, at(root, 'page:members/section:spotlight'), ['loan_card'])?.name, 'member_card');
});

test('instanceArgs: rows.first binds the first row, an unbound param takes the sample args', () => {
  const root = doc();
  const spotlight = at(root, 'page:members/section:spotlight');
  const widget = widgetOfInstance(root, spotlight)!;
  assert.deepEqual(instanceArgs(widget, spotlight, { rows: [MEMBERS[1]] }, rowOf), { member: MEMBERS[1] });
  assert.deepEqual(instanceArgs(widget, spotlight, {}, rowOf), { member: MEMBERS[0] });
  const intro = at(root, 'page:members/section:intro');
  assert.deepEqual(instanceArgs(widgetOfInstance(root, intro)!, intro, {}, rowOf), { title: 'Members', subtitle: '' });
});

test('instanceBody: the body read from the bound args, one per row of a collection item', () => {
  const root = doc();
  const card = at(root, 'page:members/section:list/item:card');
  const widget = widgetOfInstance(root, card)!;
  const texts = itemScopes('collection', MEMBERS).map((scope) =>
    instanceBody(widget, card, scope, rowOf).map((b) => (b.props as Record<string, unknown>).text),
  );
  assert.deepEqual(texts, [
    ['Ada Lovelace', 'good'],
    ['Grace Hopper', 'late'],
  ]);
  const body = instanceBody(widget, card, { row: MEMBERS[1] }, rowOf);
  assert.deepEqual(body.map((b) => b.path), ['component:member_card/node:name', 'component:member_card/node:standing']);
  assert.deepEqual(body[0].props, { text: 'Grace Hopper', style: 'heading' });
});

test('instanceBody: a drawer instance reads rows.first from its own rows', () => {
  const root = doc();
  const quick = at(root, 'page:members/overlay:quick');
  const body = instanceBody(widgetOfInstance(root, quick)!, quick, { rows: LOANS }, rowOf);
  assert.deepEqual(body.map((b) => (b.props as Record<string, unknown>).text), ['Dune', 'open']);
});

test('itemScopes: a collection per row, a record its first row, none while there are no rows', () => {
  assert.deepEqual(itemScopes('collection', MEMBERS), [{ row: MEMBERS[0], rows: MEMBERS }, { row: MEMBERS[1], rows: MEMBERS }]);
  assert.deepEqual(itemScopes('record', MEMBERS), [{ row: MEMBERS[0], rows: MEMBERS }]);
  assert.deepEqual(itemScopes('collection', []), []);
  assert.deepEqual(itemScopes('record', []), []);
});

test('bindArg: args and args.<param> have no holder on a page and are unbound', () => {
  for (const written of ['args', 'args.member', 'args.member.name']) {
    assert.deepEqual(bindArg(written, { row: MEMBERS[0], rows: MEMBERS }), { bound: false }, written);
  }
  assert.deepEqual(bindArg('argsy', {}), { bound: true, value: 'argsy' });
  assert.deepEqual(bindArg('the args.member', {}), { bound: true, value: 'the args.member' });
});

test('rowNode: a primitive item reads row and row.<field> from its row, as a body reads args', () => {
  const root = doc();
  const tag = at(root, 'page:members/section:list/item:tag');
  assert.deepEqual(rowNode(tag, MEMBERS[1]).props, { text: 'late' });
  const item = n('page:members/section:list/item:t', 'item', 'text', [], {
    title: 'row.name',
    props: { text: 'Standing: row.standing (row.name)', label: 'row', meta: { at: 'row.joined', n: 3, list: ['row.name', 'rows.first', 'arrow.name'] } },
  });
  const drawn = rowNode(item, MEMBERS[0]);
  assert.equal(drawn.title, 'Ada Lovelace');
  assert.deepEqual(drawn.props, {
    text: 'Standing: good (Ada Lovelace)',
    label: MEMBERS[0],
    meta: { at: '2024-01-02', n: 3, list: ['Ada Lovelace', 'rows.first', 'arrow.name'] },
  });
  assert.equal(drawn.path, item.path);
});

test('rowNode: a field the row does not carry reads as its own name, never the prototype', () => {
  const item = n('page:members/section:list/item:t', 'item', 'badge', [], { props: { text: 'row.shelf', label: 'on row.constructor' } });
  assert.deepEqual(rowNode(item, MEMBERS[0]).props, { text: 'shelf', label: 'on constructor' });
});
