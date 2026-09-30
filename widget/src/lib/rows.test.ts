import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { rowsDue, unanswered, viewsOf } from './rows.ts';

function node(path: string, layer: string, extra: Partial<OutlineNode>, children: OutlineNode[] = []): OutlineNode {
  return { path, layer, name: path, kind: layer, children, ...extra };
}

test('viewsOf: every view a node of the outline reads, draft views included, once each', () => {
  const root = node('/', 'root', {}, [
    node('page:a', 'page', {}, [
      node('page:a/section:chart', 'section', { view: 'draft.LoansPerMonth' }),
      node('page:a/section:list', 'section', { view: 'loans.All' }),
      node('page:a/section:again', 'section', { view: 'draft.LoansPerMonth' }),
    ]),
    node('shell:s/menu:m/nav_section:dyn', 'nav_section', { props: { from_view: 'shelves.All' } }),
  ]);
  assert.deepEqual([...viewsOf(root)].sort(), ['draft.LoansPerMonth', 'loans.All', 'shelves.All']);
  assert.deepEqual([...viewsOf(null)], []);
});

test('unanswered: a view asked at a revision no answer arrived at, or never answered at all', () => {
  const asked = new Map<string, number | null>([
    ['draft.InFlight', 31],
    ['draft.Answered', 31],
    ['loans.All', 1],
    ['draft.Never', 31],
  ]);
  const answered = new Map<string, number | null>([
    ['draft.InFlight', 30],
    ['draft.Answered', 31],
    ['loans.All', 1],
  ]);
  assert.deepEqual(unanswered(asked, answered).sort(), ['draft.InFlight', 'draft.Never']);
});

test('a view never asked for is due, draft or not', () => {
  assert.equal(rowsDue('draft.LoansPerMonth', false, undefined, 4), true);
  assert.equal(rowsDue('loans.All', false, undefined, 4), true);
});

test('a view asked for at this revision is not due while the answer is on its way', () => {
  assert.equal(rowsDue('draft.LoansPerMonth', false, 4, 4), false);
  assert.equal(rowsDue('loans.All', false, 4, 4), false);
});

test('a draft view answered at an older revision is due again; a fixture view is not', () => {
  assert.equal(rowsDue('draft.LoansPerMonth', true, 4, 5), true);
  assert.equal(rowsDue('loans.All', true, 4, 5), false);
});

test('a draft view answered at this revision is not due', () => {
  assert.equal(rowsDue('draft.LoansPerMonth', true, 5, 5), false);
});

test('a fixture view whose rows arrived without a request is not due', () => {
  assert.equal(rowsDue('loans.All', true, undefined, 5), false);
});
