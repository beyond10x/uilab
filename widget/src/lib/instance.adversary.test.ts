import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { fixtureRowOf } from './components.ts';
import { bindArg, instanceArgs, instanceBody, widgetOfInstance } from './instance.ts';

// Adversary cases for story:canvas-widget-instances: arg binding at its edges, and nesting.

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

const MEMBERS = [
  { name: 'Ada Lovelace', standing: 'good' },
  { name: 'Grace Hopper', standing: 'late' },
];

const rowOf = fixtureRowOf(['members.All'], { 'members.All': { rows: MEMBERS } });

function memberCard(): OutlineNode {
  return n(
    'component:member_card',
    'component',
    'widget',
    [n('component:member_card/node:name', 'node', 'text', [], { props: { text: 'args.member.name' } })],
    { props: { params: { member: { type: 'Member', required: true } }, uses: [] } },
  );
}

test('bindArg: rows.<n> that is not a plain index from 0 is unbound', () => {
  for (const pick of ['-1', '1.5', '1e0', 'last', 'length', '', '99999999999999999999', 'Infinity', 'NaN']) {
    assert.deepEqual(bindArg(`rows.${pick}`, { rows: MEMBERS }), { bound: false }, `rows.${pick}`);
  }
  assert.deepEqual(bindArg('rows.0', { rows: MEMBERS }), { bound: true, value: MEMBERS[0] });
});

test('bindArg: row and rows read only own fields, never the prototype', () => {
  assert.deepEqual(bindArg('row.constructor', { row: MEMBERS[0] }), { bound: false });
  assert.deepEqual(bindArg('row.__proto__', { row: MEMBERS[0] }), { bound: false });
  assert.deepEqual(bindArg('rows.first.toString', { rows: MEMBERS }), { bound: false });
});

// The server's widget checks read `row`, `rows` and `args` (and any path under one) as references
// resolved at run time, never as names (`is_reference`, crates/uilab-doc/src/check.rs). A page is no
// widget body, so an `args.<param>` written on a page instance has no holder to pass it through:
// it binds nothing, and the param takes its sample like any other unbound one.
test('instanceArgs: args.<param> written on a page instance is unbound, not the literal text', () => {
  const root = n('/', 'root', 'document', [
    memberCard(),
    n('page:members', 'page', 'list_page', [
      n('page:members/section:spot', 'section', 'member_card', [], { props: { args: { member: 'args.member' } } }),
    ]),
  ]);
  const spot = root.children[1].children[0];
  const widget = widgetOfInstance(root, spot)!;
  assert.deepEqual(instanceArgs(widget, spot, {}, rowOf), { member: MEMBERS[0] });
  assert.deepEqual(
    instanceBody(widget, spot, {}, rowOf).map((b) => (b.props as Record<string, unknown>).text),
    ['Ada Lovelace'],
  );
});

test('widgetOfInstance: mutual recursion across two widgets stops at the widget already drawn', () => {
  const root = n('/', 'root', 'document', [
    n('component:a', 'component', 'widget', [n('component:a/node:b', 'node', 'b', [], { props: { args: {} } })], { props: { uses: [] } }),
    n('component:b', 'component', 'widget', [n('component:b/node:a', 'node', 'a', [], { props: { args: {} } })], { props: { uses: [] } }),
  ]);
  const aInB = root.children[1].children[0];
  const bInA = root.children[0].children[0];
  assert.equal(widgetOfInstance(root, bInA, ['a'])?.name, 'b');
  assert.equal(widgetOfInstance(root, aInB, ['a', 'b']), null);
  assert.equal(widgetOfInstance(root, aInB, ['b'])?.name, 'a');
});

test('widgetOfInstance: an instance of a widget no longer declared is no instance', () => {
  const root = n('/', 'root', 'document', [n('page:p', 'page', 'list_page', [])]);
  const gone = n('page:p/section:s', 'section', 'member_card', [], { props: { args: { member: 'rows.first' } } });
  assert.equal(widgetOfInstance(root, gone), null);
});
