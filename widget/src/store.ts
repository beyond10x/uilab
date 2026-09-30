import { computed, reactive, shallowReactive } from 'vue';
import type {
  UilabWireDocumentState as DocumentState,
  UilabWireOutlineNode as OutlineNode,
  UilabWireProposalShown as ProposalShown,
  UilabWireRows as Rows,
  UilabWireServerMessage as ServerMessage,
  UilabWireTranscript as Transcript,
} from './generated/types.ts';
import { findNode, homePage, isDraftView, isWithin, lineage, overlayOf, pageOf, pagesOf, segments } from './lib/outline.ts';
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
});

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

/** CSS classes marking a node as selected, changed by the proposal, or being thought about. */
export function marks(path: string): Record<string, boolean> {
  const h = highlight.value;
  return {
    selected: state.doc?.selected === path,
    [`hl-${h?.mark}`]: !!h && h.path === path,
    thinking: state.phase === 'thinking' && state.thinkingTarget === path,
  };
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

function onMessage(msg: ServerMessage): void {
  switch (msg.type) {
    case 'document': {
      const before = state.doc?.selected;
      state.doc = msg.value;
      if (state.deciding) {
        state.proposal = null;
        state.deciding = false;
      }
      if (msg.value.selected !== before) reveal(msg.value.selected);
      break;
    }
    case 'transcript':
      state.transcript = msg.value;
      state.typed = null;
      if (state.phase === 'transcribing') state.phase = 'idle';
      break;
    case 'thinking':
      state.phase = 'thinking';
      state.thinkingTarget = msg.value.target;
      break;
    case 'proposal': {
      state.proposal = msg.value;
      state.deciding = false;
      state.phase = 'idle';
      state.notice = null;
      state.openOverlay = overlayOf(msg.value.changed);
      reveal(msg.value.changed);
      break;
    }
    case 'refused':
      state.notice = { kind: 'refused', check: msg.value.check, message: msg.value.message };
      state.phase = 'idle';
      state.deciding = false;
      break;
    case 'failed':
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
    // Requests that were never answered are asked again on the new connection.
    for (const view of rowsRequested) if (!state.rows[view]) rowsRequested.delete(view);
    for (const view of pendingViews) requestRows(view);
  } else if (conn === 'closed') {
    if (state.phase === 'listening') mic.end();
    if (state.phase !== 'idle') state.phase = 'idle';
    micHeld = false;
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
  if (send({ type: 'say', value: { text: t } })) {
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
