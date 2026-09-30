import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import {
  componentsOf,
  filterComponents,
  isPrimitive,
  paramsOf,
  previewNode,
  sampleArgs,
  sampleValue,
  substitute,
  typeLabel,
  useSites,
} from './components.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

/** The library outline as the server sends it, with two widgets, one used three times. Each
 *  widget node carries its use sites in `props.uses`, as `widget_uses` finds them. */
function doc(): OutlineNode {
  return n('/', 'root', 'document', [
    n('shell:app', 'shell', 'shell'),
    n('nav', 'nav', 'navigation'),
    n(
      'component:loan_card',
      'component',
      'widget',
      [
        n('component:loan_card/node:title', 'node', 'text', [], { props: { text: 'args.loan.title', style: 'heading' } }),
        n('component:loan_card/node:state', 'node', 'state_badge', [], { props: { args: { state: 'args.loan.state' } } }),
        n('component:loan_card/node:due', 'node', 'text', [], { props: { text: 'Due args.loan.due, copies args.copies' } }),
      ],
      {
        title: 'A loan as a card.',
        props: {
          params: {
            loan: { type: 'Loan', required: true },
            compact: { type: 'boolean', default: false },
            copies: { type: 'integer' },
          },
          arrange: 'column',
          uses: [
            { path: 'page:overview/section:latest' },
            { path: 'page:overview/section:list/item:card' },
            { path: 'page:overview/overlay:peek' },
          ],
        },
      },
    ),
    n(
      'component:state_badge',
      'component',
      'widget',
      [n('component:state_badge/node:badge', 'node', 'badge', [], { props: { text: 'args.state' } })],
      {
        title: 'A loan state as a toned badge.',
        props: {
          params: { state: { type: 'string', required: true } },
          arrange: 'row',
          uses: [{ path: 'component:loan_card/node:state' }, { path: 'page:overview/section:list/item:tag', trail: 'choice' }],
        },
      },
    ),
    n('component:unused', 'component', 'widget', [], { title: 'Nothing uses it.', props: { arrange: 'grid', uses: [] } }),
    n('page:overview', 'page', 'dashboard_page', [
      n('page:overview/section:latest', 'section', 'loan_card', [], { props: { args: { loan: 'rows.first' } } }),
      n('page:overview/section:list', 'section', 'collection', [
        n('page:overview/section:list/item:card', 'item', 'loan_card', [], { props: { args: { loan: 'row' } } }),
        n('page:overview/section:list/item:tag', 'item', 'badge', [], {
          props: { choice: { component: 'state_badge', args: { state: 'row.state' } } },
        }),
      ]),
      n('page:overview/overlay:peek', 'overlay', 'drawer loan_card', [], { props: { args: { loan: 'selection' } } }),
    ]),
  ]);
}

test('componentsOf lists every widget with its summary, params, arrangement and body', () => {
  const list = componentsOf(doc());
  assert.deepEqual(
    list.map((c) => [c.name, c.path, c.summary, c.arrange, c.body.map((b) => b.name)]),
    [
      ['loan_card', 'component:loan_card', 'A loan as a card.', 'column', ['title', 'state', 'due']],
      ['state_badge', 'component:state_badge', 'A loan state as a toned badge.', 'row', ['badge']],
      ['unused', 'component:unused', 'Nothing uses it.', 'grid', []],
    ],
  );
  assert.deepEqual(componentsOf(n('/', 'root', 'document')), []);
});

test('paramsOf reads name, type, required and default as declared', () => {
  const card = componentsOf(doc())[0];
  assert.deepEqual(paramsOf(card.node), [
    { name: 'loan', type: 'Loan', typeLabel: 'Loan', required: true, hasDefault: false, default: undefined },
    { name: 'compact', type: 'boolean', typeLabel: 'boolean', required: false, hasDefault: true, default: false },
    { name: 'copies', type: 'integer', typeLabel: 'integer', required: false, hasDefault: false, default: undefined },
  ]);
  assert.deepEqual(paramsOf(componentsOf(doc())[2].node), []);
});

