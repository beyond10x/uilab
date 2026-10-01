import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { draftView, draftViews, faults, findingsBadge } from './sidebar.ts';

// story:essui-app-widget. On `ess-ui/1` a read with no model binding yet is a placeholder, and ESS
// reports it as `unbound_placeholder`; uilab forwards the finding on the section, its message
// naming ESS's path first (`crates/uilab-doc/src/ess.rs` `findings`). The sidebar lists it as data
// to model, as it listed a `draft.` read, and does not count it as a warning.

const placeholder = (path: string, ess: string, name: string, fixture = 'no fixture'): Finding => ({
  check: 'unbound_placeholder',
  severity: 'warning',
  path,
  message: `\`${ess}/reads\`: \`${name}\` is a placeholder read answered by \`${fixture}\`; bind it to a view`,
});

const history = placeholder('page:loans/section:history', 'pages/loans/sections/history', 'LoanHistory', 'fixtures/history.yaml');
const trend = placeholder('page:overview/section:trend', 'pages/overview/sections/trend', 'LoansPerMonth');
const again = placeholder('page:members/section:trend', 'pages/members/sections/trend', 'LoansPerMonth');
const missing: Finding = { check: 'fixture_per_view', severity: 'warning', path: 'page:loans/section:list', message: 'no fixture answers view `loans.All`' };

test('the placeholder name is read out of the forwarded message', () => {
  assert.equal(draftView(history.message), 'LoanHistory');
  assert.equal(draftView(trend.message), 'LoansPerMonth');
});

test('a placeholder read is data to model, not a warning', () => {
  assert.deepEqual(faults([history, missing, trend]), [missing]);
  assert.deepEqual([findingsBadge([history, trend]).errors, findingsBadge([history, trend]).warnings], [0, 0]);
});

test('placeholders are listed once each, with every section that reads them', () => {
  assert.deepEqual(draftViews([trend, history, again, missing]), [
    { view: 'LoansPerMonth', paths: ['page:overview/section:trend', 'page:members/section:trend'] },
    { view: 'LoanHistory', paths: ['page:loans/section:history'] },
  ]);
});
