import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { draftView, draftViews, findingsBadge } from './sidebar.ts';

/**
 * The message the check formats for a draft read, read from the Rust source that formats it, so a
 * change there fails here instead of silently turning every view name into a raw message.
 */
function draftReadFormat(): string {
  const src = readFileSync(new URL('../../../crates/uilab-doc/src/check.rs', import.meta.url), 'utf8');
  const found = [...src.matchAll(/"draft_read",\s*&path,\s*format!\(\s*"((?:[^"\\]|\\.)*)"/g)];
  assert.equal(found.length, 1, 'check.rs formats exactly one draft_read message');
  const fmt = found[0][1];
  assert.equal(fmt.split('{}').length, 2, `one positional placeholder in ${JSON.stringify(fmt)}`);
  assert.doesNotMatch(fmt, /\{\{|\}\}|\{[^}]/, 'no escaped or named placeholders');
  return fmt;
}

const served = (path: string, view: string): Finding => ({
  check: 'draft_read',
  severity: 'warning',
  path,
  message: draftReadFormat().replace('{}', view),
});

test('draftView reads the view out of the message check.rs actually formats', () => {
  for (const view of ['draft.LoansPerMonth', 'draft.member_history.v2', 'draft.Profile_Settings2', 'draft.a-b', 'draft.']) {
    assert.equal(draftView(served('page:p/section:s', view).message), view, view);
  }
});

test('the operator document as the check formats it: badge quiet, three views, real paths, a widget body read last', () => {
  const findings: Finding[] = [
    served('page:overview/section:loans_per_month', 'draft.LoansPerMonth'),
    served('page:overview/section:loans_by_state', 'draft.LoansByState'),
    served('page:profile/section:loans_chart', 'draft.LoansPerMonth'),
    served('page:profile/section:profile_form', 'draft.ProfileSettings'),
    served('page:showcase/section:trend', 'draft.LoansPerMonth'),
    served('component:member_card/node:history', 'draft.member_history.v2'),
  ];
  assert.deepEqual([findingsBadge(findings).errors, findingsBadge(findings).warnings], [0, 0]);
  assert.deepEqual(draftViews(findings), [
    {
      view: 'draft.LoansPerMonth',
      paths: ['page:overview/section:loans_per_month', 'page:profile/section:loans_chart', 'page:showcase/section:trend'],
    },
    { view: 'draft.LoansByState', paths: ['page:overview/section:loans_by_state'] },
    { view: 'draft.ProfileSettings', paths: ['page:profile/section:profile_form'] },
    { view: 'draft.member_history.v2', paths: ['component:member_card/node:history'] },
  ]);
});

test(
  'a view name holding a backtick is listed whole',
  { todo: 'INFEASIBLE: nothing restricts a view name, but no document was found that puts a backtick in one' },
  () => {
    assert.equal(draftView(served('page:p/section:s', 'draft.a`b').message), 'draft.a`b');
  },
);
