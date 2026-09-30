import { computed, reactive, shallowReactive } from 'vue';
import type {
  UilabWireChanged as Changed,
  UilabWireDocumentState as DocumentState,
  UilabWireOperator as Operator,
  UilabWireOperatorKind as OperatorKind,
  UilabWireOutlineNode as OutlineNode,
  UilabWirePresence as Presence,
  UilabWireProposalShown as ProposalShown,
  UilabWireRows as Rows,
  UilabWireServerMessage as ServerMessage,
  UilabWireTranscript as Transcript,
} from './generated/types.ts';
import {
  applyChange,
  feedEntry,
  flashPath,
  initialName,
  localOperatorId,
  operatorColour,
  pruneInFlight,
  pushFeed,
  revisionStep,
  trackInFlight,
  type FeedEntry,
  type InFlight,
} from './lib/collab.ts';
import { findNode, homePage, isDraftView, isWithin, lineage, nearestExisting, normalize, overlayOf, pageOf, pagesOf, segments } from './lib/outline.ts';
import { MicCapture } from './mic.ts';
import type { ConnState, Transport } from './transport.ts';

/** What the operator is waiting on. */
export type Phase = 'idle' | 'arming' | 'listening' | 'transcribing' | 'thinking';

export interface Notice {
  kind: 'refused' | 'failed' | 'local';
  check?: string;
  message: string;
}

export type Mark = 'insert' | 'replace' | 'remove';

const NAME_KEY = 'uilab.operator.name';
const FLASH_MS = 1500;

function storedName(): string | null {
  try {
    return window.localStorage.getItem(NAME_KEY);
  } catch {
    return null;
  }
}

// Shallow: server messages replace whole objects, and deep unwrapping of the recursive JSON types
// is both unneeded and too deep for the type checker.
export const state = shallowReactive({
  conn: 'connecting' as ConnState,
  retryInMs: 0,
  doc: null as DocumentState | null,
  proposal: null as ProposalShown | null,
  /** Accept or reject sent; the proposal stays shown until the next document. */
  deciding: false,
  phase: 'idle' as Phase,
  thinkingTarget: null as string | null,
  transcript: null as Transcript | null,
  /** The last typed instruction. */
  typed: null as string | null,
  notice: null as Notice | null,
  rows: {} as Record<string, Rows>,
  level: 0,
  /** Page shown in the canvas, as `page:<name>`. */
  currentPage: null as string | null,
  /** Overlay open as a modal on the canvas. */
  openOverlay: null as string | null,
  /** Tree nodes shown expanded in the sidebar. */
  expanded: reactive(new Set<string>(['/'])),
  /** The local operator's name, sent in `hello`. */
  name: initialName(window.location.search, storedName()),
  presence: { operators: [] } as Presence,
  /** Revision of the outline shown; `null` before the first document. */
  revision: null as number | null,
  /** A `resync` was sent and its `document` has not come yet. */
  resyncing: false,
  /** Operator id → target of the action that operator has in flight. */
  inFlight: {} as InFlight,
  /** Normalized path → colour, for nodes changed in the last 1.5 s. */
  flashes: {} as Readonly<Record<string, string>>,
  /** Newest first, at most 50. */
  feed: [] as FeedEntry[],
});

let feedSeq = 0;
let flashSeq = 0;

let transport: Transport | null = null;
const rowsRequested = new Set<string>();
/** Views the canvas wants rows for; asked again on reconnect. */
const pendingViews = new Set<string>();
let micHeld = false;

const mic = new MicCapture({
  frame: (buf) => transport?.sendBinary(buf),
  level: (value) => {
    state.level = value;
  },
});

/** The outline the canvas and tree show: the proposal's while one is pending, except for a removal. */
export const shownOutline = computed<OutlineNode | null>(() => {
  const p = state.proposal;
  if (p && p.op !== 'Remove') return p.outline;
  return state.doc?.outline ?? null;
});

