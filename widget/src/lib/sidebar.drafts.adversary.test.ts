import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireFinding as Finding } from '../generated/types.ts';
import { draftView, draftViews, findingsBadge } from './sidebar.ts';

/**
 * A placeholder read as the server sends it on `ess-ui/1`: ESS's `unbound_placeholder` finding
 * (ess-ui-check 0.48.0, `rules.rs`), shown on the uilab section that reads it, its message naming
 * ESS's path of the read first (`crates/uilab-doc/src/ess.rs` `findings`). The Rust test
 * `wire::tests::a_placeholder_read_is_sent_as_the_sidebar_reads_it` holds the server to exactly
 * this shape, so a change on either side fails one of the two.
 */
function served(path: string, placeholder: string, fixture = 'no fixture'): Finding {
  const [page, section] = path.split('/').map((s) => s.split(':')[1]);
  const ess = page && section && path.startsWith('page:') ? `pages/${page}/sections/${section}/reads` : `${path}/reads`;
  return {
    check: 'unbound_placeholder',
    severity: 'warning',
    path,
    message: `\`${ess}\`: \`${placeholder}\` is a placeholder read answered by \`${fixture}\`; bind it to a view`,
  };
}

test('draftView reads the placeholder out of the message the server sends', () => {
  for (const name of ['LoansPerMonth', 'member_history.v2', 'Profile_Settings2', 'a-b', 'x']) {
    assert.equal(draftView(served('page:p/section:s', name).message), name, name);
    assert.equal(draftView(served('page:p/section:s', name, 'fixtures/p.yaml').message), name, name);
  }
});

test('the operator document as the server sends it: badge quiet, three placeholders, real paths, a widget body read last', () => {
  const findings: Finding[] = [
    served('page:overview/section:loans_per_month', 'LoansPerMonth'),
    served('page:overview/section:loans_by_state', 'LoansByState', 'fixtures/loans_by_state.yaml'),
    served('page:profile/section:loans_chart', 'LoansPerMonth'),
    served('page:profile/section:profile_form', 'ProfileSettings'),
    served('page:showcase/section:trend', 'LoansPerMonth'),
    served('component:member_card/node:history', 'member_history.v2'),
  ];
  assert.deepEqual([findingsBadge(findings).errors, findingsBadge(findings).warnings], [0, 0]);
  assert.deepEqual(draftViews(findings), [
    {
      view: 'LoansPerMonth',
      paths: ['page:overview/section:loans_per_month', 'page:profile/section:loans_chart', 'page:showcase/section:trend'],
    },
    { view: 'LoansByState', paths: ['page:overview/section:loans_by_state'] },
    { view: 'ProfileSettings', paths: ['page:profile/section:profile_form'] },
    { view: 'member_history.v2', paths: ['component:member_card/node:history'] },
  ]);
});

test(
  'a placeholder name holding a backtick is listed whole',
  { todo: 'INFEASIBLE: nothing restricts a placeholder name, but no document was found that puts a backtick in one' },
  () => {
    assert.equal(draftView(served('page:p/section:s', 'a`b').message), 'a`b');
  },
);
