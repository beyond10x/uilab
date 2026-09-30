import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireRows as Rows,
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
  state: { rows: Record<string, Rows> };
  requestRows(view: string | undefined): void;
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
const { state } = store;

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

/** A page whose charts and collection read the views the cases ask for. */
const outline: OutlineNode = {
  path: '/',
  layer: 'root',
  name: '',
  kind: 'document',
  children: ['draft.LoansPerMonth', 'draft.LoansByState', 'loans.All'].map((view, i) => ({
    path: `page:p/section:s${i}`,
    layer: 'section',
    name: `s${i}`,
    kind: view === 'loans.All' ? 'collection' : 'chart',
    view,
    children: [],
  })),
};

function document(revision: number): ServerMessage {
  return {
    type: 'document',
    value: { document_id: 'd1', file: 'f', selected: '/', outline, findings: [], revision, review: true },
  };
}

function rowsAskedFor(view: string): number {
  return sent.filter((m) => m.type === 'rows' && m.value.view === view).length;
}

function answer(view: string, rows: unknown[]): void {
  handlers!.message({ type: 'rows', value: { view, rows: rows as Rows['rows'] } });
}

handlers!.state('open');
handlers!.message(document(1));

test("requestRows('draft.X') sends a rows message for the draft view", () => {
  store.requestRows('draft.LoansPerMonth');
  assert.equal(rowsAskedFor('draft.LoansPerMonth'), 1);
});

test('the sample rows the server answers are the rows the canvas shows for the draft view', () => {
  answer('draft.LoansPerMonth', [{ month: '2026-05', loans: 10 }]);
  assert.deepEqual(state.rows['draft.LoansPerMonth']?.rows, [{ month: '2026-05', loans: 10 }]);
});

test('a draft view is asked once per document revision, however many composites read it', () => {
  store.requestRows('draft.LoansPerMonth');
  store.requestRows('draft.LoansPerMonth');
  assert.equal(rowsAskedFor('draft.LoansPerMonth'), 1);
});

test("a draft view's sample rows are asked for again when the document moves to a new revision", () => {
  // The server shapes sample rows by the composites that read the view, so an accepted change
  // (a new column, a chart's series) changes them; rows asked before it keep the old shape.
  handlers!.message(document(2));
  store.requestRows('draft.LoansPerMonth');
  assert.equal(rowsAskedFor('draft.LoansPerMonth'), 2);
  assert.deepEqual(
    state.rows['draft.LoansPerMonth']?.rows,
    [{ month: '2026-05', loans: 10 }],
    'the rows shown stay until the new answer comes',
  );
});

test('a fixture view is asked once and not again at a new revision', () => {
  store.requestRows('loans.All');
  answer('loans.All', [{ title: 'Dune' }]);
  handlers!.message(document(3));
  store.requestRows('loans.All');
  assert.equal(rowsAskedFor('loans.All'), 1);
});

test('a draft request left unanswered when the connection drops is asked again on the next one', () => {
  store.requestRows('draft.LoansByState');
  assert.equal(rowsAskedFor('draft.LoansByState'), 1);
  handlers!.state('closed');
  handlers!.state('open');
  assert.equal(rowsAskedFor('draft.LoansByState'), 2);
});

test('a draft view only the waiting proposal preview reads is asked again on reconnect while unanswered', () => {
  const preview: OutlineNode = {
    ...outline,
    children: [
      ...outline.children,
      { path: 'page:p/section:new', layer: 'section', name: 'new', kind: 'metric', view: 'draft.PreviewOnly', children: [] },
    ],
  };
  handlers!.message({
    type: 'proposal',
    value: {
      after: '',
      before: '',
      changed: 'page:p/section:new',
      findings: [],
      op: 'Insert',
      outline: preview,
      proposal_id: 'p1',
      target: 'page:p',
      utterance: 'add a metric of overdue members',
    },
  });
  store.requestRows('draft.PreviewOnly');
  assert.equal(rowsAskedFor('draft.PreviewOnly'), 1);
  handlers!.state('closed');
  handlers!.state('open');
  assert.equal(rowsAskedFor('draft.PreviewOnly'), 2, 'the preview is the outline shown, and it reads the view');
});
