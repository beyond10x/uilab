<script setup lang="ts">
import { computed, watchEffect } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { accountChrome, canvasMode } from '../lib/canvasmode.ts';
import { primitiveWords } from '../lib/components.ts';
import { drawsAsPrimitive, instanceBody, widgetOfInstance } from '../lib/instance.ts';
import { childrenOf, findNode, headerOf, labelOf, navLabelOf, navLayout, nodeClasses, shellOf, type HeaderMetric } from '../lib/outline.ts';
import { activePage, marks, requestRows, select, shownOutline, showPage, state, tint } from '../store.ts';
import CompositeView from './CompositeView.vue';

/** Preview draws the shell as app chrome and drops the structure labels. */
const preview = computed(() => canvasMode.mode.value === 'preview');

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
/** The page's header as ESS renders it: title, help text, action texts. */
const header = computed(() => (page.value ? headerOf(page.value) : null));
const pageOverlays = computed(() => childrenOf(page.value, 'overlay'));
const shellOverlays = computed(() => childrenOf(shell.value, 'overlay'));
const overlay = computed(() => (state.openOverlay && root.value ? findNode(root.value, state.openOverlay) : null));

watchEffect(() => {
  if (!preview.value) return;
  for (const r of barRegions.value) if (r.kind === 'account_menu') requestRows(r.view);
});

/** Rows of the views the header's headline metrics read. */
watchEffect(() => {
  for (const m of header.value?.metrics ?? []) if (m.view) requestRows(m.view);
});

/** The words a headline metric that is a widget instance shows: its widget's body primitives, bound
 *  to its args. */
function metricBody(m: HeaderMetric): string[] {
  if (!root.value || !page.value) return [];
  const node: OutlineNode = { path: page.value.path, layer: 'section', name: m.name, kind: m.component, children: [], props: (m.args ? { args: m.args } : {}) as OutlineNode['props'] };
  const widget = widgetOfInstance(root.value, node);
  if (!widget) return [];
  return instanceBody(widget, node, {})
    .filter(drawsAsPrimitive)
    .map(primitiveWords)
    .filter((w) => w !== '');
}

/** A headline metric's value: its `from` field of the first row its view returned, else `—`. */
function metricValue(m: HeaderMetric): string {
  const first = m.view ? state.rows[m.view]?.rows?.[0] : undefined;
  const v = m.from && first && typeof first === 'object' ? (first as Record<string, unknown>)[m.from] : undefined;
  return v === undefined || v === null ? '—' : typeof v === 'object' ? JSON.stringify(v) : String(v);
}

/** The account menu as chrome: the staff member's name from the rows its view reads, when loaded. */
function account(r: OutlineNode): ReturnType<typeof accountChrome> {
  return accountChrome(r.view, r.view ? state.rows[r.view]?.rows : undefined, r.title || 'Account');
}

