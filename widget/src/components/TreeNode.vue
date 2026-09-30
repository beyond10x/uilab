<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { isRemoved, marks, select, state } from '../store.ts';

const props = defineProps<{ node: OutlineNode; depth: number }>();

const PREFIX: Record<string, string> = {
  root: 'doc',
  shell: 'shl',
  region: 'reg',
  nav: 'nav',
  nav_section: 'grp',
  page: 'pg',
  section: 'sec',
  overlay: 'ovl',
  widget: 'wdg',
  item: 'itm',
};

const open = computed(() => state.expanded.has(props.node.path));
const hasChildren = computed(() => props.node.children.length > 0);

function toggle(): void {
  if (open.value) state.expanded.delete(props.node.path);
  else state.expanded.add(props.node.path);
}
</script>

<template>
  <li>
    <div
      class="tree-row"
      :class="[marks(node.path), { removed: isRemoved(node.path) }]"
      :style="{ paddingLeft: `${depth * 14 + 4}px` }"
      :title="node.path"
      @click="select(node.path)"
    >
      <span class="twisty" @click.stop="hasChildren && toggle()">{{ hasChildren ? (open ? '▾' : '▸') : '' }}</span>
      <span class="layer" :class="`layer-${node.layer}`">{{ PREFIX[node.layer] ?? node.layer }}</span>
      <span class="tree-name">{{ node.name || (node.layer === 'root' ? node.title || 'document' : node.layer) }}</span>
      <span class="muted small">{{ node.kind }}</span>
    </div>
    <ul v-if="open && hasChildren" class="tree">
      <TreeNode v-for="c in node.children" :key="c.path" :node="c" :depth="depth + 1" />
    </ul>
  </li>
</template>
