import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireOutlineNode as OutlineNode,
  UilabWireServerMessage as ServerMessage,
} from '../generated/types.ts';
import { componentsOf } from './components.ts';

// The store is browser code: this node-typed test reaches it through a specifier the type
// checker does not follow, as goalstore.test.ts does, and names the part of it the cases drive.
type ConnState = 'connecting' | 'open' | 'closed';
interface TransportHandlers {
  message(message: ServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}
interface StoreApi {
  state: { proposal: unknown };
  shownOutline: { value: OutlineNode | null };
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

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

/** The outline the server sends for the library with `loan_card` used once, at `section:latest`. */
function outline(): OutlineNode {
  return n('/', 'root', 'document', [
    n('component:loan_card', 'component', 'widget', [n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title' } })], {
      title: 'A loan as a card.',
      props: { params: { loan: { type: 'Loan', required: true } }, arrange: 'column', uses: [{ path: 'page:overview/section:latest' }] },
    }),
    n('page:overview', 'page', 'dashboard_page', [
      n('page:overview/section:latest', 'section', 'loan_card', [], { props: { args: { loan: 'rows.first' } } }),
    ]),
  ]);
}

function reset(): void {
  handlers!.state('open');
  handlers!.message({
    type: 'document',
    value: { document_id: 'd1', file: 'f', selected: '/', outline: outline(), findings: [], revision: 1, review: true },
  });
  store.state.proposal = null;
  sent.length = 0;
}

function loanCardUses(): string[] {
  const root = store.shownOutline.value;
  assert.ok(root, 'the store shows an outline');
  const card = componentsOf(root).find((c) => c.name === 'loan_card');
  assert.ok(card, 'loan_card is listed');
  return card.uses.map((u) => u.path);
}

/**
 * An accepted insert of a section goes out as a `changed` delta carrying only the section's
 * subtree (`crates/uilab-app/src/app.rs` `announce_change`: only a batch, a page, a menu section or
 * a shell sends a full document; `outline_at` is the changed path's subtree). The widget's
 * `props.uses` sits on `component:loan_card`, which the delta does not re-send, so after the store
 * applies it the Components tab must still list the new instance.
 */
test('a section inserted by a delta that instantiates a widget is listed among its uses', () => {
  reset();
  handlers!.message({
    type: 'changed',
    value: {
      by: 'ws-2',
      revision: 2,
      op: 'Insert',
      changed: 'page:overview/section:more',
      parent: 'page:overview',
      node: n('page:overview/section:more', 'section', 'loan_card', [], { props: { args: { loan: 'rows.first' } } }),
      findings: [],
    },
  });
  assert.equal(
    sent.some((m) => m.type === 'resync'),
    false,
    'the delta fits the outline, so the store applies it without a resync',
  );
  assert.deepEqual(loanCardUses(), ['page:overview/section:latest', 'page:overview/section:more']);
});

/**
 * The same through a removal: the delta drops `section:latest`, the only instance, and the tab
 * must stop listing it, or it shows the widget as used while a removal of it is now admitted.
 */
test('a section removed by a delta is no longer listed among the widget’s uses', () => {
  reset();
  handlers!.message({
    type: 'changed',
    value: {
      by: 'ws-2',
      revision: 2,
      op: 'Remove',
      changed: 'page:overview/section:latest',
      parent: 'page:overview',
      findings: [],
    },
  });
  assert.equal(
    sent.some((m) => m.type === 'resync'),
    false,
    'the delta fits the outline, so the store applies it without a resync',
  );
  assert.deepEqual(loanCardUses(), []);
});