/** Whether a node carries a mark worth seeing: selected, part of the proposal, or being worked on. */
function marked(path: string): boolean {
  return Object.values(marks(path)).some(Boolean);
}

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
  <div v-else class="frame node" :class="[marks(root.path), { preview }]" :style="tint(root.path)" :data-path="root.path" @click="select(root.path)">
    <header class="frame-bar node" :class="shell ? marks(shell.path) : {}" :style="tint(shell?.path)" @click.stop="shell && select(shell.path)">
      <strong>{{ labelOf(root) }}</strong>
      <span v-if="!preview" class="muted small">{{ shell ? shell.name : 'no shell' }}</span>
      <span class="spacer"></span>
      <template v-for="r in barRegions" :key="r.path">
        <button
          v-if="!preview || (r.kind === 'overlay_outlet' && marked(r.path))"
          class="chip node"
          :class="marks(r.path)" :style="tint(r.path)"
          @click.stop="select(r.path)"
        >
          {{ r.name }} <span class="muted">{{ r.kind }}</span>
        </button>
        <button
          v-else-if="r.kind === 'account_menu'"
          class="chrome-account node"
          :class="marks(r.path)" :style="tint(r.path)"
          :title="r.name"
          :aria-label="account(r).sample ? `${account(r).name}, sample data` : account(r).name"
          @click.stop="select(r.path)"
        >
          <span class="avatar" aria-hidden="true">{{ account(r).initial }}</span>
          {{ account(r).name }}
          <span v-if="account(r).sample" class="sample-tag" title="made-up rows: this view has no model binding yet">sample data</span>
          <span class="caret" aria-hidden="true">▾</span>
        </button>
        <button
          v-else-if="r.kind === 'notifications'"
          class="chrome-bell node"
          :class="marks(r.path)" :style="tint(r.path)"
          :title="r.title || 'notifications'"
          :aria-label="r.title || 'notifications'"
          @click.stop="select(r.path)"
        >
          🔔
        </button>
        <button
          v-else-if="r.kind !== 'overlay_outlet'"
          class="chrome-region node"
          :class="marks(r.path)" :style="tint(r.path)"
          @click.stop="select(r.path)"
        >
          {{ r.title || r.name }}
        </button>
      </template>
      <button
        v-for="o in shellOverlays"
        :key="o.path"
        :class="[preview ? 'chrome-region' : 'chip', 'node', marks(o.path), nodeClasses(o)]" :style="tint(o.path)"
        @click.stop="openOverlay(o)"
      >
        <template v-if="!preview">▢ </template>{{ o.title || o.name }}
      </button>
    </header>
    <div class="frame-body">
      <nav
        v-if="showNav"
        class="nav-col node"
        :class="[navRegion ? marks(navRegion.path) : {}, nav ? marks(nav.path) : {}]" :style="tint(nav?.path) ?? tint(navRegion?.path)"
        @click.stop="nav ? select(nav.path) : navRegion && select(navRegion.path)"
      >
        <div v-if="!preview" class="card-label">nav{{ navRegion ? ` · ${navRegion.kind}` : '' }}</div>
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
            {{ navLabelOf(e.page) }}
            <span v-if="e.fromView && !preview" class="muted small">per row of {{ e.fromView }}</span>
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
          <div v-if="!preview" class="card-label">{{ page.name }} · {{ page.kind }}</div>
          <h1 class="page-title">{{ header?.title }}</h1>
          <p v-if="header?.help || header?.helpLink" class="page-help muted small">
            <template v-if="header?.help">{{ header.help }} </template>
            <a v-if="header?.helpLink" :href="header.helpLink" target="_blank" rel="noopener" tabindex="-1">{{ header.helpLink }}</a>
          </p>
          <div v-if="header?.actions.length" class="page-actions">
            <button v-for="(a, i) in header.actions" :key="i" type="button" tabindex="-1">{{ a }}</button>
          </div>
          <div v-if="header?.metrics.length" class="page-metrics">
            <div v-for="m in header.metrics" :key="m.name" class="page-metric">
              <div v-if="m.label" class="metric-label">{{ m.label }}</div>
              <div v-if="m.component === 'metric'" class="page-metric-value">{{ metricValue(m) }}</div>
              <div v-else-if="metricBody(m).length" class="page-metric-body"><span v-for="(w, i) in metricBody(m)" :key="i">{{ w }}</span></div>
              <div v-if="!preview" class="muted small">{{ m.name }} · {{ m.component }}</div>
            </div>
          </div>
          <div v-if="pageOverlays.length" class="overlay-buttons">
            <button
              v-for="o in pageOverlays"
              :key="o.path"
              :class="[preview ? 'chrome-region' : 'chip', 'node', marks(o.path), nodeClasses(o)]" :style="tint(o.path)"
              @click.stop="openOverlay(o)"
            >
              <template v-if="preview">{{ o.title || o.name }}</template>
              <template v-else>▢ {{ o.title || o.name }} <span class="muted">{{ o.kind }}</span></template>
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
          <span v-if="preview" class="small">{{ overlay.title || overlay.name }}</span>
          <span v-else class="muted small">{{ overlay.kind }}</span>
          <button class="link" @click="state.openOverlay = null">close ✕</button>
        </div>
        <CompositeView :node="overlay" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.chrome-account,
.chrome-bell,
.chrome-region {
  border-color: transparent;
  background: transparent;
  font-size: 13px;
  padding: 3px 8px;
  border-radius: 16px;
}

.chrome-account:hover,
.chrome-bell:hover,
.chrome-region:hover {
  background: #f1f3f6;
}

.chrome-account {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.avatar {
  display: inline-grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
}

.caret {
  color: var(--muted);
  font-size: 10px;
}

.page-help {
  margin: -4px 0 8px;
}

.page-metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 10px;
}

.page-metric .metric-label {
  font-size: 12px;
  color: var(--muted);
}

.page-metric-body {
  display: flex;
  gap: 6px;
}

.page-metric-value {
  font-size: 22px;
  font-weight: 600;
}

.page-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}

.chrome-bell {
  font-size: 15px;
  line-height: 1;
  padding: 4px 6px;
}
</style>
