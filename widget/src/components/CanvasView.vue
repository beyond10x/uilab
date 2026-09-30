<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { childrenOf, findNode, labelOf, navLayout, shellOf } from '../lib/outline.ts';
import { activePage, marks, select, shownOutline, showPage, state, tint } from '../store.ts';
import CompositeView from './CompositeView.vue';

const root = computed(() => shownOutline.value);
const page = computed(() => activePage.value);
const shell = computed(() => (root.value ? shellOf(root.value, page.value) : null));
const regions = computed(() => childrenOf(shell.value, 'region'));
const navRegion = computed(() => regions.value.find((r) => r.kind === 'navigation') ?? null);
const outletRegion = computed(() => regions.value.find((r) => r.kind === 'page_outlet') ?? null);
const barRegions = computed(() => regions.value.filter((r) => !['navigation', 'page_outlet'].includes(r.kind)));
const nav = computed(() => childrenOf(root.value, 'nav')[0] ?? null);
/** A shell without a navigation region shows no menu; with no shell at all the menu is shown. */
const showNav = computed(() => !!nav.value && (!shell.value || !!navRegion.value));
const navGroups = computed(() => (root.value ? navLayout(root.value) : []));
const sections = computed(() => childrenOf(page.value, 'section'));
const pageOverlays = computed(() => childrenOf(page.value, 'overlay'));
const shellOverlays = computed(() => childrenOf(shell.value, 'overlay'));
const overlay = computed(() => (state.openOverlay && root.value ? findNode(root.value, state.openOverlay) : null));

/** Where the pending change is, when it is not on the page shown. */
const changeElsewhere = computed(() => {
  const changed = state.proposal?.changed;
  if (!changed) return null;
  return changed.startsWith('page:') ? null : changed;
});

function openPage(p: OutlineNode): void {
  showPage(p.path);
  select(p.path);
}

function openOverlay(o: OutlineNode): void {
  state.openOverlay = o.path;
  select(o.path);
}
</script>

<template>
  <div v-if="!root" class="canvas-empty">
    <p>{{ state.conn === 'open' ? 'waiting for the document…' : 'connecting to the server…' }}</p>
  </div>
  <div v-else class="frame node" :class="marks(root.path)" :style="tint(root.path)" :data-path="root.path" @click="select(root.path)">
    <header class="frame-bar node" :class="shell ? marks(shell.path) : {}" :style="tint(shell?.path)" @click.stop="shell && select(shell.path)">
      <strong>{{ labelOf(root) }}</strong>
      <span class="muted small">{{ shell ? shell.name : 'no shell' }}</span>
      <span class="spacer"></span>
      <button
        v-for="r in barRegions"
        :key="r.path"
        class="chip node"
        :class="marks(r.path)" :style="tint(r.path)"
        @click.stop="select(r.path)"
      >
        {{ r.name }} <span class="muted">{{ r.kind }}</span>
      </button>
      <button
        v-for="o in shellOverlays"
        :key="o.path"
        class="chip node"
        :class="marks(o.path)" :style="tint(o.path)"
        @click.stop="openOverlay(o)"
      >
        ▢ {{ o.title || o.name }}
      </button>
    </header>
    <div class="frame-body">
      <nav
        v-if="showNav"
        class="nav-col node"
        :class="[navRegion ? marks(navRegion.path) : {}, nav ? marks(nav.path) : {}]" :style="tint(nav?.path) ?? tint(navRegion?.path)"
        @click.stop="nav ? select(nav.path) : navRegion && select(navRegion.path)"
      >
        <div class="card-label">nav{{ navRegion ? ` · ${navRegion.kind}` : '' }}</div>
        <div v-for="(g, gi) in navGroups" :key="g.section?.path ?? `rest-${gi}`" class="nav-group">
          <div
            v-if="g.section"
            class="nav-heading node"
            :class="marks(g.section.path)" :style="tint(g.section.path)"
            @click.stop="select(g.section.path)"
          >
            {{ g.section.title || g.section.name }}
          </div>
          <div v-else-if="navGroups.length > 1" class="nav-heading muted">pages</div>
          <a
            v-for="e in g.entries"
            :key="e.page.path"
            href="#"
            class="nav-page node"
            :class="[marks(e.page.path), { current: page?.path === e.page.path }]" :style="tint(e.page.path)"
            @click.stop.prevent="openPage(e.page)"
          >
            {{ e.page.title || e.page.name }}
            <span v-if="e.fromView" class="muted small">per row of {{ e.fromView }}</span>
          </a>
        </div>
      </nav>
      <main class="outlet node" :class="outletRegion ? marks(outletRegion.path) : {}" :style="tint(outletRegion?.path)">
        <p v-if="changeElsewhere" class="change-note">change at <code>{{ changeElsewhere }}</code></p>
        <div
          v-if="page"
          class="page node"
          :class="marks(page.path)" :style="tint(page.path)"
          :data-path="page.path"
          @click.stop="select(page.path)"
        >
          <div class="card-label">{{ page.name }} · {{ page.kind }}</div>
          <h1 class="page-title">{{ page.title || page.name }}</h1>
          <div v-if="pageOverlays.length" class="overlay-buttons">
            <button
              v-for="o in pageOverlays"
              :key="o.path"
              class="chip node"
              :class="marks(o.path)" :style="tint(o.path)"
              @click.stop="openOverlay(o)"
            >
              ▢ {{ o.title || o.name }} <span class="muted">{{ o.kind }}</span>
            </button>
          </div>
          <CompositeView v-for="s in sections" :key="s.path" :node="s" />
          <p v-if="!sections.length" class="empty">no sections</p>
        </div>
        <p v-else class="empty">no pages</p>
      </main>
    </div>
    <div v-if="overlay" class="modal-backdrop" @click.stop="state.openOverlay = null">
      <div class="modal" @click.stop>
        <div class="modal-bar">
          <span class="muted small">{{ overlay.kind }}</span>
          <button class="link" @click="state.openOverlay = null">close ✕</button>
        </div>
        <CompositeView :node="overlay" />
      </div>
    </div>
  </div>
</template>
