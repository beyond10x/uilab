<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { localId, operatorView, setName, state } from '../store.ts';

const operators = computed(() =>
  state.presence.operators.map((o) => operatorView(o.id)!).sort((a, b) => Number(b.local) - Number(a.local)),
);
/** The local operator shown even before presence names it. */
const selfMissing = computed(() => !localId.value);

const draft = ref(state.name);
watch(
  () => state.name,
  (n) => (draft.value = n),
);

function commit(): void {
  if (draft.value.trim()) setName(draft.value);
  else draft.value = state.name;
}
</script>

<template>
  <div class="presence">
    <label class="you-name">
      <span class="muted small">you</span>
      <input v-model="draft" type="text" maxlength="40" aria-label="your name" @change="commit" @keydown.enter="($event.target as HTMLInputElement).blur()" />
    </label>
    <div class="presence-strip">
      <span
        v-for="o in operators"
        :key="o.id"
        class="op-chip"
        :class="[`op-${o.kind}`, { local: o.local }]"
        :style="{ '--op-colour': o.colour }"
        :title="`${o.name} · ${o.kind} · ${o.id}`"
      >
        <span v-if="o.kind === 'agent'" class="glyph">🤖</span>
        <span v-else class="op-dot"></span>
        {{ o.name }}
        <span class="op-kind">{{ o.local ? 'you' : o.kind }}</span>
      </span>
      <span v-if="selfMissing && state.conn === 'open'" class="op-chip local pending" :title="'presence does not list this browser yet'">
        <span class="op-dot"></span>{{ state.name }} <span class="op-kind">you</span>
      </span>
    </div>
  </div>
</template>
