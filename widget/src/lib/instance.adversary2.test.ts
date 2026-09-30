import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { previewNode } from './components.ts';
import { itemScopes, rowNode } from './instance.ts';

// Adversary pass 2 on story:canvas-widget-instances: rowNode's copy of the substitution against
// the one in components.ts it was copied from.

const ROW = {
  id: 'm-1',
  name: 'Ada Lovelace',
  standing: 'good',
  count: 0,
  active: false,
  gone: null,
  shelf: { name: 'Shelf A', floor: 2 },
  tags: ['x', 'y'],
};

function item(props: Record<string, unknown>, title?: string): OutlineNode {
  const node: OutlineNode = { path: 'page:p/section:l/item:t', layer: 'item', name: 't', kind: 'text', children: [], props } as OutlineNode;
  if (title !== undefined) node.title = title;
  return node;
}

/** `value` with every whole-word `row` reference rewritten as `args.r`, so the Components tab's
 *  substitution reads the same row through a param. */
function asArgs(value: unknown): unknown {
  if (typeof value === 'string') return value.replace(/(?<![\w.])row((?:\.[A-Za-z_]\w*)+)/g, 'args.r$1').replace(/^row$/, 'args.r');
  if (Array.isArray(value)) return value.map(asArgs);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, asArgs(v)]));
  return value;
}

const WRITTEN: unknown[] = [
  'row',
  'row.name',
  'row.count',
  'row.active',
  'row.gone',
  'row.gone.name',
  'row.shelf',
  'row.shelf.floor',
  'row.shelf.missing',
  'row.tags',
  'row.missing',
  'row.missing.deeper',
  'Shelf: row.shelf (floor row.shelf.floor)',
  'row.count copies, active row.active, gone [row.gone]',
  'borrow.name arrow.name row_name rows.first',
  'row.name.',
  '(row.name)',
  7,
  true,
  null,
  ['row.name', ['row.standing', { deep: 'row.shelf.name' }]],
  { 'row.name': 'row.name', nested: { at: 'row.standing' } },
];

test('rowNode reads row.<field> exactly as previewNode reads args.<param>.<field>, value by value', () => {
  for (const written of WRITTEN) {
    const byRow = rowNode(item({ text: written }), ROW).props;
    const byArgs = previewNode(item({ text: asArgs(written) }), { r: ROW }).props;
    assert.deepEqual(byRow, byArgs, JSON.stringify(written));
  }
});

test('rowNode fills the title the way previewNode does, and keeps path, name, kind and layer', () => {
  for (const written of ['row', 'row.shelf', 'row.count', 'row.gone', 'Due: row.name']) {
    const byRow = rowNode(item({}, written), ROW);
    const byArgs = previewNode(item({}, asArgs(written) as string), { r: ROW });
    assert.equal(byRow.title, byArgs.title, written);
    assert.deepEqual([byRow.path, byRow.name, byRow.kind, byRow.layer], ['page:p/section:l/item:t', 't', 'text', 'item']);
  }
});

test('rowNode leaves the node it was given untouched, so the next row reads its own values', () => {
  const node = item({ text: 'row.name', meta: { at: 'row.standing' } }, 'row.name');
  const first = rowNode(node, ROW);
  const second = rowNode(node, { name: 'Grace Hopper', standing: 'late' });
  assert.deepEqual(node.props, { text: 'row.name', meta: { at: 'row.standing' } });
  assert.equal(node.title, 'row.name');
  assert.deepEqual([first.title, second.title], ['Ada Lovelace', 'Grace Hopper']);
  assert.deepEqual(second.props, { text: 'Grace Hopper', meta: { at: 'late' } });
});

test('itemScopes: a record over one row, a collection over one row, and any other kind per row', () => {
  const one = [ROW];
  assert.deepEqual(itemScopes('record', one), [{ row: ROW, rows: one }]);
  assert.deepEqual(itemScopes('collection', one), [{ row: ROW, rows: one }]);
  assert.equal(itemScopes('record', [ROW, ROW, ROW]).length, 1);
});
