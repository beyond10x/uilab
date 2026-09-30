<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { yamlBlock } from '../lib/yamlblock.ts';
import { yamlTokens } from '../lib/yamltokens.ts';
import { yamlText } from '../remote.ts';
import { state } from '../store.ts';

const lines = computed(() => yamlTokens(yamlText.text ?? ''));
const selected = computed(() => state.doc?.selected ?? '/');
const block = computed(() => (yamlText.text ? yamlBlock(yamlText.text, selected.value) : null));
const pre = ref<HTMLElement | null>(null);

function marked(i: number): boolean {
  const b = block.value;
  return !!b && i >= b.start && i < b.end;
}

// The selected block is brought into view when the tab opens and when the selection moves; a
// reload that leaves it where it was does not scroll. The first placement jumps, later ones glide.
let placed = false;
watch(
  () => (block.value ? `${selected.value}@${block.value.start}` : null),
  async (where) => {
    if (!where) return;
    const start = block.value!.start;
    await nextTick();
    const line = pre.value?.querySelector<HTMLElement>(`[data-line="${start}"]`);
    if (!line) return;
    line.scrollIntoView({ block: 'center', behavior: placed ? 'smooth' : 'auto' });
    placed = true;
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
    ><span class="ln">{{ i + 1 }}</span><span class="yaml-text"><template v-if="l.length"><span
      v-for="(t, j) in l"
      :key="j"
      :class="`tok-${t.kind}`"
    >{{ t.text }}</span></template><template v-else> </template></span></div></pre>
  </div>
</template>

<style scoped>
.tok-key {
  color: #953800;
}

.tok-string {
  color: #116329;
}

.tok-number,
.tok-literal {
  color: #0550ae;
}

.tok-comment {
  color: #6e7781;
  font-style: italic;
}

.tok-punct {
  color: #8c959f;
}

.yaml-line.marked {
  background: #dce6fd;
}
</style>
