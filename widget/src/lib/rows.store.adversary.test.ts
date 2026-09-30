import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireRows as Rows,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';

// Adversary cases for story:draft-sample-rows, driving the real store through a fake transport,
// the way `rows.store.test.ts` does.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: { rows: Record<string, Rows>; revision: number | null };
  requestRows(view: string | undefined): void;
  connect(t: {
    start(h: TransportHandlers): void;
    send(m: ClientMessage): boolean;
    sendBinary(frame: ArrayBuffer): boolean;
  }): void;
}

(globalThis as unknown as { window: unknown }).window = {
  location: { search: '?name=Adversary' },
  localStorage: { getItem: () => null, setItem: () => undefined },
};
const storeModule: string = '../store.ts';
const store = (await import(storeModule)) as StoreApi;
const { state } = store;

let handlers: TransportHandlers | null = null;
let open = false;
const sent: ClientMessage[] = [];
store.connect({
  start: (h) => {
    handlers = h;
  },
  send: (m) => {
    if (!open) return false;
    sent.push(m);
    return true;
  },
  sendBinary: () => true,
});

/** A page whose composites read `views`, one chart each. */
function outlineReading(views: string[]): OutlineNode {
  return {
    path: '/',
    layer: 'root',
    name: '',
    kind: 'document',
    children: views.map((view, i) => ({
      path: `page:p/section:s${i}`,
      layer: 'section',
      name: `s${i}`,
      kind: 'chart',
      view,
      children: [],
    })),
  };
}

function document(revision: number, reads: string[]): ServerMessage {
  return {
    type: 'document',
    value: {
      document_id: 'd1',
      file: 'f',
      selected: '/',
      outline: outlineReading(reads),
      findings: [],
      revision,
      review: true,
    },
  };
}

function asked(view: string): number {
  return sent.filter((m) => m.type === 'rows' && m.value.view === view).length;
}

function answer(view: string, rows: unknown[]): void {
  handlers!.message({ type: 'rows', value: { view, rows: rows as Rows['rows'] } });
}

function connectionOpen(): void {
  open = true;
  handlers!.state('open');
}

function connectionDrops(): void {
  open = false;
  handlers!.state('closed');
}

connectionOpen();
handlers!.message(document(1, []));

test('a revision bump with 20 draft views read by 3 composites each asks each view once, not once per composite', () => {
  const views = Array.from({ length: 20 }, (_, i) => `draft.Storm${i}`);
  handlers!.message(document(10, views));
  for (let pass = 0; pass < 3; pass++) for (const v of views) store.requestRows(v);
  for (const v of views) answer(v, [{ name: 'a', value: 1 }]);
  handlers!.message(document(11, views));
  for (let pass = 0; pass < 3; pass++) for (const v of views) store.requestRows(v);
  assert.deepEqual(
    views.map(asked),
    views.map(() => 2),
    'one request per view per revision',
  );
  handlers!.message(document(11, views));
  for (const v of views) store.requestRows(v);
  assert.deepEqual(views.map(asked), views.map(() => 2), 'the same revision again asks nothing');
});

test('a draft view a proposal preview reads, asked at the current revision, is asked again once the proposal is accepted', () => {
  handlers!.message(document(20, []));
  store.requestRows('draft.MembersWithOverdue');
  answer('draft.MembersWithOverdue', [{ name: 'Name 1', value: 3 }]);
  assert.equal(asked('draft.MembersWithOverdue'), 1);
  handlers!.message(document(21, ['draft.MembersWithOverdue']));
  store.requestRows('draft.MembersWithOverdue');
  assert.equal(asked('draft.MembersWithOverdue'), 2);
});

test('a draft re-request in flight when the connection drops is asked again on the next connection', () => {
  // Rows for the view are on screen from revision 30; revision 31 changes a composite that reads
  // it (a new column) and the re-request goes out; the connection drops before the answer. The
  // server is still at revision 31 when the browser comes back.
  handlers!.message(document(30, ['draft.InFlight']));
  store.requestRows('draft.InFlight');
  answer('draft.InFlight', [{ month: '2026-05' }]);
  handlers!.message(document(31, ['draft.InFlight']));
  store.requestRows('draft.InFlight');
  assert.equal(asked('draft.InFlight'), 2, 'the re-request for revision 31 went out');
  assert.deepEqual(state.rows['draft.InFlight']?.rows, [{ month: '2026-05' }], 'revision 30 rows on screen');

  connectionDrops();
  connectionOpen();
  handlers!.message(document(31, ['draft.InFlight']));
  store.requestRows('draft.InFlight');
  assert.equal(
    asked('draft.InFlight'),
    3,
    'the answer for revision 31 never came, so the rows on screen still have the revision 30 shape',
  );
});

test('a draft view no composite reads any more is not asked for on reconnect', () => {
  handlers!.message(document(40, ['draft.Gone']));
  store.requestRows('draft.Gone');
  answer('draft.Gone', [{ name: 'a', value: 1 }]);
  assert.equal(asked('draft.Gone'), 1);
  // Revision 41 removes the only composite that read it.
  handlers!.message(document(41, []));
  connectionDrops();
  connectionOpen();
  assert.equal(asked('draft.Gone'), 1, 'nothing on the canvas reads draft.Gone');
});
