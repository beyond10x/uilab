<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { displayValue } from '../lib/components.ts';
import { propsOf } from '../lib/outline.ts';
import { marks, select, tint } from '../store.ts';

const props = defineProps<{ node: OutlineNode }>();

const p = computed(() => propsOf(props.node));
/** The words a primitive shows: its `text`, else its `label`, else its name. */
const text = computed(() => displayValue(p.value.text ?? p.value.label ?? props.node.name));
const label = computed(() => displayValue(p.value.label ?? p.value.text ?? props.node.name));
const heading = computed(() => p.value.style === 'heading');
const alt = computed(() => displayValue(p.value.alt ?? p.value.text ?? 'image'));
</script>

<template>
  <div
    class="prim node"
    :class="marks(node.path)"
    :style="tint(node.path)"
    :data-path="node.path"
    :title="`${node.name} · ${node.kind}`"
    @click.stop="select(node.path)"
  >
    <template v-if="node.kind === 'text'">
      <h4 v-if="heading" class="prim-heading">{{ text }}</h4>
      <span v-else>{{ text }}</span>
    </template>
    <span v-else-if="node.kind === 'badge'" class="tag">{{ text }}</span>
    <div v-else-if="node.kind === 'image'" class="prim-image">{{ alt }}</div>
    <a v-else-if="node.kind === 'link'" href="#" tabindex="-1" @click.prevent>{{ text }}</a>
    <button v-else-if="node.kind === 'button'" type="button" tabindex="-1">{{ label }}</button>
    <div v-else class="placeholder prim-placeholder">{{ node.kind }} · {{ node.name }}</div>
  </div>
</template>
