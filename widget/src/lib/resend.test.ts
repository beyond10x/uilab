import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireProposalShown as ProposalShown,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';

// The store is browser code: this node-typed test reaches it through a specifier the type
// checker does not follow, and names the part of it the cases drive.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: {
    proposal: { proposal_id: string } | null;
    deciding: boolean;
    feed: { kind: string }[];
    phase: string;
    inFlight: Record<string, string>;
    notice: unknown;
  };
  agentActions: { value: { op: { id: string } }[] };
  accept(): void;
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

const outline: OutlineNode = { path: '/', layer: 'root', name: '', kind: 'document', children: [] };

const p1: ProposalShown = {
  proposal_id: 'p1',
  by: 'api-1',
  op: 'Insert',
  target: 'page:members',
  changed: 'page:members/section:list',
  utterance: 'step 1',
  before: '',
  after: '',
  findings: [],
  outline,
};

test('the waiting proposal re-sent for a browser that connects changes nothing in one already connected', () => {
  handlers!.state('open');
  handlers!.message({
    type: 'presence',
    value: {
      operators: [
        { id: 'ws-1', kind: 'human', name: 'Tester', last_seen_ms: 0 },
        { id: 'api-1', kind: 'agent', name: 'Bot', last_seen_ms: 0 },
      ],
    },
  });
  handlers!.message({ type: 'proposal', value: p1 });
  store.accept();
  assert.equal(state.deciding, true, 'precondition: this browser pressed Accept');
  const proposalsBefore = state.feed.filter((e) => e.kind === 'proposal').length;
  assert.equal(proposalsBefore, 1, 'precondition: one proposal in the feed');

  // Another browser connects: app.rs Cmd::Connected broadcasts the same proposal message to
  // every subscriber, this browser included.
  handlers!.message({ type: 'proposal', value: p1 });

  assert.equal(
    state.feed.filter((e) => e.kind === 'proposal').length,
    1,
    'the activity feed lists the same proposal twice',
  );
  assert.equal(state.deciding, true, 'the card forgot that Accept was pressed and offers it again');
});

function ended(by: string): ServerMessage {
  return {
    type: 'goal',
    value: { goal_id: 'goal-1', by, text: 't', state: 'done', current: undefined, steps: [] },
  };
}

test('an ended goal re-sent for a browser that connects does not end an instruction its operator is now working on', () => {
  handlers!.state('open');
  handlers!.message({
    type: 'presence',
    value: {
      operators: [
        { id: 'ws-1', kind: 'human', name: 'Tester', last_seen_ms: 0 },
        { id: 'api-1', kind: 'agent', name: 'Bot', last_seen_ms: 0 },
      ],
    },
  });
  state.proposal = null;
  state.deciding = false;
  state.phase = 'idle';
  state.inFlight = {};
  // This browser's goal is over; it now gives one instruction and waits on it.
  handlers!.message(ended('ws-1'));
  store.say('add a search box');
  handlers!.message({ type: 'thinking', value: { target: 'page:members', by: 'ws-1' } });
  assert.equal(state.phase, 'thinking', 'precondition: this browser waits on its instruction');

  // Another browser connects: app.rs Cmd::Connected re-broadcasts the latest goal, ended.
  handlers!.message(ended('ws-1'));
  assert.equal(state.phase, 'thinking', 'the status line went idle while the instruction is still being worked on');
});

test('an ended goal re-sent for a browser that connects keeps the banner of its agent now working on an instruction', () => {
  state.inFlight = {};
  handlers!.message(ended('api-1'));
  handlers!.message({ type: 'thinking', value: { target: 'page:members', by: 'api-1' } });
  assert.deepEqual(
    store.agentActions.value.map((a) => a.op.id),
    ['api-1'],
    'precondition: the agent banner shows the agent at work',
  );
  handlers!.message(ended('api-1'));
  assert.deepEqual(
    store.agentActions.value.map((a) => a.op.id),
    ['api-1'],
    'the agent banner went away while the agent is still working on its instruction',
  );
});
