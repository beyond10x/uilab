import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOperator as Operator, UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import {
  AGENT_COLOURS,
  HUMAN_COLOURS,
  applyChange,
  feedEntry,
  flashPath,
  initialName,
  localOperatorId,
  operatorColour,
  pruneInFlight,
  pushFeed,
  revisionStep,
  trackInFlight,
  type InFlight,
} from './collab.ts';
import { findNode, nearestExisting } from './outline.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

function doc(): OutlineNode {
  return n('/', 'root', 'document', [
    n('shell:app', 'shell', 'shell'),
    n('page:loans', 'page', 'list_page', [
      n('page:loans/section:list', 'section', 'collection', [n('page:loans/section:list/item:status', 'item', 'record')]),
      n('page:loans/overlay:edit', 'overlay', 'drawer form'),
    ]),
    n('page:members', 'page', 'list_page'),
  ]);
}

const paths = (node: OutlineNode | null | undefined) => node?.children.map((c) => c.path);

// ---- applyChange -------------------------------------------------------------------------------

test('Insert appends the node as the last child of the parent', () => {
  const before = doc();
  const node = n('page:loans/section:overdue', 'section', 'collection', [], { title: 'Overdue' });
  const after = applyChange(before, { op: 'Insert', changed: node.path, parent: 'page:loans', node });
  assert.ok(after);
  assert.deepEqual(paths(findNode(after, 'page:loans')), [
    'page:loans/section:list',
    'page:loans/overlay:edit',
    'page:loans/section:overdue',
  ]);
  assert.equal(findNode(after, 'page:loans/section:overdue')?.title, 'Overdue');
});

test('Insert under the root appends a page', () => {
  const node = n('page:overview', 'page', 'dashboard_page');
  const after = applyChange(doc(), { op: 'Insert', changed: 'page:overview', parent: '/', node });
  assert.deepEqual(paths(after), ['shell:app', 'page:loans', 'page:members', 'page:overview']);
});

test('Replace swaps the node in place, subtree included', () => {
  const node = n('page:loans/section:list', 'section', 'board', [n('page:loans/section:list/widget:a', 'widget', 'metric')], { title: 'Renamed' });
  const after = applyChange(doc(), { op: 'Replace', changed: node.path, parent: 'page:loans', node });
  assert.ok(after);
  assert.deepEqual(paths(findNode(after, 'page:loans')), ['page:loans/section:list', 'page:loans/overlay:edit']);
  const list = findNode(after, 'page:loans/section:list');
  assert.equal(list?.kind, 'board');
  assert.equal(list?.title, 'Renamed');
  assert.deepEqual(paths(list), ['page:loans/section:list/widget:a']);
  assert.equal(findNode(after, 'page:loans/section:list/item:status'), null);
});

test('Remove deletes the node and its subtree', () => {
  const after = applyChange(doc(), { op: 'Remove', changed: 'page:loans/section:list', parent: 'page:loans' });
  assert.ok(after);
  assert.deepEqual(paths(findNode(after, 'page:loans')), ['page:loans/overlay:edit']);
  assert.equal(findNode(after, 'page:loans/section:list/item:status'), null);
});

test('a change reaches nodes at depth and tolerates slashes around paths', () => {
  const after = applyChange(doc(), { op: 'Remove', changed: '/page:loans/section:list/item:status/', parent: 'page:loans/section:list/' });
  assert.deepEqual(paths(findNode(after!, 'page:loans/section:list')), []);
});

test('applyChange leaves its input untouched and shares branches it did not change', () => {
  const before = doc();
  const snapshot = structuredClone(before);
  const after = applyChange(before, { op: 'Remove', changed: 'page:loans/overlay:edit', parent: 'page:loans' })!;
  assert.deepEqual(before, snapshot);
  assert.notEqual(after, before);
  assert.notEqual(findNode(after, 'page:loans'), findNode(before, 'page:loans'));
  assert.equal(findNode(after, 'page:members'), findNode(before, 'page:members'));
  assert.equal(findNode(after, 'page:loans/section:list'), findNode(before, 'page:loans/section:list'));
});

test('a change that does not fit the outline returns null', () => {
  const d = doc();
  const node = n('page:loans/section:list', 'section', 'collection');
  // parent missing
  assert.equal(applyChange(d, { op: 'Insert', changed: 'page:nope/section:x', parent: 'page:nope', node }), null);
  // insert of a path already present
  assert.equal(applyChange(d, { op: 'Insert', changed: node.path, parent: 'page:loans', node }), null);
  // replace or remove of a missing node
  assert.equal(applyChange(d, { op: 'Replace', changed: 'page:loans/section:gone', parent: 'page:loans', node }), null);
  assert.equal(applyChange(d, { op: 'Remove', changed: 'page:loans/section:gone', parent: 'page:loans' }), null);
  // a node not directly under the named parent
  assert.equal(applyChange(d, { op: 'Remove', changed: 'page:loans/section:list/item:status', parent: 'page:loans' }), null);
  // insert or replace without a node
  assert.equal(applyChange(d, { op: 'Replace', changed: node.path, parent: 'page:loans' }), null);
  // the root cannot be removed
  assert.equal(applyChange(d, { op: 'Remove', changed: '/', parent: '/' }), null);
});

