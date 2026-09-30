import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireOutlineNode as OutlineNode,
  UilabWireProposalShown as ProposalShown,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';
import { NO_CARD, settleCard, type Card } from './card.ts';

const outline: OutlineNode = { path: '/', layer: 'root', name: '', kind: 'document', children: [] };

function proposal(id: string, by?: string, changed = 'page:loans/section:x'): ProposalShown {
  return { proposal_id: id, by, op: 'Insert', target: 'page:loans', changed, utterance: 'add x', before: '', after: '', findings: [], outline };
}

const msg = {
  proposal: (p: ProposalShown): ServerMessage => ({ type: 'proposal', value: p }),
  changed: (by: string, changed = 'page:loans/section:x'): ServerMessage => ({
    type: 'changed',
    value: { by, revision: 1, op: 'Insert', changed, parent: 'page:loans', findings: [] },
  }),
  document: (undoable?: string): ServerMessage => ({
    type: 'document',
    value: { document_id: 'd', file: 'f', selected: '/', outline, findings: [], revision: 1, review: true, undoable },
  }),
  refused: (): ServerMessage => ({ type: 'refused', value: { check: 'c', message: 'm' } }),
};

const open = (p: ProposalShown): Card => settleCard(NO_CARD, msg.proposal(p));

test('a proposal opens the card; a new one replaces it and resets deciding', () => {
  const p1 = proposal('p1', 'h1');
  const p2 = proposal('p2', 'a1');
  assert.deepEqual(open(p1), { pending: p1, deciding: false });
  assert.deepEqual(settleCard({ pending: p1, deciding: true }, msg.proposal(p2)), { pending: p2, deciding: false });
});

test('the same proposal again keeps the card as it is, decision sent or not', () => {
  const p = proposal('p1', 'a1');
  const deciding: Card = { pending: p, deciding: true };
  assert.equal(settleCard(deciding, msg.proposal({ ...p })), deciding);
  const card = open(p);
  assert.equal(settleCard(card, msg.proposal({ ...p })), card);
});

test('an undecided card survives unrelated documents, changes and refusals', () => {
  const p = proposal('p1', 'a1');
  const card = open(p);
  assert.equal(settleCard(card, msg.document()), card, 'a document with nothing undoable');
  assert.equal(settleCard(card, msg.document('p0')), card, 'a document undoing an older proposal');
  assert.equal(settleCard(card, msg.changed('h1')), card, 'a change by another operator');
  assert.equal(settleCard(card, msg.changed('a1', 'page:members/section:y')), card, 'a change by the same operator elsewhere');
  assert.equal(settleCard(card, msg.refused()), card);
});

test('without an operator on the proposal, only a decision or its own undoable closes it', () => {
  const card = open(proposal('p1'));
  assert.equal(settleCard(card, msg.changed('a1')), card);
  assert.deepEqual(settleCard(card, msg.document('p1')), NO_CARD);
});

test('a decided card closes on the next document or changed', () => {
  const card: Card = { pending: proposal('p1', 'h1'), deciding: true };
  assert.deepEqual(settleCard(card, msg.document()), NO_CARD);
  assert.deepEqual(settleCard(card, msg.changed('a1', 'elsewhere')), NO_CARD);
});

test('auto-apply: proposal then changed from the same operator leaves no card', () => {
  const card = open(proposal('p1', 'a1'));
  assert.deepEqual(settleCard(card, msg.changed('a1', '/page:loans/section:x/')), NO_CARD);
});

test('auto-apply: proposal then a document that can undo it leaves no card', () => {
  const card = open(proposal('p1', 'h1'));
  assert.deepEqual(settleCard(card, msg.document('p1')), NO_CARD);
});

test('a goal message that decided the card proposal closes it; one still waiting on it keeps it', () => {
  const card = open(proposal('p1', 'a1'));
  const goal = (status: 'proposed' | 'accepted' | 'rejected', proposal_id = 'p1'): ServerMessage => ({
    type: 'goal',
    value: {
      goal_id: 'goal-1',
      by: 'a1',
      text: 't',
      state: 'running',
      steps: [{ instruction: 'i', target: 'page:loans', why: 'w', status, proposal_id }],
    },
  });
  assert.equal(settleCard(card, goal('proposed')), card);
  assert.equal(settleCard(card, goal('rejected', 'p0')), card, 'another proposal');
  assert.deepEqual(settleCard(card, goal('rejected')), NO_CARD, 'stopped or rejected elsewhere');
  assert.deepEqual(settleCard(card, goal('accepted')), NO_CARD);
});

test('without a card, documents and changes leave it empty', () => {
  assert.equal(settleCard(NO_CARD, msg.document('p1')), NO_CARD);
  assert.equal(settleCard(NO_CARD, msg.changed('a1')), NO_CARD);
});
