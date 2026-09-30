import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireGoal as Goal,
  UilabWireServerMessage as ServerMessage,
  UilabWireStepStatus as StepStatus,
} from '../generated/types.ts';

/** One icon per step status, for the goal panel. */
export const STEP_ICONS: Record<StepStatus, string> = {
  pending: '○',
  thinking: '…',
  proposed: '?',
  accepted: '✓',
  rejected: '✗',
  refused: '⚠',
  declined: '–',
};

/** A step as the goal panel shows it. */
export interface StepView {
  index: number;
  instruction: string;
  target: string;
  why: string;
  status: StepStatus;
  icon: string;
  proposal_id?: string;
  /** The step being carried out now. */
  current: boolean;
}

/** The goal after one server message: a `goal` message replaces it whole; nothing else touches it. */
export function settleGoal(goal: Goal | null, msg: ServerMessage): Goal | null {
  return msg.type === 'goal' ? msg.value : goal;
}

/**
 * Whether `after` is the moment `before` ended: the same goal, planning or running before and
 * over now. A goal sent again, or first seen already over, ended nothing just now.
 */
export function goalEnded(before: Goal | null, after: Goal): boolean {
  return !!before && before.goal_id === after.goal_id && isActive(before) && !isActive(after);
}

/** Planning or running: the goal still acts. */
export function isActive(goal: Goal | null): boolean {
  return goal?.state === 'planning' || goal?.state === 'running';
}

export function canStop(goal: Goal | null): boolean {
  return isActive(goal);
}

/** `goal i/n` for the agent banner while the goal runs, `goal: planning` before it has steps. */
export function bannerLabel(goal: Goal | null): string | null {
  if (!goal || !isActive(goal)) return null;
  if (goal.state === 'planning') return 'goal: planning';
  return `goal ${(goal.current ?? 0) + 1}/${goal.steps.length}`;
}

export function stepViews(goal: Goal | null): StepView[] {
  if (!goal) return [];
  const current = isActive(goal) ? goal.current : undefined;
  return goal.steps.map((s, index) => ({
    index,
    instruction: s.instruction,
    target: s.target,
    why: s.why,
    status: s.status,
    icon: STEP_ICONS[s.status],
    proposal_id: s.proposal_id,
    current: index === current,
  }));
}

export function stateLabel(goal: Goal): string {
  switch (goal.state) {
    case 'planning':
      return 'planning…';
    case 'running':
      return `running step ${(goal.current ?? 0) + 1} of ${goal.steps.length}`;
    case 'failed':
      return goal.message ? `failed: ${goal.message}` : 'failed';
    default:
      return goal.state;
  }
}

/** The `goal` message for typed text at a target; `null` for blank text. */
export function goalMessage(text: string, target: string | undefined): ClientMessage | null {
  const t = text.trim();
  if (!t) return null;
  return { type: 'goal', value: target ? { text: t, target } : { text: t } };
}

/** The `stop_goal` message for a goal that still acts; `null` otherwise. */
export function stopMessage(goal: Goal | null): ClientMessage | null {
  return goal && isActive(goal) ? { type: 'stop_goal', value: { goal_id: goal.goal_id } } : null;
}
