import type { UilabWireProposalShown as ProposalShown, UilabWireServerMessage as ServerMessage } from '../generated/types.ts';
import { normalize } from './outline.ts';

/** The Accept/Reject card: the proposal waiting for a decision, and whether one was sent. */
export interface Card {
  pending: ProposalShown | null;
  /** Accept or reject sent; the card stays until the outcome arrives. */
  deciding: boolean;
}

export const NO_CARD: Card = { pending: null, deciding: false };

/**
 * The card after one server message. A proposal opens it; a decided card closes on the next
 * `document` or `changed`. An undecided card also closes when the server applied the proposal by
 * itself (`--auto-apply`): a `document` whose undoable is that proposal, or a `changed` from the
 * same operator at the same path. A `goal` whose step holds the proposal as accepted or rejected
 * closes it too.
 */
export function settleCard(card: Card, msg: ServerMessage): Card {
  const p = card.pending;
  switch (msg.type) {
    case 'proposal':
      // The same proposal again is no new proposal: a decision already sent stands.
      if (p && p.proposal_id === msg.value.proposal_id) return card;
      return { pending: msg.value, deciding: false };
    case 'document':
      return p && (card.deciding || msg.value.undoable === p.proposal_id) ? NO_CARD : card;
    case 'changed': {
      const c = msg.value;
      const applied = !!p && !!p.by && p.by === c.by && normalize(p.changed) === normalize(c.changed);
      return p && (card.deciding || applied) ? NO_CARD : card;
    }
    case 'goal': {
      // A new goal: the server rejected whatever proposal was waiting.
      if (p && msg.value.state === 'planning') return NO_CARD;
      // A goal step decided elsewhere, or its proposal rejected by a stop.
      const step = p ? msg.value.steps.find((s) => s.proposal_id === p.proposal_id) : undefined;
      return step && (step.status === 'accepted' || step.status === 'rejected') ? NO_CARD : card;
    }
    default:
      return card;
  }
}
