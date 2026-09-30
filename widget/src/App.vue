<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { accept, agentActions, goalBanner, micDown, micUp, operatorView, reject, selectNearest, state, undo, type ViewMode } from './store.ts';

/** The goal's own banner, when no agent banner of its operator already says `goal i/n`. */
const goalOwner = computed(() => (goalBanner.value && state.goal ? operatorView(state.goal.by) : null));
const goalOnly = computed(() => !!goalOwner.value && !agentActions.value.some((a) => a.op.id === goalOwner.value!.id));
import { API } from './remote.ts';
import CanvasView from './components/CanvasView.vue';
import DocsView from './components/DocsView.vue';
import HelpModal from './components/HelpModal.vue';
import SidebarPanel from './components/SidebarPanel.vue';
import YamlView from './components/YamlView.vue';

/** Whether keyboard focus is where typed keys belong to the element, not to the app. */
function typing(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el || !el.tagName) return false;
  return el.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName);
}

const VIEWS: { mode: ViewMode; label: string; key: string }[] = [
  { mode: 'ui', label: 'UI', key: '1' },
  { mode: 'yaml', label: 'YAML', key: '2' },
  { mode: 'docs', label: 'Docs', key: '3' },
];

function exportYaml(): void {
  window.location.assign(API.yamlDownload);
}

function onKeyDown(ev: KeyboardEvent): void {
  if (state.helpOpen) {
    // Help is modal: Esc closes it and every other key waits.
    if (ev.key === 'Escape' || ev.key === '?') {
      ev.preventDefault();
      state.helpOpen = false;
    }
    return;
  }
  if (typing(ev.target)) return;
  const plain = !ev.ctrlKey && !ev.metaKey && !ev.altKey;
  const view = plain ? VIEWS.find((v) => v.key === ev.key) : undefined;
  if (view) {
    ev.preventDefault();
    state.view = view.mode;
  } else if (plain && ev.key === '?') {
    ev.preventDefault();
    state.helpOpen = true;
  } else if (ev.code === 'Space') {
    ev.preventDefault();
    if (!ev.repeat) void micDown();
  } else if (ev.key === 'Enter' && state.proposal) {
    ev.preventDefault();
    accept();
  } else if (ev.key === 'Escape') {
    if (state.proposal) {
      ev.preventDefault();
      reject();
    } else if (state.openOverlay) {
      state.openOverlay = null;
    }
  } else if ((ev.ctrlKey || ev.metaKey) && !ev.shiftKey && ev.key.toLowerCase() === 'z') {
    ev.preventDefault();
    undo();
  }
}

function onKeyUp(ev: KeyboardEvent): void {
  if (ev.code === 'Space') {
    if (!typing(ev.target)) ev.preventDefault();
    micUp();
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  window.addEventListener('blur', micUp);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('keyup', onKeyUp);
  window.removeEventListener('blur', micUp);
});
</script>

<template>
  <div class="app">
    <div class="canvas-pane">
      <div class="canvas-head">
        <nav class="view-tabs" aria-label="view">
          <button
            v-for="v in VIEWS"
            :key="v.mode"
            class="view-tab"
            :class="{ active: state.view === v.mode }"
            :aria-pressed="state.view === v.mode"
            :title="`${v.label} (${v.key})`"
            @click="state.view = v.mode"
          >
            {{ v.label }} <kbd>{{ v.key }}</kbd>
          </button>
          <span class="spacer"></span>
          <button class="export" title="download the document as YAML" @click="exportYaml">Export YAML</button>
        </nav>
        <div v-if="agentActions.length || goalOnly" class="op-banners">
          <div
            v-for="a in agentActions"
            :key="a.op.id"
            class="op-banner"
            :style="{ '--op-colour': a.op.colour }"
            @click="selectNearest(a.target)"
          >
            🤖 <strong>{{ a.op.name }}</strong> is operating on <code>{{ a.target }}</code>
            <span v-if="goalBanner && state.goal?.by === a.op.id" class="op-goal">· {{ goalBanner }}</span>
          </div>
          <div v-if="goalOnly && goalOwner" class="op-banner" :style="{ '--op-colour': goalOwner.colour }">
            🎯 <strong>{{ goalOwner.local ? 'your' : `${goalOwner.name}’s` }}</strong> {{ goalBanner }}: {{ state.goal?.text }}
          </div>
        </div>
      </div>
      <CanvasView v-if="state.view === 'ui'" />
      <YamlView v-else-if="state.view === 'yaml'" />
      <DocsView v-else />
    </div>
    <aside class="sidebar"><SidebarPanel /></aside>
    <HelpModal v-if="state.helpOpen" />
  </div>
</template>
