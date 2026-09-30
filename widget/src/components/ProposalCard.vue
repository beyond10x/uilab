<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireProposalShown as ProposalShown } from '../generated/types.ts';
import { diffLines, hunks } from '../lib/diff.ts';
import { accept, operatorView, reject, select, state } from '../store.ts';

const props = defineProps<{ proposal: ProposalShown }>();

const diff = computed(() => hunks(diffLines(props.proposal.before, props.proposal.after)));
const opClass = computed(() => `op-${props.proposal.op.toLowerCase()}`);
const by = computed(() => operatorView(props.proposal.by));
</script>

<template>
  <section class="proposal" :class="opClass">
    <div class="proposal-head">
      <span class="op-badge">{{ proposal.op }}</span>
      <code class="path" @click="select(proposal.target)">{{ proposal.target }}</code>
      <template v-if="proposal.changed !== proposal.target">
        →
        <code class="path">{{ proposal.changed }}</code>
      </template>
      <span v-if="by" class="by small" :style="{ color: by.colour }">
        by {{ by.kind === 'agent' ? '🤖 ' : '' }}{{ by.local ? 'you' : by.name }}
      </span>
    </div>
    <p class="utterance">“{{ proposal.utterance }}”</p>
    <div class="diff">
      <p v-if="!diff.length" class="muted small">no textual change</p>
      <template v-for="h in diff" :key="h.header">
        <div class="diff-hunk">{{ h.header }}</div>
        <div v-for="(l, i) in h.lines" :key="`${h.header}-${i}`" class="diff-line" :class="{ add: l.op === '+', del: l.op === '-' }">
          <span class="diff-op">{{ l.op }}</span>{{ l.text }}
        </div>
      </template>
    </div>
    <ul v-if="proposal.findings.length" class="findings">
      <li v-for="(f, i) in proposal.findings" :key="i" :class="f.severity">
        <strong>{{ f.severity }}</strong> <code>{{ f.check }}</code> {{ f.message }}
        <code class="path" @click="select(f.path)">{{ f.path }}</code>
      </li>
    </ul>
    <div class="decide">
      <button class="primary" :disabled="state.deciding" @click="accept">Accept <kbd>Enter</kbd></button>
      <button :disabled="state.deciding" @click="reject">Reject <kbd>Esc</kbd></button>
      <span v-if="state.deciding" class="muted small">sent…</span>
    </div>
  </section>
</template>
