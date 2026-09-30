import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireGoal as Goal,
  UilabWireGoalStep as GoalStep,
  UilabWireOutlineNode as OutlineNode,
  UilabWireServerMessage as ServerMessage,
  UilabWireStepStatus as StepStatus,
} from '../generated/types.ts';
import { stopMessage } from './goal.ts';

// The store is browser code: this node-typed test reaches it through a specifier the type
// checker does not follow, and names the part of it the cases drive.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: {
    goal: Goal | null;
    proposal: { proposal_id: string } | null;
    deciding: boolean;
    phase: string;
    inFlight: Record<string, string>;
    notice: unknown;
  };
  agentActions: { value: { op: { id: string } }[] };
  connect(t: {
    start(h: TransportHandlers): void;
    send(m: ClientMessage): boolean;
    sendBinary(frame: ArrayBuffer): boolean;
  }): void;
}

// The store reads the page's query string and local storage when it loads.
(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Tester' },
  localStorage: { getItem: () => null, setItem: () => undefined },
};
const storeModule: string = '../store.ts';
const store = (await import(storeModule)) as StoreApi;
const { state, agentActions } = store;

let handlers: TransportHandlers | null = null;
const sent: ClientMessage[] = [];
store.connect({
  start: (h) => {
    handlers = h;
  },
  send: (m) => {
    sent.push(m);
    return true;
  },
  sendBinary: () => true,
});

function conn(s: ConnState): void {
  handlers!.state(s);
}

function deliver(msg: ServerMessage): void {
  handlers!.message(msg);
}

const outline: OutlineNode = {
  path: '/',
  layer: 'root',
  name: '',
  kind: 'document',
  children: [{ path: 'page:members', layer: 'page', name: 'members', kind: 'page', children: [] }],
};

function documentMsg(document_id = 'd1'): ServerMessage {
  return {
    type: 'document',
    value: { document_id, file: 'f', selected: 'page:members', outline, findings: [], revision: 1, review: true },
  };
}

/** Tester is this browser's human operator (ws-1); Bot is an API agent (api-1). */
function presenceMsg(): ServerMessage {
  return {
    type: 'presence',
    value: {
      operators: [
        { id: 'ws-1', kind: 'human', name: 'Tester', last_seen_ms: 0 },
        { id: 'api-1', kind: 'agent', name: 'Bot', last_seen_ms: 0 },
      ],
    },
  };
}

function step(status: StepStatus, proposal_id?: string): GoalStep {
  return { instruction: 'add a list', target: 'page:members', why: 'w', status, proposal_id };
}

function goalMsg(over: Partial<Goal>): ServerMessage {
  return {
    type: 'goal',
    value: { goal_id: 'goal-1', by: 'ws-1', text: 'build out the member area', state: 'planning', steps: [], ...over },
  };
}

function reset(): void {
  conn('open');
  deliver(documentMsg());
  deliver(presenceMsg());
  state.goal = null;
  state.proposal = null;
  state.deciding = false;
  state.phase = 'idle';
  state.inFlight = {};
  state.notice = null;
}

test('stopping a goal while its step thinks returns the owner browser to idle', () => {
  reset();
  deliver(goalMsg({ state: 'running', current: 0, steps: [step('thinking'), step('pending')] }));
  deliver({ type: 'thinking', value: { target: 'page:members', by: 'ws-1' } });
  assert.equal(state.phase, 'thinking', 'precondition: the step is being thought about');
  // What the server sends on stop_goal while the step thinks: the goal, stopped, and nothing
  // else. The late answer is discarded (app.rs, Cmd::Proposed) without a message.
  deliver(goalMsg({ state: 'stopped', steps: [step('pending'), step('pending')] }));
  assert.equal(state.phase, 'idle', 'the status line still says "thinking about page:members…"');
  assert.ok(!('ws-1' in state.inFlight), 'the operator is still marked as acting on the node');
});

test('stopping an agent goal while its step thinks drops the agent banner', () => {
  reset();
  deliver(goalMsg({ by: 'api-1', state: 'running', current: 0, steps: [step('thinking')] }));
  deliver({ type: 'thinking', value: { target: 'page:members', by: 'api-1' } });
  assert.equal(agentActions.value.length, 1, 'precondition: the banner shows the agent operating');
  deliver(goalMsg({ by: 'api-1', state: 'stopped', steps: [step('pending')] }));
  assert.deepEqual(
    agentActions.value.map((a) => a.op.id),
    [],
    'the banner still says the agent is operating on page:members after its goal stopped',
  );
});

test('a goal the server does not hold after a reconnect cannot be stopped from the panel', () => {
  reset();
  deliver(goalMsg({ state: 'running', current: 0, steps: [step('proposed', 'p1'), step('pending')] }));
  // The server restarts: the socket drops and reconnects, and the new session sends its document
  // and presence and no goal message, because it holds no goal (app.rs, Cmd::Connected).
  conn('closed');
  conn('open');
  deliver(documentMsg('d2'));
  deliver(presenceMsg());
  assert.equal(
    stopMessage(state.goal),
    null,
    'the panel still shows a running goal with a Stop button; stop_goal is refused with wrong_state',
  );
});

test('starting a goal closes the card of the proposal the server rejected for it', () => {
  reset();
  deliver({
    type: 'proposal',
    value: {
      proposal_id: 'p0',
      by: 'ws-1',
      op: 'Insert',
      target: 'page:members',
      changed: 'page:members/section:list',
      utterance: 'add a list',
      before: '',
      after: '',
      findings: [],
      outline,
    },
  });
  assert.equal(state.proposal?.proposal_id, 'p0', 'precondition: the card is open');
  // start_goal (app.rs) rejects the waiting proposal and broadcasts only the planning goal.
  deliver(goalMsg({ state: 'planning' }));
  assert.equal(
    state.proposal,
    null,
    'the card of the rejected proposal stays open; Accept on it is refused "that proposal is not waiting"',
  );
});