test('an explicit default of null counts as a default', () => {
  const w = n('component:w', 'component', 'widget', [], { props: { params: { note: { type: 'string', default: null } } } });
  const [p] = paramsOf(w);
  assert.equal(p.hasDefault, true);
  assert.equal(p.default, null);
  assert.deepEqual(sampleArgs(paramsOf(w)), { note: null });
});

test('typeLabel writes a constructor map compactly', () => {
  assert.equal(typeLabel('string'), 'string');
  assert.equal(typeLabel({ list: 'Loan' }), 'list<Loan>');
  assert.equal(typeLabel({ enum: ['open', 'late'] }), 'enum<open | late>');
  assert.equal(typeLabel(undefined), '?');
});

test('sample args use a default, else a value shaped by type and name', () => {
  assert.equal(sampleValue({ name: 'title', type: 'string' }), 'title');
  assert.equal(sampleValue({ name: 'count', type: 'number' }), 3);
  assert.equal(sampleValue({ name: 'copies', type: 'integer' }), 3);
  assert.equal(sampleValue({ name: 'compact', type: 'boolean' }), true);
  assert.deepEqual(sampleValue({ name: 'loan', type: 'Loan' }), { name: 'loan' });
  assert.deepEqual(sampleValue({ name: 'member', type: { record: 'Member' } }), { name: 'member' });
  assert.deepEqual(sampleValue({ name: 'loans', type: { list: 'Loan' } }), [{ name: 'loans' }]);
  assert.equal(sampleValue({ name: 'state', type: { enum: ['open', 'late'] } }), 'open');
  const card = componentsOf(doc())[0];
  assert.deepEqual(sampleArgs(paramsOf(card.node)), { loan: { name: 'loan' }, compact: false, copies: 3 });
});

test('args references are replaced from the sample args, whole values keeping their type', () => {
  const args = { loan: { name: 'loan', title: 'Dune' }, copies: 3, compact: false };
  assert.equal(substitute('args.loan.title', args), 'Dune');
  assert.equal(substitute('args.copies', args), 3);
  assert.equal(substitute('args.compact', args), false);
  assert.deepEqual(substitute('args.loan', args), { name: 'loan', title: 'Dune' });
  assert.equal(substitute('Due args.loan.title, copies args.copies.', args), 'Due Dune, copies 3.');
  assert.deepEqual(substitute({ text: 'args.loan.name', parts: ['args.copies', 'plain'], n: 1 }, args), {
    text: 'loan',
    parts: [3, 'plain'],
    n: 1,
  });
  assert.equal(substitute('rows.first', args), 'rows.first');
  assert.equal(substitute('myargs.copies', args), 'myargs.copies');
});

test('a field the sample does not have reads as its own name; an unknown param stays as written', () => {
  const args = { loan: { name: 'loan' } };
  assert.equal(substitute('args.loan.due', args), 'due');
  assert.equal(substitute('args.ghost', args), 'args.ghost');
  assert.equal(substitute('args.ghost.title', args), 'args.ghost.title');
});

test('previewNode substitutes a body node’s props and title and keeps its path', () => {
  const card = componentsOf(doc())[0];
  const args = sampleArgs(paramsOf(card.node));
  const title = previewNode(card.body[0], args);
  assert.equal(title.path, 'component:loan_card/node:title');
  assert.deepEqual(title.props, { text: 'title', style: 'heading' });
  const due = previewNode(card.body[2], args);
  assert.deepEqual(due.props, { text: 'Due due, copies 3' });
  const nested = previewNode(card.body[1], args);
  assert.deepEqual(nested.props, { args: { state: 'state' } });
});

