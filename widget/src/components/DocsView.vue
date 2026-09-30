<script setup lang="ts">
import { computed } from 'vue';
import { renderMarkdown } from '../lib/markdown.ts';
import { docsText } from '../remote.ts';

// renderMarkdown escapes every piece of input, so its output is safe to insert as HTML.
const html = computed(() => (docsText.text === null ? '' : renderMarkdown(docsText.text)));
</script>

<template>
  <div class="text-view">
    <div class="text-view-bar muted small">
      <span v-if="docsText.loading">loading…</span>
      <span class="spacer"></span>
      <button class="link" @click="docsText.reload()">reload</button>
    </div>
    <p v-if="docsText.error" class="notice">{{ docsText.error }}</p>
    <article v-if="docsText.text !== null" class="md" v-html="html"></article>
  </div>
</template>
