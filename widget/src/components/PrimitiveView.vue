<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { displayValue, primitiveShape } from '../lib/components.ts';
import { nodeClasses, propsOf } from '../lib/outline.ts';
import { marks, select, tint } from '../store.ts';

const props = defineProps<{ node: OutlineNode }>();

const p = computed(() => propsOf(props.node));
const shape = computed(() => primitiveShape(props.node));
/** The words a primitive shows: its `text`, else its `label`, else its name. */
const text = computed(() => displayValue(p.value.text ?? p.value.label ?? props.node.name));
const label = computed(() => displayValue(p.value.label ?? p.value.text ?? props.node.name));
const alt = computed(() => displayValue(p.value.alt ?? p.value.text ?? 'image'));
</script>

<template>
  <div
    class="prim node"
    :class="[marks(node.path), nodeClasses(node), { 'prim-block': shape === 'rule' }]"
    :style="tint(node.path)"
    :data-path="node.path"
    :title="`${node.name} · ${node.kind}`"
    @click.stop="select(node.path)"
  >
    <h4 v-if="shape === 'heading'" class="prim-heading">{{ text }}</h4>
    <span v-else-if="shape === 'text'">{{ text }}</span>
    <span v-else-if="shape === 'badge'" class="tag">{{ text }}</span>
    <div v-else-if="shape === 'image'" class="prim-image">{{ alt }}</div>
    <a v-else-if="shape === 'link'" href="#" tabindex="-1" @click.prevent>{{ text }}</a>
    <button v-else-if="shape === 'button'" type="button" tabindex="-1">{{ label }}</button>
    <hr v-else-if="shape === 'rule'" class="prim-rule" />
    <div v-else class="placeholder prim-placeholder">{{ node.kind }} · {{ node.name }}</div>
  </div>
</template>
