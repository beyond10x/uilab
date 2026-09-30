import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireRows as Rows,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';

// Adversary pass 2 on story:draft-sample-rows. The server shapes a draft view's sample rows by
// the composites of the outline the canvas shows: the document, or while a proposal waits the
// document that proposal would make (`App::preview`). The browser keys its requests by document
// revision alone, and a proposal shown, replaced or rejected leaves the revision where it was.

type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: { rows: Record<string, Rows>; proposal: unknown };
  requestRows(view: string | undefined): void;
  reject(): void;
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

function section(name: string, kind: string, view: string): OutlineNode {
  return { path: `page:p/section:${name}`, layer: 'section', name, kind, view, children: [] };
}

/** The document: charts over two draft views. */
const outline: OutlineNode = {
  path: '/',
  layer: 'root',
  name: '',
  kind: 'document',
  children: [section('per_month', 'chart', 'draft.LoansPerMonth'), section('by_state', 'chart', 'draft.LoansByState')],
};

function document(revision: number): ServerMessage {
  return {
    type: 'document',
    value: { document_id: 'd1', file: 'f', selected: '/', outline, findings: [], revision, review: true },
  };
}

function propose(id: string, children: OutlineNode[]): void {
  handlers!.message({
    type: 'proposal',
    value: {
      after: '',
      before: '',
      changed: children.at(-1)!.path,
      findings: [],
      op: 'Insert',
      outline: { ...outline, children },
      proposal_id: id,
      target: 'page:p',
      utterance: `proposal ${id}`,
    },
  });
  assert.equal((state.proposal as { proposal_id: string } | null)?.proposal_id, id);
}

/** The operator rejects the waiting proposal; the server answers with the document at the same
 *  revision (`Client::Reject` → `send_document`, no revision bump). */
function rejectAt(revision: number): void {
  store.reject();
  handlers!.message(document(revision));
  assert.equal(state.proposal, null);
}

function rowsAskedFor(view: string): number {
  return sent.filter((m) => m.type === 'rows' && m.value.view === view).length;
}

function answer(view: string, rows: unknown[]): void {
  handlers!.message({ type: 'rows', value: { view, rows: rows as Rows['rows'] } });
}

handlers!.state('open');
handlers!.message(document(1));

test('a proposal preview that reads a draft view the document already reads, through another field, asks for its rows again', () => {
  store.requestRows('draft.LoansPerMonth');
  answer('draft.LoansPerMonth', [{ month: '2026-05', loans: 10 }]);
  assert.equal(rowsAskedFor('draft.LoansPerMonth'), 1);
  // The preview adds a metric over the same view reading `members`; the rows in hand have no
  // `members`, so the metric shows a dash until the server answers from the preview.
  propose('pa', [...outline.children, section('overdue_members', 'metric', 'draft.LoansPerMonth')]);
  store.requestRows('draft.LoansPerMonth');
  assert.equal(
    rowsAskedFor('draft.LoansPerMonth'),
    2,
    'the rows in hand were shaped by the document, not by the preview now shown',
  );
  rejectAt(1);
});

test('rows answered for a preview are asked again when another proposal replaces it', () => {
  propose('pb1', [...outline.children, section('overdue', 'metric', 'draft.Overdue')]);
  store.requestRows('draft.Overdue');
  answer('draft.Overdue', [{ members: 10 }]);
  assert.equal(rowsAskedFor('draft.Overdue'), 1);
  // A refinement: the next instruction replaces the waiting proposal (`propose` rejects the
  // pending one) with a collection over the same view, naming other columns.
  propose('pb2', [...outline.children, section('overdue', 'collection', 'draft.Overdue')]);
  store.requestRows('draft.Overdue');
  assert.equal(
    rowsAskedFor('draft.Overdue'),
    2,
    'the rows in hand were shaped by the replaced proposal',
  );
  rejectAt(1);
});

test('rows first answered while a preview was shown are asked again once the proposal is rejected', () => {
  // The document's chart over draft.LoansByState was never on screen (another page) until the
  // proposal revealed it, replaced by a metric: the answer is shaped by the preview.
  propose('pc', [outline.children[0]!, section('by_state', 'metric', 'draft.LoansByState')]);
  store.requestRows('draft.LoansByState');
  answer('draft.LoansByState', [{ members: 3 }]);
  assert.equal(rowsAskedFor('draft.LoansByState'), 1);
  rejectAt(1);
  store.requestRows('draft.LoansByState');
  assert.equal(
    rowsAskedFor('draft.LoansByState'),
    2,
    'the chart the document shows again gets rows without its x or series',
  );
});
