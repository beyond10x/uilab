import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { draftView, draftViews, draftsLabel, faults, findingsBadge, phaseLabel, presenceChips, renamed, SIDEBAR_ORDER } from './sidebar.ts';

const finding = (severity: Finding['severity']): Finding => ({ check: 'c', message: 'm', path: '/', severity });

const draftRead = (path: string, view: string): Finding => ({
  check: 'draft_read',
  severity: 'warning',
  path,
  message: `reads \`${view}\`, a placeholder with no model binding yet`,
});

/** The operator's document on 2026-09-30: five draft reads over three views, in document order. */
const OPERATOR_DRAFTS: Finding[] = [
  draftRead('page:dashboard/section:loans_chart', 'draft.LoansPerMonth'),
  draftRead('page:dashboard/section:state_chart', 'draft.LoansByState'),
  draftRead('page:reports/section:monthly', 'draft.LoansPerMonth'),
  draftRead('page:profile/section:settings', 'draft.ProfileSettings'),
  draftRead('page:reports/section:trend', 'draft.LoansPerMonth'),
];

test('the findings badge does not count draft reads', () => {
  const b = findingsBadge([...OPERATOR_DRAFTS, finding('error'), finding('warning')]);
  assert.deepEqual([b.errors, b.warnings, b.title], [1, 1, '1 error, 1 warning']);
});

test('a document whose only findings are draft reads has the quiet badge', () => {
  const b = findingsBadge(OPERATOR_DRAFTS);
  assert.deepEqual([b.errors, b.warnings, b.title], [0, 0, 'no findings']);
});

test('the findings list leaves draft reads out and keeps every other finding in order', () => {
  const e = finding('error');
  const w = finding('warning');
  assert.deepEqual(faults([OPERATOR_DRAFTS[0], e, OPERATOR_DRAFTS[1], w]), [e, w]);
  assert.deepEqual(faults(OPERATOR_DRAFTS), []);
});

test('the view a draft read names is read from its message', () => {
  assert.equal(draftView(OPERATOR_DRAFTS[0].message), 'draft.LoansPerMonth');
  assert.equal(draftView('reads `draft.Profile_Settings2`, a placeholder'), 'draft.Profile_Settings2');
  assert.equal(draftView('something else entirely'), null);
});

test('the draft list names each view once, in document order, with the sections that read it', () => {
  assert.deepEqual(draftViews([finding('error'), ...OPERATOR_DRAFTS]), [
    { view: 'draft.LoansPerMonth', paths: ['page:dashboard/section:loans_chart', 'page:reports/section:monthly', 'page:reports/section:trend'] },
    { view: 'draft.LoansByState', paths: ['page:dashboard/section:state_chart'] },
    { view: 'draft.ProfileSettings', paths: ['page:profile/section:settings'] },
  ]);
});

test('a draft read whose message names no view is still listed, under its message', () => {
  const odd: Finding = { check: 'draft_read', severity: 'warning', path: 'page:x/section:y', message: 'placeholder' };
  assert.deepEqual(draftViews([odd]), [{ view: 'placeholder', paths: ['page:x/section:y'] }]);
});

test('a document without draft reads has an empty draft list', () => {
  assert.deepEqual(draftViews([finding('warning')]), []);
});

test('the collapsed draft list says how many views it holds', () => {
  assert.equal(draftsLabel(3), '3 draft views');
  assert.equal(draftsLabel(1), '1 draft view');
});

test('the sidebar lists faults, not every finding, and has a collapsed data-to-model list', () => {
  const sfc = readFileSync(new URL('../components/SidebarPanel.vue', import.meta.url), 'utf8');
  assert.doesNotMatch(sfc, /in doc\.findings"/);
  assert.match(sfc, /draftViews\(/);
  assert.match(sfc, /const draftsOpen = ref\(false\)/);
  assert.match(sfc, /Data to model/);
});

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