/** The node the pending proposal changes and how. */
export const highlight = computed<{ path: string; mark: Mark } | null>(() => {
  const p = state.proposal;
  if (!p) return null;
  const mark: Mark = p.op === 'Insert' ? 'insert' : p.op === 'Replace' ? 'replace' : 'remove';
  return { path: p.changed, mark };
});

/** The page the canvas shows. */
export const activePage = computed<OutlineNode | null>(() => {
  const root = shownOutline.value;
  if (!root) return null;
  const pages = pagesOf(root);
  const byPath = (p: string | null) => (p ? pages.find((x) => x.path === p) ?? null : null);
  return byPath(state.currentPage) ?? byPath(pageOf(state.doc?.selected ?? '')) ?? homePage(root);
});

// ---- operators ---------------------------------------------------------------------------------

/** The local operator's id in presence, when it can be told apart. */
export const localId = computed(() => localOperatorId(state.presence.operators, state.name));

/** An operator as the latest presence knows it. */
export interface OperatorView {
  id: string;
  name: string;
  kind: OperatorKind;
  colour: string;
  local: boolean;
}

export function operatorView(id: string | undefined | null): OperatorView | null {
  if (!id) return null;
  const op: Operator | undefined = state.presence.operators.find((o) => o.id === id);
  const kind: OperatorKind = op?.kind ?? 'human';
  return { id, name: op?.name ?? id, kind, colour: operatorColour(id, kind), local: id === localId.value };
}

/**
 * Whether a message's `by` is this browser's operator: absent, or the local id. When presence
 * cannot say (it lists nobody or several of the local name), a human of the local name counts,
 * and an operator presence does not list yet counts only while this browser waits on an answer.
 */
function isLocal(by: string | undefined): boolean {
  if (!by) return true;
  const id = localId.value;
  if (id) return by === id;
  const op = state.presence.operators.find((o) => o.id === by);
  if (op) return op.kind === 'human' && op.name === state.name;
  return state.phase !== 'idle' || state.deciding;
}

/** Agent actions in flight, one per agent, with the operator resolved. */
export const agentActions = computed(() =>
  Object.entries(state.inFlight)
    .map(([id, target]) => ({ op: operatorView(id)!, target }))
    .filter((a) => a.op.kind === 'agent'),
);

/** Normalized path → colour of the agent operating on it. */
const agentTargets = computed(() => new Map(agentActions.value.map((a) => [normalize(a.target), a.op.colour])));

/** CSS classes marking a node as selected, changed by the proposal, or being thought about. */
export function marks(path: string): Record<string, boolean> {
  const h = highlight.value;
  const p = normalize(path);
  return {
    selected: state.doc?.selected === path,
    [`hl-${h?.mark}`]: !!h && h.path === path,
    thinking: state.phase === 'thinking' && state.thinkingTarget === path,
    'agent-op': agentTargets.value.has(p),
    flash: p in state.flashes,
  };
}

/** Inline style giving a node the colour of the operator acting on it, if any. */
export function tint(path: string | null | undefined): Record<string, string> | undefined {
  if (!path) return undefined;
  const p = normalize(path);
  const colour = agentTargets.value.get(p) ?? state.flashes[p];
  return colour ? { '--op-colour': colour } : undefined;
}

function flash(path: string, colour: string): void {
  const p = normalize(path);
  const token = ++flashSeq;
  flashTokens.set(p, token);
  state.flashes = { ...state.flashes, [p]: colour };
  setTimeout(() => {
    if (flashTokens.get(p) !== token) return;
    flashTokens.delete(p);
    const next = { ...state.flashes };
    delete next[p];
    state.flashes = next;
  }, FLASH_MS);
}
const flashTokens = new Map<string, number>();

export function setName(name: string): void {
  const n = name.trim();
  if (!n || n === state.name) return;
  state.name = n;
  try {
    window.localStorage.setItem(NAME_KEY, n);
  } catch {
    // Storage may be unavailable (private mode); the name still holds for this page.
  }
  hello();
}

