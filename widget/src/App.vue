<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue';
import { accept, agentActions, micDown, micUp, reject, selectNearest, state, undo } from './store.ts';
import CanvasView from './components/CanvasView.vue';
import SidebarPanel from './components/SidebarPanel.vue';

/** Whether keyboard focus is where typed keys belong to the element, not to the app. */
function typing(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el || !el.tagName) return false;
  return el.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName);
}

function onKeyDown(ev: KeyboardEvent): void {
  if (typing(ev.target)) return;
  if (ev.code === 'Space') {
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
      <div v-if="agentActions.length" class="op-banners">
        <div
          v-for="a in agentActions"
          :key="a.op.id"
          class="op-banner"
          :style="{ '--op-colour': a.op.colour }"
          @click="selectNearest(a.target)"
        >
          🤖 <strong>{{ a.op.name }}</strong> is operating on <code>{{ a.target }}</code>
        </div>
      </div>
      <CanvasView />
    </div>
    <aside class="sidebar"><SidebarPanel /></aside>
  </div>
</template>
