import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { findingsBadge, phaseLabel, presenceChips, renamed, SIDEBAR_ORDER } from './sidebar.ts';

const finding = (severity: Finding['severity']): Finding => ({ check: 'c', message: 'm', path: '/', severity });

test('idle shows no status', () => {
  assert.equal(phaseLabel('idle', null, false), null);
});

test('every other phase has its label', () => {
  assert.equal(phaseLabel('arming', null, false), 'opening microphone…');
  assert.equal(phaseLabel('listening', null, false), 'listening');
  assert.equal(phaseLabel('transcribing', null, false), 'transcribing…');
  assert.equal(phaseLabel('thinking', null, false), 'thinking…');
  assert.equal(phaseLabel('thinking', 'page:loans', false), 'thinking about page:loans…');
});

test('a goal being planned shows while the microphone is idle', () => {
  assert.equal(phaseLabel('idle', null, true), 'planning the goal…');
});

test('the findings badge counts errors and warnings and names them in full in its title', () => {
  const b = findingsBadge([finding('error'), finding('warning'), finding('error')]);
  assert.equal(b.errors, 2);
  assert.equal(b.warnings, 1);
  assert.equal(b.title, '2 errors, 1 warning');
});

test('a document without findings has a quiet badge', () => {
  const b = findingsBadge([]);
  assert.deepEqual([b.errors, b.warnings, b.title], [0, 0, 'no findings']);
});

test('a chip rename sends only a new, non-blank name', () => {
  assert.equal(renamed('  Ada  ', 'Grace'), 'Ada');
  assert.equal(renamed('   ', 'Grace'), null);
  assert.equal(renamed('Grace ', 'Grace'), null);
});

test('the sidebar is ordered header, card, input, tree, activity', () => {
  assert.deepEqual(SIDEBAR_ORDER, ['header', 'card', 'input', 'tree', 'activity']);
});

test('the sidebar shows no idle status', () => {
  const sfc = readFileSync(new URL('../components/SidebarPanel.vue', import.meta.url), 'utf8');
  assert.doesNotMatch(sfc, /'idle'/);
});

const CONNS = ['open', 'connecting', 'closed'] as const;
const op = (id: string, name: string, local = false, kind: 'human' | 'agent' = 'human') => ({ id, name, kind, colour: '#000', local });

test('the local operator has a chip, first, in every connection state before presence lists it', () => {
  for (const conn of CONNS) {
    for (const listed of [[], [op('h-1', 'Grace'), op('a-1', 'planner', false, 'agent')]]) {
      const chips = presenceChips(listed, 'Ada', conn);
      assert.deepEqual(
        chips.map((c) => [c.name, c.local, !!c.pending]),
        [['Ada', true, true], ...listed.map((o) => [o.name, false, false])],
        `${conn} with ${listed.length} listed`,
      );
    }
  }
});

test('a local operator presence lists comes first and is not pending, in every connection state', () => {
  for (const conn of CONNS) {
    const chips = presenceChips([op('h-1', 'Grace'), op('h-2', 'Ada', true)], 'Ada', conn);
    assert.deepEqual(
      chips.map((c) => [c.id, c.local, !!c.pending]),
      [
        ['h-2', true, false],
        ['h-1', false, false],
      ],
      conn,
    );
  }
});

test('a chip title says who it is and, for a pending local chip, why it is pending', () => {
  const [pendingOpen] = presenceChips([], 'Ada', 'open');
  const [pendingClosed] = presenceChips([], 'Ada', 'closed');
  const [pendingConnecting] = presenceChips([], 'Ada', 'connecting');
  const [listedLocal, other] = presenceChips([op('h-2', 'Ada', true), op('a-1', 'planner', false, 'agent')], 'Ada', 'open');
  assert.equal(pendingOpen.title, 'you (presence does not list this browser yet) · click to rename');
  assert.equal(pendingClosed.title, 'you (not connected) · click to rename');
  assert.equal(pendingConnecting.title, 'you (not connected) · click to rename');
  assert.equal(listedLocal.title, 'you · click to rename');
  assert.equal(other.title, 'planner · agent · a-1');
});
