<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { MODE_HELP } from '../lib/canvasmode.ts';
import { renderMarkdown } from '../lib/markdown.ts';
import { helpText } from '../remote.ts';
import { state } from '../store.ts';

/** The widget's own keys; App.vue handles them. */
const SHORTCUTS: [string[], string][] = [
  [['Space'], 'hold to speak an instruction'],
  [['Enter'], 'accept the proposal'],
  [['Esc'], 'reject the proposal, or close an overlay or this help'],
  [['Ctrl', 'Z'], 'undo the last accepted change'],
  [['?'], 'open this help'],
  [['1'], 'show the UI'],
  [['2'], 'show the document as YAML'],
  [['3'], 'show the generated docs'],
  [['4'], 'show the components (widgets) with previews'],
  MODE_HELP,
];

// renderMarkdown escapes every piece of input, so its output is safe to insert as HTML.
const html = computed(() => (helpText.text === null ? '' : renderMarkdown(helpText.text)));

onMounted(() => helpText.reload());
</script>

<template>
  <div class="help-backdrop" @click="state.helpOpen = false">
    <div class="help" role="dialog" aria-modal="true" aria-label="help" @click.stop>
      <div class="modal-bar">
        <strong>Help</strong>
        <button class="link" @click="state.helpOpen = false">close <kbd>Esc</kbd></button>
      </div>
      <h3>Keys</h3>
      <p class="muted small">Keys do nothing while a text field has focus; in the name field Enter keeps and Esc cancels the new name.</p>
      <table class="shortcuts">
        <tr v-for="[keys, what] in SHORTCUTS" :key="what">
          <td>
            <template v-for="(k, i) in keys" :key="k"><template v-if="i">+</template><kbd>{{ k }}</kbd></template>
          </td>
          <td>{{ what }}</td>
        </tr>
      </table>
      <p v-if="helpText.loading && helpText.text === null" class="muted">loading…</p>
      <p v-if="helpText.error" class="notice">{{ helpText.error }}</p>
      <article v-if="helpText.text !== null" class="md" v-html="html"></article>
    </div>
  </div>
</template>