test('flashPath is the node, or its parent after a removal; nearestExisting walks up', () => {
  assert.equal(flashPath({ op: 'Insert', changed: 'page:loans/section:x', parent: 'page:loans' }), 'page:loans/section:x');
  assert.equal(flashPath({ op: 'Remove', changed: 'page:loans/section:x', parent: 'page:loans' }), 'page:loans');
  assert.equal(nearestExisting(doc(), 'page:loans/section:gone/item:x'), 'page:loans');
  assert.equal(nearestExisting(doc(), 'page:loans/section:list'), 'page:loans/section:list');
  assert.equal(nearestExisting(doc(), 'page:nope'), '/');
});

// ---- revisions ---------------------------------------------------------------------------------

test('revisionStep applies the next revision only', () => {
  assert.equal(revisionStep(4, 5), 'apply');
  assert.equal(revisionStep(0, 1), 'apply');
  assert.equal(revisionStep(4, 6), 'gap');
  assert.equal(revisionStep(4, 50), 'gap');
  assert.equal(revisionStep(4, 4), 'stale');
  assert.equal(revisionStep(4, 2), 'stale');
  assert.equal(revisionStep(null, 1), 'gap');
});

// ---- operators ---------------------------------------------------------------------------------

test('operator colours are stable per id and come from the palette of the kind', () => {
  const ids = ['op-1', 'op-2', 'claude-7f3a', 'a', '', 'ünïcode-id'];
  for (const id of ids) {
    assert.equal(operatorColour(id, 'human'), operatorColour(id, 'human'));
    assert.equal(operatorColour(id, 'agent'), operatorColour(id, 'agent'));
    assert.ok((HUMAN_COLOURS as readonly string[]).includes(operatorColour(id, 'human')));
    assert.ok((AGENT_COLOURS as readonly string[]).includes(operatorColour(id, 'agent')));
  }
  // Pinned values: a change of hash or palette shows up here, not as colours shifting between reloads.
  assert.equal(operatorColour('op-1', 'human'), operatorColour('op-1'));
  const four = ['op-1', 'op-2', 'op-3', 'op-4'];
  assert.deepEqual(four.map((id) => operatorColour(id, 'agent')), ['#a21caf', '#db2777', '#9333ea', '#9333ea']);
  assert.deepEqual(four.map((id) => operatorColour(id, 'human')), ['#0f766e', '#57534e', '#0891b2', '#57534e']);
  assert.ok(new Set(['op-1', 'op-2', 'op-3', 'op-4', 'op-5', 'op-6'].map((id) => operatorColour(id))).size > 1, 'ids spread over the palette');
  for (const c of AGENT_COLOURS) assert.ok(!(HUMAN_COLOURS as readonly string[]).includes(c), 'agent and human colours are disjoint');
});

test('initialName prefers the query, then the stored name, then the default', () => {
  assert.equal(initialName('?name=Ada', 'Grace'), 'Ada');
  assert.equal(initialName('?mock&name=%20Ada%20', null), 'Ada');
  assert.equal(initialName('?name=', 'Grace'), 'Grace');
  assert.equal(initialName('', '  '), 'Timo');
  assert.equal(initialName('', null), 'Timo');
});

const ops: Operator[] = [
  { id: 'h1', name: 'Timo', kind: 'human', last_seen_ms: 0 },
  { id: 'a1', name: 'Claude', kind: 'agent', last_seen_ms: 0 },
];

test('localOperatorId finds the one human of the local name', () => {
  assert.equal(localOperatorId(ops, 'Timo'), 'h1');
  assert.equal(localOperatorId(ops, 'Claude'), null, 'an agent of the name is not the local operator');
  assert.equal(localOperatorId([...ops, { id: 'h2', name: 'Timo', kind: 'human', last_seen_ms: 0 }], 'Timo'), null);
});

test('a move is a feed line by the operator that moved the selection, with its reason', () => {
  const value = { by: 'h1', selected_by: 'agent', from: 'page:loans/section:list', to: '/', reason: 'a new page goes under the root', utterance: 'u' };
  assert.deepEqual(feedEntry({ type: 'moved', value: { ...value, navigate_only: false } }, 5, 3), {
    seq: 3,
    at: 5,
    kind: 'moved',
    by: 'agent',
    what: 'retarget',
    path: '/',
    text: 'a new page goes under the root',
  });
  assert.equal(feedEntry({ type: 'moved', value: { ...value, navigate_only: true } }, 5, 4)?.what, 'navigate');
});

// ---- in flight ---------------------------------------------------------------------------------

