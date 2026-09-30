import type {
  UilabWireChanged as Changed,
  UilabWireOperator as Operator,
  UilabWireOperatorKind as OperatorKind,
  UilabWireOutlineNode as OutlineNode,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';
import { normalize } from './outline.ts';

// ---------------------------------------------------------------------------------------------
// Operator identity

/** The local operator's name: `?name=` wins, then the stored name, then the default. */
export function initialName(search: string, stored: string | null, fallback = 'Timo'): string {
  const fromQuery = new URLSearchParams(search).get('name')?.trim();
  if (fromQuery) return fromQuery;
  const kept = stored?.trim();
  return kept || fallback;
}

/** Colours for humans and for agents; the two sets do not overlap, so an agent always reads as one. */
export const HUMAN_COLOURS = ['#0891b2', '#4f46e5', '#0f766e', '#475569', '#0369a1', '#57534e'] as const;
export const AGENT_COLOURS = ['#c026d3', '#db2777', '#9333ea', '#be123c', '#a21caf'] as const;

/** FNV-1a over the UTF-16 code units: stable across sessions and browsers. */
export function hashId(id: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < id.length; i++) {
    h ^= id.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

/** The colour of an operator: its id hashed into the palette of its kind. */
export function operatorColour(id: string, kind: OperatorKind = 'human'): string {
  const palette = kind === 'agent' ? AGENT_COLOURS : HUMAN_COLOURS;
  return palette[hashId(id) % palette.length];
}

/**
 * The id presence gives the local operator. The server does not echo an id for `hello`, so this
 * is the human operator of the local name; with several of that name, none is claimed.
 */
export function localOperatorId(operators: readonly Operator[], name: string): string | null {
  const mine = operators.filter((o) => o.kind === 'human' && o.name === name);
  return mine.length === 1 ? mine[0].id : null;
}

// ---------------------------------------------------------------------------------------------
// Revisions and incremental outline changes

/** What to do with a `changed` of `revision` after `last`: apply it, ignore it, or resync. */
export type RevisionStep = 'apply' | 'stale' | 'gap';

export function revisionStep(last: number | null, revision: number): RevisionStep {
  if (last === null) return 'gap';
  if (revision === last + 1) return 'apply';
  if (revision <= last) return 'stale';
  return 'gap';
}

/**
 * The outline after one change, or `null` when the change does not fit this outline (a missing
 * parent or node, an insert of a path already there): the caller then resyncs. Nodes off the
 * changed branch are shared with the input; the branch itself is copied.
 */
export function applyChange(root: OutlineNode, change: Pick<Changed, 'op' | 'changed' | 'parent' | 'node'>): OutlineNode | null {
  const target = normalize(change.changed);
  const parentPath = normalize(change.parent);
  if (target === '/') {
    // Only a whole-document replacement can name the root.
    return change.op === 'Replace' && change.node && parentPath === '/' ? change.node : null;
  }
  const edit = (parent: OutlineNode): OutlineNode | null => {
    const at = parent.children.findIndex((c) => normalize(c.path) === target);
    switch (change.op) {
      case 'Insert':
        if (!change.node || at >= 0) return null;
        return { ...parent, children: [...parent.children, change.node] };
      case 'Replace': {
        if (!change.node || at < 0) return null;
        const children = parent.children.slice();
        children[at] = change.node;
        return { ...parent, children };
      }
      case 'Remove':
        if (at < 0) return null;
        return { ...parent, children: parent.children.filter((_, i) => i !== at) };
      default:
        return null;
    }
  };
  const walk = (node: OutlineNode): OutlineNode | null => {
    const here = normalize(node.path);
    if (here === parentPath) return edit(node);
    const next = node.children.findIndex((c) => {
      const p = normalize(c.path);
      return parentPath === p || parentPath.startsWith(p + '/');
    });
    if (next < 0) return null;
    const replaced = walk(node.children[next]);
    if (!replaced) return null;
    const children = node.children.slice();
    children[next] = replaced;
    return { ...node, children };
  };
  return walk(root);
}

/** The path to flash after a change: the node itself, or its parent when it was removed. */
export function flashPath(change: Pick<Changed, 'op' | 'changed' | 'parent'>): string {
  return change.op === 'Remove' ? change.parent : change.changed;
}

// ---------------------------------------------------------------------------------------------
// Actions in flight

/** Operator id → the target of the action that operator has in flight. */
export type InFlight = Readonly<Record<string, string>>;

/** The in-flight map after one server message: `thinking` starts an action, an outcome ends it. */
export function trackInFlight(inFlight: InFlight, msg: ServerMessage): InFlight {
  switch (msg.type) {
    case 'thinking': {
      const by = msg.value.by;
      if (!by) return inFlight;
      return { ...inFlight, [by]: msg.value.target };
    }
    case 'proposal':
    case 'refused':
    case 'failed':
    case 'changed': {
      const by = msg.value.by;
      if (!by || !(by in inFlight)) return inFlight;
      const next = { ...inFlight };
      delete next[by];
      return next;
    }
    default:
      return inFlight;
  }
}

/** Drops actions of operators presence no longer lists. */
export function pruneInFlight(inFlight: InFlight, operators: readonly Operator[]): InFlight {
  const present = new Set(operators.map((o) => o.id));
  const kept = Object.entries(inFlight).filter(([id]) => present.has(id));
  return kept.length === Object.keys(inFlight).length ? inFlight : Object.fromEntries(kept);
}

// ---------------------------------------------------------------------------------------------
// Activity feed

export type FeedKind = 'thinking' | 'transcript' | 'proposal' | 'changed' | 'refused' | 'failed';

export interface FeedEntry {
  seq: number;
  at: number;
  kind: FeedKind;
  /** Operator id, when the message named one. */
  by?: string;
  /** Operation for `proposal`/`changed`, check for `refused`. */
  what?: string;
  path?: string;
  text?: string;
}

export const FEED_LIMIT = 50;

/** The feed entry for a message, or `null` for messages the feed does not show. */
export function feedEntry(msg: ServerMessage, at: number, seq: number, inFlight: InFlight = {}): FeedEntry | null {
  const base = { seq, at };
  switch (msg.type) {
    case 'thinking':
      return { ...base, kind: 'thinking', by: msg.value.by, path: msg.value.target };
    case 'transcript':
      return { ...base, kind: 'transcript', by: msg.value.by, text: msg.value.text };
    case 'proposal':
      return { ...base, kind: 'proposal', by: msg.value.by, what: msg.value.op, path: msg.value.changed, text: msg.value.utterance };
    case 'changed':
      return { ...base, kind: 'changed', by: msg.value.by, what: msg.value.op, path: msg.value.changed };
    case 'refused': {
      const by = msg.value.by;
      return { ...base, kind: 'refused', by, what: msg.value.check, path: by ? inFlight[by] : undefined, text: msg.value.message };
    }
    case 'failed': {
      const by = msg.value.by;
      return { ...base, kind: 'failed', by, path: by ? inFlight[by] : undefined, text: msg.value.message };
    }
    default:
      return null;
  }
}

/** The feed with `entry` first, keeping the newest `limit`. */
export function pushFeed(feed: readonly FeedEntry[], entry: FeedEntry, limit = FEED_LIMIT): FeedEntry[] {
  return [entry, ...feed].slice(0, limit);
}
