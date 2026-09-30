<script setup lang="ts">
import { computed, ref } from 'vue';
import { micDown, micUp, say, select, selectedBy, selectedNode, shownOutline, state, undo } from '../store.ts';
import ActivityFeed from './ActivityFeed.vue';
import PresenceStrip from './PresenceStrip.vue';
import ProposalCard from './ProposalCard.vue';
import TreeNode from './TreeNode.vue';

const text = ref('');
const doc = computed(() => state.doc);
const errors = computed(() => doc.value?.findings.filter((f) => f.severity === 'error') ?? []);
const warnings = computed(() => doc.value?.findings.filter((f) => f.severity === 'warning') ?? []);

const connLabel = computed(() => {
  if (state.conn === 'open') return 'connected';
  if (state.conn === 'connecting') return 'connecting…';
  return `disconnected, retrying in ${Math.ceil(state.retryInMs / 1000)} s`;
});

const statusLabel = computed(() => {
  switch (state.phase) {
    case 'arming':
      return 'opening microphone…';
    case 'listening':
      return 'listening';
    case 'transcribing':
      return 'transcribing…';
    case 'thinking':
      return state.thinkingTarget ? `thinking about ${state.thinkingTarget}…` : 'thinking…';
    default:
      return 'idle';
  }
});

function send(): void {
  say(text.value);
  text.value = '';
}

function down(ev: PointerEvent): void {
  (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
  void micDown();
}
</script>

<template>
  <div class="side">
    <div class="side-head">
      <PresenceStrip />
      <div class="conn" :class="`conn-${state.conn}`"><span class="dot"></span>{{ connLabel }}</div>
      <template v-if="doc">
        <div class="doc-title">{{ doc.title || shownOutline?.title || 'untitled' }}</div>
        <div class="muted small file">{{ doc.file }}</div>
      </template>
    </div>


    <details v-if="doc" class="findings-box">
      <summary>
        <span :class="{ 'count-error': errors.length }">{{ errors.length }} error{{ errors.length === 1 ? '' : 's' }}</span>,
        <span :class="{ 'count-warning': warnings.length }">{{ warnings.length }} warning{{ warnings.length === 1 ? '' : 's' }}</span>
      </summary>
      <ul class="findings">
        <li v-for="(f, i) in doc.findings" :key="i" :class="f.severity">
          <code>{{ f.check }}</code> {{ f.message }}
          <code class="path" @click="select(f.path)">{{ f.path }}</code>
        </li>
        <li v-if="!doc.findings.length" class="muted">none</li>
      </ul>
    </details>

    <div class="tree-box">
      <ul v-if="shownOutline" class="tree">
        <TreeNode :node="shownOutline" :depth="0" />
      </ul>
    </div>

    <div class="selected-path">
      <span class="muted small">selected</span>
      <code>{{ doc?.selected ?? '—' }}</code>
      <span v-if="selectedNode" class="muted small">{{ selectedNode.kind }}</span>
      <span v-if="selectedBy" class="selected-by small" :style="{ color: selectedBy.colour }">selected by {{ selectedBy.kind === 'agent' ? '🤖 ' : '' }}{{ selectedBy.local ? 'you' : selectedBy.name }}</span>
    </div>

    <div class="voice">
      <button
        class="mic"
        :class="{ live: state.phase === 'listening', arming: state.phase === 'arming' }"
        :disabled="state.conn !== 'open'"
        @pointerdown.prevent="down"
        @pointerup="micUp"
        @pointercancel="micUp"
        @lostpointercapture="micUp"
      >
        {{ state.phase === 'listening' ? '● listening' : '🎙 hold to talk' }}
        <span class="hint">or hold <kbd>Space</kbd></span>
      </button>
      <div class="meter"><div class="meter-fill" :style="{ width: `${Math.round(state.level * 100)}%` }"></div></div>
      <div class="status" :class="`phase-${state.phase}`">{{ statusLabel }}</div>
      <form class="say" @submit.prevent="send">
        <input v-model="text" type="text" placeholder="or type an instruction, Enter sends" />
      </form>
      <div v-if="state.transcript" class="transcript">
        <span class="muted small">heard ({{ (state.transcript.audio_ms / 1000).toFixed(1) }} s audio, {{ state.transcript.took_ms }} ms)</span>
        <div>{{ state.transcript.text || '(nothing)' }}</div>
      </div>
      <div v-else-if="state.typed" class="transcript">
        <span class="muted small">typed</span>
        <div>{{ state.typed }}</div>
      </div>
    </div>

    <div v-if="state.notice" class="notice">
      <code v-if="state.notice.check">{{ state.notice.check }}</code>
      <strong v-else>{{ state.notice.kind }}</strong>
      {{ state.notice.message }}
    </div>

    <ProposalCard v-if="state.proposal" :proposal="state.proposal" />

    <div class="undo-row">
      <button :disabled="!doc?.undoable || !!state.proposal" @click="undo">Undo <kbd>Ctrl+Z</kbd></button>
    </div>

    <ActivityFeed />
  </div>
</template>
