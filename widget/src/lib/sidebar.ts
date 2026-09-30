import type { UilabWireFinding as Finding } from '../generated/types.ts';

/** What the operator is waiting on; mirrors the store's `Phase`. */
export type Phase = 'idle' | 'arming' | 'listening' | 'transcribing' | 'thinking';

/** The sidebar's sections, top to bottom: a waiting proposal comes before everything but the header. */
export const SIDEBAR_ORDER = ['header', 'card', 'input', 'tree', 'activity'] as const;

/** The connection states; mirrors the transport's `ConnState`. */
export type Conn = 'connecting' | 'open' | 'closed';

/** An operator as the header shows it. */
export interface ChipOperator {
  id: string;
  name: string;
  kind: 'human' | 'agent';
  colour: string;
  local: boolean;
}

export interface Chip extends ChipOperator {
  /** The local operator before presence lists it. */
  pending?: boolean;
  title: string;
}

function chipTitle(o: ChipOperator & { pending?: boolean }, conn: Conn): string {
  if (!o.local) return `${o.name} · ${o.kind} · ${o.id}`;
  if (!o.pending) return 'you · click to rename';
  return conn === 'open' ? 'you (presence does not list this browser yet) · click to rename' : 'you (not connected) · click to rename';
}

/**
 * The header's chips: the local operator first, then the others presence lists. The local chip is
 * there in every connection state, so a browser can rename itself before presence lists it and
 * while it is not connected; until presence lists it, it is a pending chip of the local name.
 */
export function presenceChips(listed: readonly ChipOperator[], name: string, conn: Conn): Chip[] {
  const sorted = [...listed].sort((a, b) => Number(b.local) - Number(a.local));
  const out: (ChipOperator & { pending?: boolean })[] = sorted.some((o) => o.local)
    ? sorted
    : [{ id: '', name, kind: 'human', colour: 'var(--muted)', local: true, pending: true }, ...sorted];
  return out.map((o) => ({ ...o, title: chipTitle(o, conn) }));
}

/**
 * The status line under the microphone. Idle says nothing (`null`); a goal being planned while the
 * microphone is idle is shown, and so is every other phase.
 */
export function phaseLabel(phase: Phase, thinkingTarget: string | null, goalPlanning: boolean): string | null {
  switch (phase) {
    case 'arming':
      return 'opening microphone…';
    case 'listening':
      return 'listening';
    case 'transcribing':
      return 'transcribing…';
    case 'thinking':
      return thinkingTarget ? `thinking about ${thinkingTarget}…` : 'thinking…';
    default:
      return goalPlanning ? 'planning the goal…' : null;
  }
}

export interface FindingsBadge {
  errors: number;
  warnings: number;
  /** The counts in words, for the badge's tooltip and accessible name. */
  title: string;
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? '' : 's'}`;
}

/** The header's findings badge for the document's findings. */
export function findingsBadge(findings: readonly Pick<Finding, 'severity'>[]): FindingsBadge {
  const errors = findings.filter((f) => f.severity === 'error').length;
  const warnings = findings.filter((f) => f.severity === 'warning').length;
  const title = errors || warnings ? `${plural(errors, 'error')}, ${plural(warnings, 'warning')}` : 'no findings';
  return { errors, warnings, title };
}

/** The name a chip edit sends: trimmed, and only when it is not blank and differs from the current one. */
export function renamed(draft: string, current: string): string | null {
  const n = draft.trim();
  return n && n !== current ? n : null;
}