function hello(): void {
  transport?.send({ type: 'hello', value: { name: state.name, kind: 'human' } });
}

function send(message: Parameters<Transport['send']>[0]): boolean {
  if (transport?.send(message)) return true;
  state.notice = { kind: 'local', message: 'not connected to the server' };
  return false;
}

function reveal(path: string): void {
  const root = shownOutline.value;
  if (!root) return;
  for (const n of lineage(root, path) ?? []) state.expanded.add(n.path);
  const page = pageOf(path);
  if (page) state.currentPage = page;
}

function resync(): void {
  if (state.resyncing) return;
  if (transport?.send({ type: 'resync', value: { revision: state.revision ?? 0 } })) state.resyncing = true;
}

/** Applies a `changed` in revision order; anything out of order asks for a snapshot instead. */
function onChanged(c: Changed): void {
  const step = revisionStep(state.revision, c.revision);
  if (step === 'stale') return;
  const doc = state.doc;
  const outline = step === 'apply' && doc ? applyChange(doc.outline, c) : null;
  if (!doc || !outline) {
    resync();
    return;
  }
  state.doc = { ...doc, outline, findings: c.findings, revision: c.revision };
  state.revision = c.revision;
  if (state.deciding) {
    state.proposal = null;
    state.deciding = false;
  }
  const at = flashPath(c);
  flash(at, operatorView(c.by)?.colour ?? 'var(--accent)');
  for (const n of lineage(outline, at)?.slice(0, -1) ?? []) state.expanded.add(n.path);
}

function onMessage(msg: ServerMessage): void {
  const entry = feedEntry(msg, Date.now(), ++feedSeq, state.inFlight);
  if (entry) state.feed = pushFeed(state.feed, entry);
  const inFlight = trackInFlight(state.inFlight, msg);
  if (inFlight !== state.inFlight) state.inFlight = inFlight;
  const local = 'by' in msg.value ? isLocal(msg.value.by) : true;

  switch (msg.type) {
    case 'document': {
      const before = state.doc?.selected;
      state.doc = msg.value;
      state.revision = msg.value.revision;
      state.resyncing = false;
      if (state.deciding) {
        state.proposal = null;
        state.deciding = false;
      }
      if (msg.value.selected !== before) reveal(msg.value.selected);
      break;
    }
    case 'changed':
      onChanged(msg.value);
      break;
    case 'presence':
      state.presence = msg.value;
      state.inFlight = pruneInFlight(state.inFlight, msg.value.operators);
      break;
    case 'transcript':
      if (!local) break;
      state.transcript = msg.value;
      state.typed = null;
      if (state.phase === 'transcribing') state.phase = 'idle';
      break;
    case 'thinking':
      if (!local) break;
      state.phase = 'thinking';
      state.thinkingTarget = msg.value.target;
      break;
    case 'proposal': {
      state.proposal = msg.value;
      state.deciding = false;
      if (local) {
        state.phase = 'idle';
        state.notice = null;
        state.openOverlay = overlayOf(msg.value.changed);
        reveal(msg.value.changed);
      }
      break;
    }
    case 'refused':
      if (!local) break;
      state.notice = { kind: 'refused', check: msg.value.check, message: msg.value.message };
      state.phase = 'idle';
      state.deciding = false;
      break;
    case 'failed':
      if (!local) break;
      // The server may refuse audio (speech off) while the operator still holds the key.
      if (state.phase === 'listening') mic.end();
      state.notice = { kind: 'failed', message: msg.value.message };
      state.phase = 'idle';
      state.deciding = false;
      break;
    case 'rows':
      state.rows = { ...state.rows, [msg.value.view]: msg.value };
      break;
  }
}

