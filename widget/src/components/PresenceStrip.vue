<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { presenceChips, renamed } from '../lib/sidebar.ts';
import { operatorView, setName, state } from '../store.ts';

/** Operators presence lists, the local one first, and the local one in every connection state. */
const chips = computed(() =>
  presenceChips(
    state.presence.operators.map((o) => operatorView(o.id)!),
    state.name,
    state.conn,
  ),
);

const editing = ref(false);
const draft = ref('');
const input = ref<HTMLInputElement[] | null>(null);

async function edit(): Promise<void> {
  draft.value = state.name;
  editing.value = true;
  await nextTick();
  input.value?.[0]?.select();
}

function commit(): void {
  if (!editing.value) return;
  editing.value = false;
  const n = renamed(draft.value, state.name);
  if (n) setName(n);
}

function cancel(): void {
  editing.value = false;
}
</script>

<template>
  <div class="presence-strip">
    <span
      v-for="o in chips"
      :key="o.id || 'self'"
      class="op-chip"
      :class="[`op-${o.kind}`, { local: o.local, pending: o.pending, editing: o.local && editing }]"
      :style="{ '--op-colour': o.colour }"
      :title="o.title"
    >
      <span v-if="o.kind === 'agent'" class="glyph">🤖</span>
      <span v-else class="op-dot"></span>
      <template v-if="o.local && editing">
        <input
          ref="input"
          v-model="draft"
          class="chip-input"
          type="text"
          maxlength="40"
          aria-label="your name"
          @blur="commit"
          @keydown.enter.prevent="commit"
          @keydown.esc.stop.prevent="cancel"
        />
      </template>
      <button v-else-if="o.local" type="button" class="chip-name" aria-label="rename yourself" @click="edit">{{ o.name }}</button>
      <span v-else class="chip-label">{{ o.name }}</span>
      <span class="op-kind">{{ o.local ? 'you' : o.kind }}</span>
    </span>
  </div>
</template>

<style scoped>
.presence-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
}

.op-chip {
  max-width: 100%;
  min-width: 0;
}

.op-chip .glyph,
.op-chip .op-dot,
.op-chip .op-kind {
  flex: none;
}

.chip-name,
.chip-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chip-name {
  border: none;
  background: none;
  padding: 0;
  font: inherit;
  color: inherit;
  cursor: text;
}

.chip-name:hover {
  text-decoration: underline dotted;
}

.chip-input[type='text'] {
  width: 9ch;
  padding: 0 4px;
  font-size: 12px;
  font-weight: 600;
  border: none;
  outline: 1px solid var(--op-colour);
  border-radius: 3px;
}
</style>