test('a move carries the action to its new target, and a navigation-only move ends it', () => {
  const value = { by: 'h1', selected_by: 'agent', from: 'page:loans', to: 'page:members', reason: 'r', utterance: 'u' };
  const thinking = trackInFlight({}, { type: 'thinking', value: { by: 'h1', target: 'page:loans' } });
  assert.deepEqual(trackInFlight(thinking, { type: 'moved', value: { ...value, navigate_only: false } }), { h1: 'page:members' });
  assert.deepEqual(trackInFlight(thinking, { type: 'moved', value: { ...value, navigate_only: true } }), {});
});

test('thinking starts an action; that operator\'s proposal clears it', () => {
  let f: InFlight = {};
  f = trackInFlight(f, { type: 'thinking', value: { by: 'a1', target: 'page:loans' } });
  assert.deepEqual(f, { a1: 'page:loans' });
  f = trackInFlight(f, {
    type: 'proposal',
    value: { by: 'a1', proposal_id: 'p1', target: 'page:loans', changed: 'page:loans/section:x', op: 'Insert', utterance: '', before: '', after: '', findings: [], outline: doc() },
  });
  assert.deepEqual(f, {});
});

test('a goal message alone leaves actions in flight; whether it ended one takes the goal before it', () => {
  const thinking = trackInFlight({}, { type: 'thinking', value: { by: 'a1', target: 'page:loans' } });
  for (const state of ['planning', 'running', 'done', 'stopped', 'failed'] as const) {
    const after = trackInFlight(thinking, { type: 'goal', value: { goal_id: 'goal-1', by: 'a1', text: 't', state, steps: [] } });
    assert.equal(after, thinking, state);
  }
});

test('each outcome clears only its own operator', () => {
  let f: InFlight = {};
  f = trackInFlight(f, { type: 'thinking', value: { by: 'a1', target: 'page:loans' } });
  f = trackInFlight(f, { type: 'thinking', value: { by: 'h1', target: 'page:members' } });
  f = trackInFlight(f, { type: 'refused', value: { by: 'h1', check: 'layer.misplaced', message: '' } });
  assert.deepEqual(f, { a1: 'page:loans' });
  f = trackInFlight(f, { type: 'failed', value: { by: 'h1', message: '' } });
  assert.deepEqual(f, { a1: 'page:loans' });
  f = trackInFlight(f, { type: 'changed', value: { by: 'a1', revision: 3, op: 'Remove', changed: 'page:loans/section:list', parent: 'page:loans', findings: [] } });
  assert.deepEqual(f, {});
});

test('a later thinking moves the target; messages without by and other messages leave it', () => {
  let f: InFlight = trackInFlight({}, { type: 'thinking', value: { by: 'a1', target: 'page:loans' } });
  f = trackInFlight(f, { type: 'thinking', value: { by: 'a1', target: 'page:members' } });
  assert.deepEqual(f, { a1: 'page:members' });
  const same = trackInFlight(f, { type: 'failed', value: { message: 'no by' } });
  assert.equal(same, f);
  assert.equal(trackInFlight(f, { type: 'thinking', value: { target: 'page:loans' } }), f);
  assert.equal(trackInFlight(f, { type: 'transcript', value: { by: 'a1', text: 'x', audio_ms: 0, took_ms: 0 } }), f);
  assert.equal(trackInFlight(f, { type: 'presence', value: { operators: [] } }), f);
});

test('pruneInFlight drops operators that left', () => {
  const f: InFlight = { a1: 'page:loans', gone: 'page:members' };
  assert.deepEqual(pruneInFlight(f, ops), { a1: 'page:loans' });
  const kept: InFlight = { a1: 'page:loans' };
  assert.equal(pruneInFlight(kept, ops), kept);
});

// ---- feed --------------------------------------------------------------------------------------

test('feed entries carry operator, op or check and path; the feed keeps the newest 50', () => {
  const inFlight: InFlight = { a1: 'page:loans' };
  assert.deepEqual(feedEntry({ type: 'refused', value: { by: 'a1', check: 'layer.misplaced', message: 'no' } }, 10, 1, inFlight), {
    seq: 1,
    at: 10,
    kind: 'refused',
    by: 'a1',
    what: 'layer.misplaced',
    path: 'page:loans',
    text: 'no',
  });
  assert.deepEqual(
    feedEntry({ type: 'changed', value: { by: 'a1', revision: 2, op: 'Insert', changed: 'page:loans/section:x', parent: 'page:loans', findings: [] } }, 11, 2),
    { seq: 2, at: 11, kind: 'changed', by: 'a1', what: 'Insert', path: 'page:loans/section:x' },
  );
  assert.equal(feedEntry({ type: 'presence', value: { operators: [] } }, 0, 0), null);
  let feed = [] as ReturnType<typeof pushFeed>;
  for (let k = 0; k < 60; k++) feed = pushFeed(feed, { seq: k, at: k, kind: 'thinking' });
  assert.equal(feed.length, 50);
  assert.equal(feed[0].seq, 59);
  assert.equal(feed.at(-1)!.seq, 10);
});
