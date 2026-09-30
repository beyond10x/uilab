import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { componentsOf, useSites } from './components.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

/**
 * A widget may be named like a primitive kind: only built-in composite kinds are reserved
 * (`widget_named_like_builtin`), so `badge` is a legal widget name, and `component: badge` is an
 * instance of it. The server outline gives a primitive node its primitive name as kind
 * (`component:loan_card/node:due` → kind `badge`, as the unit's own uilab-app test asserts), so a
 * primitive badge and an instance of the widget `badge` carry the same kind. The docs' use sites
 * (`widget_uses`) count only the instance; a primitive is not a use of a widget.
 */
test('a primitive whose kind matches a widget name is not a use site of that widget', () => {
  const root = n('/', 'root', 'document', [
    n(
      'component:badge',
      'component',
      'widget',
      [n('component:badge/node:tag', 'node', 'badge', [], { props: { text: 'args.label' } })],
      { title: 'A toned tag.', props: { params: { label: { type: 'string', required: true } }, arrange: 'row' } },
    ),
    n(
      'component:loan_card',
      'component',
      'widget',
      [n('component:loan_card/node:due', 'node', 'badge', [], { props: { text: 'args.loan.due' } })],
      { title: 'A loan as a card.', props: { params: { loan: { type: 'Loan', required: true } }, arrange: 'column' } },
    ),
    n('page:overview', 'page', 'dashboard_page', [
      n('page:overview/section:latest', 'section', 'badge', [], { props: { args: { label: 'rows.first' } } }),
    ]),
  ]);
  assert.deepEqual(useSites(root, 'badge'), [{ path: 'page:overview/section:latest' }]);
  const badge = componentsOf(root).find((c) => c.name === 'badge')!;
  assert.equal(badge.uses.length, 1, `the widget \`badge\` counts primitives as uses: ${JSON.stringify(badge.uses)}`);
});
