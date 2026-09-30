<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { renamed } from '../lib/sidebar.ts';
import { localId, operatorView, setName, state, type OperatorView } from '../store.ts';

/** Operators presence lists, the local one first; before presence names this browser, a pending chip. */
const chips = computed<(OperatorView & { pending?: boolean })[]>(() => {
  const listed = state.presence.operators.map((o) => operatorView(o.id)!).sort((a, b) => Number(b.local) - Number(a.local));
  if (localId.value || state.conn !== 'open') return listed;
  const self = { id: '', name: state.name, kind: 'human' as const, colour: 'var(--muted)', local: true, pending: true };
  return [self, ...listed];
});

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

function title(o: OperatorView & { pending?: boolean }): string {
  if (o.local) return o.pending ? 'you (presence does not list this browser yet) · click to rename' : 'you · click to rename';
  return `${o.name} · ${o.kind} · ${o.id}`;
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
      :title="title(o)"
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
      <template v-else>{{ o.name }}</template>
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
