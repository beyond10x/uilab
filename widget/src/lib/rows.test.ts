import { test } from 'node:test';
import assert from 'node:assert/strict';
import { rowsDue } from './rows.ts';

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