test('isPrimitive tells the nine primitive kinds from composites and widget instances', () => {
  for (const k of ['text', 'badge', 'icon', 'button', 'link', 'input', 'toggle', 'image', 'divider']) {
    assert.equal(isPrimitive(n(`component:w/node:${k}`, 'node', k)), true, k);
  }
  assert.equal(isPrimitive(n('component:w/node:list', 'node', 'collection')), false);
  assert.equal(isPrimitive(n('component:w/node:state', 'node', 'state_badge')), false);
  assert.equal(isPrimitive(n('page:p/section:text', 'section', 'text')), false);
});

test('use sites are every node instantiating the widget, typed or in untyped props, with a count', () => {
  const root = doc();
  assert.deepEqual(useSites(root, 'loan_card'), [
    { path: 'page:overview/section:latest' },
    { path: 'page:overview/section:list/item:card' },
    { path: 'page:overview/overlay:peek' },
  ]);
  assert.deepEqual(useSites(root, 'state_badge'), [
    { path: 'component:loan_card/node:state' },
    { path: 'page:overview/section:list/item:tag', trail: 'choice' },
  ]);
  assert.deepEqual(useSites(root, 'unused'), []);
  const list = componentsOf(root);
  assert.deepEqual(
    list.map((c) => [c.name, c.uses.length]),
    [
      ['loan_card', 3],
      ['state_badge', 2],
      ['unused', 0],
    ],
  );
});

test('use sites are the server’s, in its order with their trails, page headers and page kinds included', () => {
  const uses = [
    { path: 'page:p/section:s', trail: 'toolbar/first' },
    { path: 'page:p', trail: 'header/metrics/due' },
    { path: '/', trail: 'page_kinds/board_page/sections/s' },
  ];
  const root = n('/', 'root', 'document', [
    n('component:chip', 'component', 'widget', [], { props: { uses: [...uses, { trail: 'no path' }, 'junk', null] } }),
    n('component:bare', 'component', 'widget'),
    n('page:p', 'page', 'list_page', [n('page:p/section:other', 'section', 'chip')]),
  ]);
  assert.deepEqual(useSites(root, 'chip'), uses, 'entries without a path are skipped; the outline is not walked');
  assert.deepEqual(useSites(root, 'bare'), []);
  assert.deepEqual(useSites(root, 'ghost'), []);
  assert.equal(componentsOf(root)[0].uses.length, 3);
});

test('a widget named like a primitive counts only its instances, not primitives of that kind', () => {
  const root = n('/', 'root', 'document', [
    n('component:button', 'component', 'widget', [n('component:button/node:b', 'node', 'button', [], { props: { label: 'args.label' } })], {
      props: { params: { label: { type: 'string' } }, uses: [{ path: 'page:p', trail: 'header/actions/go' }] },
    }),
    n('component:card', 'component', 'widget', [n('component:card/node:ok', 'node', 'button', [], { props: { label: 'OK' } })], {
      props: { uses: [] },
    }),
    n('page:p', 'page', 'list_page', [
      n('page:p/section:list', 'section', 'collection', [n('page:p/section:list/item:go', 'item', 'button')]),
    ]),
  ]);
  const button = componentsOf(root).find((c) => c.name === 'button')!;
  assert.deepEqual(button.uses, [{ path: 'page:p', trail: 'header/actions/go' }]);
});

test('search filters by name, summary and param names, case-insensitively', () => {
  const list = componentsOf(doc());
  const names = (q: string) => filterComponents(list, q).map((c) => c.name);
  assert.deepEqual(names(''), ['loan_card', 'state_badge', 'unused']);
  assert.deepEqual(names('   '), ['loan_card', 'state_badge', 'unused']);
  assert.deepEqual(names('BADGE'), ['state_badge']);
  assert.deepEqual(names('toned'), ['state_badge']);
  assert.deepEqual(names('copies'), ['loan_card']);
  assert.deepEqual(names('state'), ['state_badge']);
  assert.deepEqual(names('loan'), ['loan_card', 'state_badge']);
  assert.deepEqual(names('nothing uses'), ['unused']);
  assert.deepEqual(names('zzz'), []);
});
