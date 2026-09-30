import { test } from 'node:test';
import assert from 'node:assert/strict';
import { diffLines, hunks } from './diff.ts';

const render = (text: string[]) => text.join('\n') + '\n';

test('identical texts have no changes and no hunks', () => {
  const d = diffLines('a\nb\n', 'a\nb\n');
  assert.deepEqual(d.map((l) => l.op), [' ', ' ']);
  assert.deepEqual(hunks(d), []);
});

test('an inserted block shows as added lines between kept ones', () => {
  const d = diffLines(render(['sections:', '  list:', '    component: collection']), render(['sections:', '  list:', '    component: collection', '  overdue:', '    component: collection']));
  assert.deepEqual(
    d.map((l) => l.op + l.text),
    [' sections:', '   list:', '     component: collection', '+  overdue:', '+    component: collection'],
  );
  assert.deepEqual(d.at(-1), { op: '+', text: '    component: collection', b: 5 });
});

test('a replaced line is a removal followed by an addition', () => {
  const d = diffLines('title: Loans\nkind: list_page\n', 'title: All loans\nkind: list_page\n');
  assert.deepEqual(d.map((l) => l.op + l.text), ['-title: Loans', '+title: All loans', ' kind: list_page']);
});

test('empty before is all additions; empty after is all removals', () => {
  assert.deepEqual(diffLines('', 'a\nb').map((l) => l.op), ['+', '+']);
  assert.deepEqual(diffLines('a\nb', '').map((l) => l.op), ['-', '-']);
});

test('the diff reproduces both texts', () => {
  const before = 'a\nb\nc\nd\ne\nf\n';
  const after = 'a\nc\nd\nx\ne\ny\n';
  const d = diffLines(before, after);
  assert.equal(render(d.filter((l) => l.op !== '+').map((l) => l.text)), before);
  assert.equal(render(d.filter((l) => l.op !== '-').map((l) => l.text)), after);
});

test('hunks keep three lines of context and merge when close', () => {
  const before = Array.from({ length: 20 }, (_, k) => `line ${k + 1}`);
  const after = [...before];
  after[1] = 'changed 2';
  after[17] = 'changed 18';
  const h = hunks(diffLines(render(before), render(after)));
  assert.equal(h.length, 2);
  assert.equal(h[0].header, '@@ -1,5 +1,5 @@');
  assert.equal(h[1].header, '@@ -15,6 +15,6 @@');
  const apart = [...before];
  apart[1] = 'changed 2';
  apart[9] = 'changed 10';
  assert.equal(hunks(diffLines(render(before), render(apart))).length, 2);
  const close = [...before];
  close[1] = 'changed 2';
  close[8] = 'changed 9';
  assert.equal(hunks(diffLines(render(before), render(close))).length, 1);
});