function onState(conn: ConnState, retryInMs?: number): void {
  state.conn = conn;
  state.retryInMs = retryInMs ?? 0;
  if (conn === 'open') {
    // `hello` goes first on every connection; the server names operators by it.
    hello();
    state.resyncing = false;
    // Requests that were never answered are asked again on the new connection.
    for (const view of rowsRequested) if (!state.rows[view]) rowsRequested.delete(view);
    for (const view of pendingViews) requestRows(view);
  } else if (conn === 'closed') {
    if (state.phase === 'listening') mic.end();
    if (state.phase !== 'idle') state.phase = 'idle';
    micHeld = false;
    // Outcomes sent while disconnected are lost; presence on the next connection says who is left.
    state.inFlight = {};
  }
}

export function connect(t: Transport): void {
  transport = t;
  t.start({ message: onMessage, state: onState });
}

export function select(path: string): void {
  if (!state.doc) return;
  if (send({ type: 'select', value: { path } })) state.doc = { ...state.doc, selected: path };
  reveal(path);
}

export function showPage(path: string): void {
  state.currentPage = path;
  state.openOverlay = null;
}

export function requestRows(view: string | undefined): void {
  if (!view || isDraftView(view)) return;
  pendingViews.add(view);
  if (state.rows[view] || rowsRequested.has(view)) return;
  if (state.conn === 'open' && transport?.send({ type: 'rows', value: { view } })) rowsRequested.add(view);
}

export function say(text: string): void {
  const t = text.trim();
  if (!t) return;
  state.notice = null;
  const target = state.doc?.selected;
  if (send({ type: 'say', value: target ? { text: t, target } : { text: t } })) {
    state.typed = t;
    state.transcript = null;
    state.phase = 'thinking';
    state.thinkingTarget = state.doc?.selected ?? null;
  }
}

export function accept(): void {
  const p = state.proposal;
  if (!p || state.deciding) return;
  if (send({ type: 'accept', value: { proposal_id: p.proposal_id } })) state.deciding = true;
}

export function reject(): void {
  const p = state.proposal;
  if (!p || state.deciding) return;
  if (send({ type: 'reject', value: { proposal_id: p.proposal_id } })) state.deciding = true;
}

export function undo(): void {
  const id = state.doc?.undoable;
  if (!id || state.proposal) return;
  state.notice = null;
  send({ type: 'undo', value: { proposal_id: id } });
}

export async function micDown(): Promise<void> {
  if (micHeld || state.phase === 'listening') return;
  micHeld = true;
  state.notice = null;
  if (state.conn !== 'open') {
    state.notice = { kind: 'local', message: 'not connected to the server' };
    return;
  }
  state.phase = 'arming';
  try {
    await mic.ensure();
  } catch (err) {
    state.phase = 'idle';
    state.notice = { kind: 'local', message: `microphone unavailable: ${err instanceof Error ? err.message : String(err)}` };
    return;
  }
  if (!micHeld) {
    state.phase = 'idle';
    return;
  }
  if (!send({ type: 'mic', value: { state: 'open' } })) {
    state.phase = 'idle';
    return;
  }
  mic.begin();
  state.phase = 'listening';
}

export function micUp(): void {
  if (!micHeld) return;
  micHeld = false;
  if (state.phase !== 'listening') {
    mic.end();
    return;
  }
  mic.end();
  send({ type: 'mic', value: { state: 'closed' } });
  state.phase = 'transcribing';
}

/** Selected node of the shown outline. */
export const selectedNode = computed(() => {
  const root = shownOutline.value;
  const sel = state.doc?.selected;
  return root && sel ? findNode(root, sel) : null;
});

/** Whether `path` lies inside the node a removal would take out. */
export function isRemoved(path: string): boolean {
  const h = highlight.value;
  return !!h && h.mark === 'remove' && isWithin(path, h.path) && segments(h.path).length > 0;
}

/** Selects `path`, or the nearest node above it when it is gone (a removal in the feed). */
export function selectNearest(path: string): void {
  const root = state.doc?.outline;
  if (root) select(nearestExisting(root, path));
}

/** The operator who made the current selection, per presence. */
export const selectedBy = computed(() => {
  const by = state.presence.selected_by;
  return by ? operatorView(by) : null;
});
