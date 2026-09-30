<script setup lang="ts">
import { computed, ref } from 'vue';
import { draftsLabel, draftViews, faults, findingsBadge, phaseLabel } from '../lib/sidebar.ts';
import { micDown, micUp, say, sayGoal, select, selectedBy, selectedNode, shownOutline, state, undo } from '../store.ts';
import ActivityFeed from './ActivityFeed.vue';
import GoalPanel from './GoalPanel.vue';
import PresenceStrip from './PresenceStrip.vue';
import ProposalCard from './ProposalCard.vue';
import TreeNode from './TreeNode.vue';

const text = ref('');
const findingsOpen = ref(false);
const doc = computed(() => state.doc);
const badge = computed(() => findingsBadge(doc.value?.findings ?? []));
const listed = computed(() => faults(doc.value?.findings ?? []));
const drafts = computed(() => draftViews(doc.value?.findings ?? []));
const draftsOpen = ref(false);

const connLabel = computed(() => {
  if (state.conn === 'open') return 'connected';
  if (state.conn === 'connecting') return 'connecting…';
  return `disconnected, retrying in ${Math.ceil(state.retryInMs / 1000)} s`;
});

const statusLabel = computed(() => phaseLabel(state.phase, state.thinkingTarget, state.goal?.state === 'planning'));

function send(): void {
  if (state.goalMode) sayGoal(text.value);
  else say(text.value);
  text.value = '';
}

function down(ev: PointerEvent): void {
  (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
  void micDown();
}
</script>

<template>
  <div class="side">
    <div class="side-head" data-section="header">
      <div class="head-row">
        <PresenceStrip />
        <span class="conn" :class="`conn-${state.conn}`" :title="connLabel" role="status" :aria-label="connLabel">
          <span class="dot"></span>
        </span>
        <button
          v-if="doc"
          type="button"
          class="badge"
          :class="{ 'has-error': badge.errors, 'has-warning': !badge.errors && badge.warnings }"
          :title="`${badge.title} (click to ${findingsOpen ? 'hide' : 'list'})`"
          :aria-label="badge.title"
          :aria-expanded="findingsOpen"
          @click="findingsOpen = !findingsOpen"
        >
          <template v-if="badge.errors || badge.warnings">
            <span v-if="badge.errors" class="count-error">✕ {{ badge.errors }}</span>
            <span v-if="badge.warnings" class="count-warning">⚠ {{ badge.warnings }}</span>
          </template>
          <span v-else>✓</span>
        </button>
        <button class="help-button" title="help (?)" aria-label="help" @click="state.helpOpen = true">?</button>
      </div>
      <div v-if="state.conn !== 'open'" class="conn-line small" :class="`conn-${state.conn}`" aria-hidden="true">{{ connLabel }}</div>
      <ul v-if="doc && findingsOpen" class="findings">
        <li v-for="(f, i) in listed" :key="i" :class="f.severity">
          <code>{{ f.check }}</code> {{ f.message }}
          <code class="path" @click="select(f.path)">{{ f.path }}</code>
        </li>
        <li v-if="!listed.length" class="muted">none</li>
      </ul>
      <div v-if="doc && drafts.length" class="drafts">
        <button
          type="button"
          class="drafts-toggle"
          :title="`views the document reads that have no model binding yet (click to ${draftsOpen ? 'hide' : 'list'})`"
          :aria-expanded="draftsOpen"
          @click="draftsOpen = !draftsOpen"
        >
          <span class="caret" aria-hidden="true">{{ draftsOpen ? '▾' : '▸' }}</span>
          Data to model
          <span class="muted">· {{ draftsLabel(drafts.length) }}</span>
        </button>
        <ul v-if="draftsOpen" class="draft-list">
          <li v-for="d in drafts" :key="d.view">
            <button type="button" class="link draft-view" :title="`select ${d.paths[0]}`" @click="select(d.paths[0])">
              <code>{{ d.view }}</code>
            </button>
            <span class="muted small">read by</span>
            <span class="draft-paths">
              <button v-for="p in d.paths" :key="p" type="button" class="link draft-path" :title="`select ${p}`" @click="select(p)">
                <code>{{ p }}</code>
              </button>
            </span>
          </li>
        </ul>
      </div>
    </div>

    <div class="card-area" data-section="card">
      <div v-if="state.notice" class="notice">
        <code v-if="state.notice.check">{{ state.notice.check }}</code>
        <strong v-else>{{ state.notice.kind }}</strong>
        {{ state.notice.message }}
      </div>
      <ProposalCard v-if="state.proposal" :proposal="state.proposal" />
      <GoalPanel v-if="state.goal" :goal="state.goal" />
      <div class="undo-row">
        <button :disabled="!doc?.undoable || !!state.proposal" @click="undo">Undo <kbd>Ctrl+Z</kbd></button>
      </div>
    </div>

    <div class="voice" data-section="input">
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
      <div v-if="statusLabel" class="status" :class="`phase-${state.phase}`">{{ statusLabel }}</div>
      <form class="say" @submit.prevent="send">
        <input
          v-model="text"
          type="text"
          :placeholder="state.goalMode ? 'type a goal, Enter plans it in steps' : 'or type an instruction, Enter sends'"
        />
        <div class="say-mode" role="group" aria-label="what Enter sends">
          <button type="button" :aria-pressed="!state.goalMode" :class="{ active: !state.goalMode }" @click="state.goalMode = false">instruction</button>
          <button type="button" :aria-pressed="state.goalMode" :class="{ active: state.goalMode }" @click="state.goalMode = true">goal</button>
        </div>
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

    <div class="tree-area" data-section="tree">
      <div v-if="doc" class="doc-title" :title="doc.file">{{ doc.title || shownOutline?.title || 'untitled' }}</div>
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
    </div>

    <div data-section="activity">
      <ActivityFeed />
    </div>
  </div>
</template>

<style scoped>
.side-head {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 6px;
}

.head-row {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.head-row > :first-child {
  flex: 1;
}

.conn-line {
  color: var(--muted);
}

.conn {
  flex: none;
}

.badge {
  flex: none;
  display: inline-flex;
  gap: 6px;
  padding: 1px 8px;
  font-size: 12px;
  border-radius: 10px;
  color: var(--muted);
}

.badge.has-error {
  border-color: var(--remove);
}

.badge.has-warning {
  border-color: var(--replace);
}

.help-button {
  flex: none;
}

.side-head .findings {
  margin: 0;
  max-height: 30vh;
  overflow: auto;
}

.drafts-toggle {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  padding: 1px 8px;
  font-size: 12px;
  border-style: dashed;
}

.drafts-toggle .caret {
  width: 10px;
  color: var(--muted);
}

.draft-list {
  list-style: none;
  margin: 6px 0 0;
  padding: 0;
  font-size: 12px;
  max-height: 30vh;
  overflow: auto;
}

.draft-list li {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px;
  padding: 3px 0 3px 8px;
  border-left: 3px dashed var(--line);
  margin-bottom: 4px;
}

.draft-paths {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 4px;
  min-width: 0;
}

.draft-list button.link {
  padding: 0;
  font-size: 12px;
  word-break: break-all;
  text-align: left;
}

.draft-view code {
  color: var(--fg);
  font-weight: 600;
}

.draft-path code {
  color: var(--accent);
}

.card-area,
.tree-area {
  display: grid;
  gap: 8px;
}

.doc-title {
  margin-top: 0;
  font-size: 14px;
  cursor: help;
}

.undo-row button {
  font-size: 12px;
  padding: 2px 8px;
}
</style>
