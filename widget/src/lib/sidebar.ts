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

/** The header's findings badge for the document's findings; draft reads are not counted. */
export function findingsBadge(findings: readonly Pick<Finding, 'severity' | 'check'>[]): FindingsBadge {
  const counted = faults(findings);
  const errors = counted.filter((f) => f.severity === 'error').length;
  const warnings = counted.filter((f) => f.severity === 'warning').length;
  const title = errors || warnings ? `${plural(errors, 'error')}, ${plural(warnings, 'warning')}` : 'no findings';
  return { errors, warnings, title };
}

/**
 * The check that reports a read of a `draft.` view. The check still reports it (the agent, the eval
 * and `/api/state` see it); the sidebar shows it as data to model, not as a warning.
 */
export const DRAFT_READ = 'draft_read';

/** The findings that are faults: every finding but a draft read, in order. */
export function faults<F extends Pick<Finding, 'check'>>(findings: readonly F[]): F[] {
  return findings.filter((f) => f.check !== DRAFT_READ);
}

/** The view a draft read's message names (``reads `draft.X`, …``), or `null` when it names none. */
export function draftView(message: string): string | null {
  return /reads `([^`]+)`/.exec(message)?.[1] ?? null;
}

/** A draft view and the sections that read it, in document order. */
export interface DraftView {
  view: string;
  paths: string[];
}

/**
 * The document's open data needs: each draft view once, in the order the document first reads it,
 * with every section that reads it. A message that names no view is listed under the message.
 */
export function draftViews(findings: readonly Pick<Finding, 'check' | 'path' | 'message'>[]): DraftView[] {
  const byView = new Map<string, DraftView>();
  for (const f of findings) {
    if (f.check !== DRAFT_READ) continue;
    const view = draftView(f.message) ?? f.message;
    const entry = byView.get(view) ?? { view, paths: [] };
    if (!entry.paths.includes(f.path)) entry.paths.push(f.path);
    byView.set(view, entry);
  }
  return [...byView.values()];
}

/** The collapsed draft list's label. */
export function draftsLabel(n: number): string {
  return plural(n, 'draft view');
}

/** The name a chip edit sends: trimmed, and only when it is not blank and differs from the current one. */
export function renamed(draft: string, current: string): string | null {
  const n = draft.trim();
  return n && n !== current ? n : null;
}
