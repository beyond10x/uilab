import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { componentsOf, fixtureRowOf, previewNode, sampleArgs, useSiteTarget, viewForEntity, viewsRead } from './components.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

/**
 * The outline `uilab serve` sends for `examples/library/library.ui.yaml` with one widget
 * (`staff_badge`, param `staff: Staff`) and one shell overlay instantiating it added, as read from
 * `/api/state`. The shell region `account` reads `staff.Me` in the document, but a region node
 * carries neither `view` nor `props` in the outline. `staff.Me` is a declared fixture view.
 */
function servedLibrary(): OutlineNode {
  return n('/', 'root', 'document', [
    n('shell:app', 'shell', 'shell', [
      n('shell:app/region:nav', 'region', 'navigation'),
      n('shell:app/region:account', 'region', 'account_menu'),
      n('shell:app/region:main', 'region', 'page_outlet'),
      n('shell:app/region:overlay', 'region', 'overlay_outlet'),
      n('shell:app/region:notify', 'region', 'notifications'),
      n('shell:app/overlay:whoami', 'overlay', 'dialog staff_badge', [], { props: { args: { staff: 'me' } } }),
    ]),
    n('nav', 'nav', 'navigation', [], { props: { home: 'overview' } }),
    n(
      'component:staff_badge',
      'component',
      'widget',
      [
        n('component:staff_badge/node:name', 'node', 'text', [], { props: { text: 'args.staff.name' } }),
        n('component:staff_badge/node:email', 'node', 'text', [], { props: { text: 'args.staff.email' } }),
      ],
      {
        title: 'Who is signed in',
        props: { params: { staff: { type: 'Staff', required: true } }, arrange: 'row', uses: [{ path: 'shell:app/overlay:whoami' }] },
      },
    ),
    n('page:overview', 'page', 'dashboard_page', [
      n('page:overview/section:on_loan', 'section', 'metric', [], { view: 'loans.Summary' }),
      n('page:overview/section:recent', 'section', 'collection', [], { view: 'loans.All' }),
    ], { props: { shell: 'app' } }),
    n('page:members', 'page', 'list_page', [n('page:members/section:list', 'section', 'collection', [], { view: 'members.All' })], {
      props: { shell: 'app' },
    }),
  ]);
}

/** The fixture rows the server answers for the library's declared views. */
const ROWS = {
  'staff.Me': { view: 'staff.Me', rows: [{ id: 's-1', name: 'Example Librarian', email: 'librarian@example.com' }] },
  'members.All': { view: 'members.All', rows: [{ name: 'Robin Example', joined: '2024-03-01', loans: 1, standing: 'good' }] },
};

test('a use site in a shell overlay opens that overlay in the UI tab', () => {
  const widget = componentsOf(servedLibrary()).find((c) => c.name === 'staff_badge')!;
  assert.deepEqual(widget.uses, [{ path: 'shell:app/overlay:whoami' }]);
  const target = useSiteTarget(widget.uses[0]);
  assert.equal(target.overlay, 'shell:app/overlay:whoami', 'a shell overlay shows on every page of its shell; the link must open it');
});

test('an entity whose declared fixture view is read only by a shell region still previews with its fixture row', () => {
  const root = servedLibrary();
  const widget = componentsOf(root).find((c) => c.name === 'staff_badge')!;
  const args = sampleArgs(widget.params, fixtureRowOf(viewsRead(root), ROWS));
  const shown = widget.body.map((b) => (previewNode(b, args).props as Record<string, unknown>).text);
  assert.deepEqual(shown, ['Example Librarian', 'librarian@example.com']);
});

test('a multi-word entity maps to its snake_case view', () => {
  assert.equal(viewForEntity('LoanRequest', ['members.All', 'loan_requests.All']), 'loan_requests.All');
});
