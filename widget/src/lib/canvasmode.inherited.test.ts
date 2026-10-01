import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { nodeActions } from './canvasmode.ts';

// story:essui-app-widget, `canvasmode.inherited.test.ts`. A node a page kind or a widget
// contributes is not written in the document, so no remove applies to it on the canvas; a node the
// author wrote keeps it.

function n(path: string, layer: string, kind: string, extra: Partial<OutlineNode> = {}): OutlineNode {
  const name = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!.split(':')[1];
  return { path, layer, name, kind, children: [], ...extra };
}

const filters = n('page:loans/section:filters', 'section', 'filter_bar', { inherited: true });
const body = n('page:overview/section:latest/node:title', 'node', 'text', { inherited: true });
const list = n('page:loans/section:list', 'section', 'collection');

test('the actions for an inherited node exclude remove, in both canvas modes', () => {
  for (const node of [filters, body]) {
    for (const mode of ['structure', 'preview'] as const) {
      assert.ok(!nodeActions(node, mode).includes('remove'), `${node.path} in ${mode}`);
    }
  }
});

test('a node the author wrote keeps remove in structure', () => {
  assert.deepEqual(nodeActions(list, 'structure'), ['remove']);
  assert.deepEqual(nodeActions({ ...list, inherited: false }, 'structure'), ['remove']);
});

test('preview applies no action; the root and the menu are not removable', () => {
  assert.deepEqual(nodeActions(list, 'preview'), []);
  assert.deepEqual(nodeActions(n('/', 'root', 'document'), 'structure'), []);
  assert.deepEqual(nodeActions(n('nav', 'nav', 'navigation'), 'structure'), []);
});
