import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';
import type { FeedEntry } from './collab.ts';

// The store is browser code: this node-typed test reaches it through a specifier the type
// checker does not follow, and names the part of it the cases drive.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: { phase: string; thinkingTarget: string | null; feed: FeedEntry[]; inFlight: Record<string, string> };
  say(text: string): void;
  connect(t: {
    start(h: TransportHandlers): void;
    send(m: ClientMessage): boolean;
    sendBinary(frame: ArrayBuffer): boolean;
  }): void;
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Tester' },
  localStorage: { getItem: () => null, setItem: () => undefined },
};
const storeModule: string = '../store.ts';
const store = (await import(storeModule)) as StoreApi;
const { state } = store;

let handlers: TransportHandlers | null = null;
store.connect({
  start: (h) => {
    handlers = h;
  },
  send: () => true,
  sendBinary: () => true,
});

const outline: OutlineNode = {
  path: '/',
  layer: 'root',
  name: '',
  kind: 'document',
  children: [
    { path: 'page:loans', layer: 'page', name: 'loans', kind: 'list_page', children: [] },
    { path: 'page:members', layer: 'page', name: 'members', kind: 'list_page', children: [] },
  ],
};

handlers!.state('open');
handlers!.message({
  type: 'document',
  value: { document_id: 'd1', file: 'f', selected: 'page:loans', outline, findings: [], revision: 1, review: true },
});
handlers!.message({
  type: 'presence',
  value: {
    operators: [
      { id: 'ws-1', name: 'Tester', kind: 'human', last_seen_ms: 0 },
      { id: 'agent', name: 'agent', kind: 'agent', last_seen_ms: 0 },
    ],
  },
});

function moved(navigate_only: boolean, to: string): ServerMessage {
  return {
    type: 'moved',
    value: {
      by: 'ws-1',
      selected_by: 'agent',
      from: 'page:loans',
      to,
      reason: `the instruction names ${to}`,
      navigate_only,
      utterance: 'u',
    },
  };
}

test('a navigation-only move ends the wait and puts the reason in the feed, by the agent', () => {
  store.say('go to the members page');
  handlers!.message({ type: 'thinking', value: { by: 'ws-1', target: 'page:loans' } });
  assert.equal(state.phase, 'thinking');
  handlers!.message(moved(true, 'page:members'));
  assert.equal(state.phase, 'idle');
  assert.deepEqual(state.inFlight, {});
  const entry = state.feed[0];
  assert.equal(entry.kind, 'moved');
  assert.equal(entry.by, 'agent');
  assert.equal(entry.path, 'page:members');
  assert.equal(entry.text, 'the instruction names page:members');
});

test('a move that asks again keeps the wait and thinks at the new target', () => {
  store.say('create a new page in the sidebar for overdue loans');
  handlers!.message({ type: 'thinking', value: { by: 'ws-1', target: 'page:loans' } });
  handlers!.message(moved(false, '/'));
  assert.equal(state.phase, 'thinking');
  assert.equal(state.thinkingTarget, '/');
  assert.deepEqual(state.inFlight, { 'ws-1': '/' });
  assert.equal(state.feed[0].kind, 'moved');
  assert.equal(state.feed[0].what, 'retarget');
});
