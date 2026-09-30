import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';

// The store is browser code: this node-typed test reaches it through a specifier the type
// checker does not follow (as goalstore.test.ts does), and names the part of it the cases drive.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: {
    proposal: { proposal_id: string } | null;
    proposalBase: OutlineNode | null;
    deciding: boolean;
  };
  proposalMarks: { value: ReadonlyMap<string, string> };
  shownOutline: { value: OutlineNode | null };
  accept(): void;
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

const deliver = (msg: ServerMessage) => handlers!.message(msg);
const conn = (s: ConnState) => handlers!.state(s);

function n(path: string, layer: string, kind: string, children: OutlineNode[] = []): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children };
}

const sec = (page: string, name: string) => n(`page:${page}/section:${name}`, 'section', 'collection');
const root = (...pages: OutlineNode[]) => n('/', 'root', 'document', pages);

function documentMsg(outline: OutlineNode, revision: number): ServerMessage {
  return { type: 'document', value: { document_id: 'd1', file: 'f', selected: '/', outline, findings: [], revision, review: true } };
}

function proposalMsg(id: string, outline: OutlineNode, changed: string): ServerMessage {
  return {
    type: 'proposal',
    value: { proposal_id: id, by: 'agent-1', op: 'Insert', changed, target: changed, outline, before: '', after: '', findings: [], utterance: 'u' },
  };
}

const marks = () => Object.fromEntries([...store.proposalMarks.value.entries()].sort(([a], [b]) => a.localeCompare(b)));
const shownPaths = () => store.shownOutline.value?.children.map((c) => c.path);

// D0: members with a list; loans empty. P1 inserts page:members/section:card.
const d0 = root(n('page:members', 'page', 'list_page', [sec('members', 'list')]), n('page:loans', 'page', 'list_page'));
const p1 = root(n('page:members', 'page', 'list_page', [sec('members', 'list'), sec('members', 'card')]), n('page:loans', 'page', 'list_page'));
const p1Marks = { 'page:members/section:card': 'added' };

test('a proposal is marked against the document it arrived with', () => {
  conn('open');
  deliver(documentMsg(d0, 1));
  deliver(proposalMsg('p1', p1, 'page:members/section:card'));
  assert.equal(state.proposalBase, d0);
  assert.deepEqual(marks(), p1Marks);
});

test('an incremental change by someone else while the card waits is not marked as the proposal', () => {
  deliver({
    type: 'changed',
    value: { revision: 2, by: 'op-2', op: 'Insert', changed: 'page:loans/section:due', parent: 'page:loans', node: sec('loans', 'due'), findings: [] },
  });
  assert.ok(state.proposal, 'the card stays open');
  assert.deepEqual(marks(), p1Marks);
  assert.deepEqual(shownPaths(), ['page:members', 'page:loans']);
});

test('the same proposal sent again after a reconnect keeps the base it arrived with', () => {
  conn('closed');
  conn('open');
  // A snapshot that no longer has the loans page, then the waiting card again.
  deliver(documentMsg(root(n('page:members', 'page', 'list_page', [sec('members', 'list')])), 3));
  deliver(proposalMsg('p1', p1, 'page:members/section:card'));
  assert.equal(state.proposal?.proposal_id, 'p1');
  assert.equal(state.proposalBase, d0);
  assert.deepEqual(marks(), p1Marks);
});

test('a refused accept leaves the card, and with it the base', () => {
  store.accept();
  assert.equal(state.deciding, true);
  deliver({ type: 'refused', value: { check: 'stale', message: 'the document changed since the proposal' } });
  assert.equal(state.deciding, false);
  assert.ok(state.proposal);
  assert.equal(state.proposalBase, d0);
});

test('a second proposal replaces the base with the document current when it arrives', () => {
  const d3 = root(n('page:members', 'page', 'list_page', [sec('members', 'list')]));
  const p2 = root(n('page:members', 'page', 'list_page', [sec('members', 'list')]), n('page:fines', 'page', 'list_page'));
  deliver(proposalMsg('p2', p2, 'page:fines'));
  assert.deepEqual(state.proposalBase, d3);
  assert.deepEqual(marks(), { 'page:fines': 'added' });
});

test('an accepted card closes and drops its base', () => {
  store.accept();
  deliver(documentMsg(root(n('page:members', 'page', 'list_page', [sec('members', 'list')]), n('page:fines', 'page', 'list_page')), 4));
  assert.equal(state.proposal, null);
  assert.equal(state.proposalBase, null);
  assert.equal(store.proposalMarks.value.size, 0);
});
