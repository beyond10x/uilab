import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { findingsBadge, phaseLabel, renamed, SIDEBAR_ORDER, templateSections } from './sidebar.ts';

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
  const sfc = readFileSync(new URL('../components/SidebarPanel.vue', import.meta.url), 'utf8');
  assert.deepEqual(templateSections(sfc), SIDEBAR_ORDER);
});

test('the sidebar shows no file path row and no idle status', () => {
  const sfc = readFileSync(new URL('../components/SidebarPanel.vue', import.meta.url), 'utf8');
  assert.doesNotMatch(sfc, /class="[^"]*\bfile\b/);
  assert.doesNotMatch(sfc, /'idle'/);
  assert.match(sfc, /:title="doc\.file"/);
});

test('the name edit lives in the operator chip', () => {
  const sfc = readFileSync(new URL('../components/PresenceStrip.vue', import.meta.url), 'utf8');
  assert.doesNotMatch(sfc, /you-name/);
  assert.match(sfc, /class="op-chip[^"]*"[\s\S]*?<input[^>]*aria-label="your name"/);
});

test('template sections are read in document order', () => {
  const sfc = '<template><div data-section="b"></div><x data-section="a" /></template>';
  assert.deepEqual(templateSections(sfc), ['b', 'a']);
});
