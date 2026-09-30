import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { useSiteTarget, viewForEntity, viewsRead } from './components.ts';

function n(path: string, layer: string, kind: string, children: OutlineNode[] = [], extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children, ...extra };
}

test('an irregular plural does not stop the regular one from matching: `Person` still maps to `persons.All`', () => {
  assert.equal(viewForEntity('Person', ['persons.All']), 'persons.All', 'b0acee8 mapped Person to persons.All; e3d970d maps it to nothing');
  assert.equal(viewForEntity('Child', ['child.All']), 'child.All');
  assert.equal(viewForEntity('Persons', ['persons.All']), 'persons.All');
});

test('a draft view a shell region reads stays out of viewsRead, and its non-draft siblings stay in', () => {
  const root = n('/', 'root', 'document', [
    n('shell:app', 'shell', 'shell', [
      n('shell:app/region:account', 'region', 'account_menu', [], { view: 'draft.Profile' }),
      n('shell:app/region:assistant', 'region', 'assistant', [], { view: 'staff.Me' }),
    ]),
  ]);
  assert.deepEqual(viewsRead(root), ['staff.Me']);
});

test('a use site in a shell region (no overlay) has no page and no overlay to show', () => {
  assert.deepEqual(useSiteTarget({ path: 'shell:app/region:account' }), {
    select: 'shell:app/region:account',
    page: null,
    overlay: null,
  });
});
