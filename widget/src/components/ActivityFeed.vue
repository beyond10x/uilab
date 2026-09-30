<script setup lang="ts">
import type { FeedEntry } from '../lib/collab.ts';
import { operatorView, selectNearest, state } from '../store.ts';

function time(ms: number): string {
  return new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function who(e: FeedEntry) {
  return operatorView(e.by);
}

function open(e: FeedEntry): void {
  if (e.path) selectNearest(e.path);
}
</script>

<template>
  <details class="feed-box" open>
    <summary>activity <span class="muted small">({{ state.feed.length }})</span></summary>
    <ol class="feed">
      <li
        v-for="e in state.feed"
        :key="e.seq"
        class="feed-row"
        :class="[`feed-${e.kind}`, { clickable: !!e.path }]"
        :title="e.text"
        @click="open(e)"
      >
        <span class="feed-time muted">{{ time(e.at) }}</span>
        <span v-if="who(e)" class="feed-who" :style="{ color: who(e)!.colour }">
          {{ who(e)!.kind === 'agent' ? '🤖 ' : '' }}{{ who(e)!.local ? 'you' : who(e)!.name }}
        </span>
        <span v-else class="feed-who muted">you</span>
        <span class="feed-kind">{{ e.kind }}</span>
        <code v-if="e.what" class="feed-what">{{ e.what }}</code>
        <code v-if="e.path" class="path">{{ e.path }}</code>
        <span v-if="e.kind === 'transcript' || e.kind === 'moved' || (!e.path && e.text)" class="feed-text muted">{{ e.text }}</span>
      </li>
      <li v-if="!state.feed.length" class="muted small">nothing yet</li>
    </ol>
  </details>
</template>
