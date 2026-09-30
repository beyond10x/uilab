<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { accept, agentActions, goalBanner, micDown, micUp, operatorView, reject, selectNearest, state, undo, type ViewMode } from './store.ts';

/** The goal's own banner, when no agent banner of its operator already says `goal i/n`. */
const goalOwner = computed(() => (goalBanner.value && state.goal ? operatorView(state.goal.by) : null));
const goalOnly = computed(() => !!goalOwner.value && !agentActions.value.some((a) => a.op.id === goalOwner.value!.id));
import { canvasMode, togglesMode } from './lib/canvasmode.ts';
import { enterAccepts, type KeyTarget } from './lib/keys.ts';
import { API } from './remote.ts';
import CanvasView from './components/CanvasView.vue';
import ComponentsView from './components/ComponentsView.vue';
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
  { mode: 'components', label: 'Components', key: '4' },
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
  } else if (togglesMode(ev)) {
    ev.preventDefault();
    if (!ev.repeat) canvasMode.toggle();
  } else if (ev.code === 'Space') {
    ev.preventDefault();
    if (!ev.repeat) void micDown();
  } else if (ev.key === 'Enter' && state.proposal && enterAccepts(ev.target as KeyTarget | null, focusedByPointer)) {
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

/**
 * Whether the element that has focus got it from a mouse click: a focus within a moment of a
 * pointer press. `:focus-visible` cannot tell, since Chrome reports it true at keydown.
 */
let pointerAt = -Infinity;
let focusedByPointer = false;

function onPointerDown(): void {
  pointerAt = performance.now();
}

function onFocusIn(): void {
  focusedByPointer = performance.now() - pointerAt < 500;
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
  window.addEventListener('pointerdown', onPointerDown, true);
  window.addEventListener('focusin', onFocusIn, true);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('keyup', onKeyUp);
  window.removeEventListener('blur', micUp);
  window.removeEventListener('pointerdown', onPointerDown, true);
  window.removeEventListener('focusin', onFocusIn, true);
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
          <button
            v-if="state.view === 'ui'"
            class="view-tab canvas-mode"
            :class="{ active: canvasMode.mode.value === 'preview' }"
            :aria-pressed="canvasMode.mode.value === 'preview'"
            title="switch the canvas between structure and preview (p)"
            @click="canvasMode.toggle()"
          >
            {{ canvasMode.mode.value === 'preview' ? 'Preview' : 'Structure' }} <kbd>p</kbd>
          </button>
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
      <DocsView v-else-if="state.view === 'docs'" />
      <ComponentsView v-else />
    </div>
    <aside class="sidebar"><SidebarPanel /></aside>
    <HelpModal v-if="state.helpOpen" />
  </div>
</template>
