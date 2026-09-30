import { test } from 'node:test';
import assert from 'node:assert/strict';
import { targetIn, workspaceOf } from './workspace.ts';
import { goalMessage } from './goal.ts';

test('only the Components tab is a workspace of its own', () => {
  assert.equal(workspaceOf('components'), 'components');
  for (const view of ['ui', 'yaml', 'docs']) assert.equal(workspaceOf(view), undefined);
});

// Found on 2026-09-30: "put some basic set of components" on the Components tab went to
// `page:overview`, the page on screen in the canvas, and became three sections of it.
test('on the Components tab a page selection gives way to the root', () => {
  assert.deepEqual(targetIn('components', 'page:overview', 'page:overview'), { target: '/' });
  assert.deepEqual(targetIn('components', '/', 'page:overview'), { target: '/' });
  assert.deepEqual(targetIn('components', undefined, 'page:overview'), { target: '/' });
});

test('on the Components tab a selected widget or body node stays the target', () => {
  assert.deepEqual(targetIn('components', 'component:loan_card', undefined), { target: 'component:loan_card' });
  assert.deepEqual(targetIn('components', 'component:loan_card/node:title', 'page:overview'), {
    target: 'component:loan_card/node:title',
  });
});

test('in the canvas the root gives way to the page on screen, which is selected', () => {
  assert.deepEqual(targetIn('ui', '/', 'page:overview'), { target: 'page:overview', selectPage: 'page:overview' });
  assert.deepEqual(targetIn('ui', 'page:loans/section:list', 'page:overview'), { target: 'page:loans/section:list' });
  assert.deepEqual(targetIn('ui', '/', undefined), { target: '/' });
});

test('a goal carries the workspace it was given in', () => {
  assert.deepEqual(goalMessage('build a component set', '/', 'components'), {
    type: 'goal',
    value: { text: 'build a component set', target: '/', workspace: 'components' },
  });
  assert.deepEqual(goalMessage('build it', 'page:members', undefined), {
    type: 'goal',
    value: { text: 'build it', target: 'page:members' },
  });
});
