<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { yamlBlock } from '../lib/yamlblock.ts';
import { yamlText } from '../remote.ts';
import { state } from '../store.ts';

const lines = computed(() => (yamlText.text ?? '').replace(/\r\n?/g, '\n').replace(/\n$/, '').split('\n'));
const selected = computed(() => state.doc?.selected ?? '/');
const block = computed(() => (yamlText.text ? yamlBlock(yamlText.text, selected.value) : null));
const pre = ref<HTMLElement | null>(null);

function marked(i: number): boolean {
  const b = block.value;
  return !!b && i >= b.start && i < b.end;
}

watch(
  block,
  async (b) => {
    if (!b) return;
    await nextTick();
    pre.value?.querySelector<HTMLElement>(`[data-line="${b.start}"]`)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
  },
  { immediate: true },
);
</script>

<template>
  <div class="text-view">
    <div class="text-view-bar muted small">
      <span v-if="yamlText.loading">loading…</span>
      <span v-else-if="block">lines {{ block.start + 1 }}–{{ block.end }}: <code>{{ selected }}</code></span>
      <span v-else-if="yamlText.text && selected !== '/'">no block found for <code>{{ selected }}</code></span>
      <span class="spacer"></span>
      <button class="link" @click="yamlText.reload()">reload</button>
    </div>
    <p v-if="yamlText.error" class="notice">{{ yamlText.error }}</p>
    <pre v-if="yamlText.text !== null" ref="pre" class="yaml"><div
      v-for="(l, i) in lines"
      :key="i"
      class="yaml-line"
      :class="{ marked: marked(i) }"
      :data-line="i"
    ><span class="ln">{{ i + 1 }}</span><span class="yaml-text">{{ l || ' ' }}</span></div></pre>
  </div>
</template>
