<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue';
import {
  componentsOf,
  entityViews,
  filterComponents,
  fixtureRowOf,
  isPrimitive,
  previewNode,
  sampleArgs,
  useSiteTarget,
  viewsRead,
  type ComponentInfo,
  type UseSite,
} from '../lib/components.ts';
import { isRemoved, marks, requestRows, select, selectNearest, showPage, shownOutline, state, tint } from '../store.ts';
import CompositeView from './CompositeView.vue';
import PrimitiveView from './PrimitiveView.vue';

const query = ref('');
const all = computed(() => (shownOutline.value ? componentsOf(shownOutline.value) : []));
const shown = computed(() => filterComponents(all.value, query.value));

/** The views the document reads: where an entity param's sample row comes from. */
const views = computed(() => (shownOutline.value ? viewsRead(shownOutline.value) : []));
const rowOf = computed(() => fixtureRowOf(views.value, state.rows));

watchEffect(() => {
  for (const c of all.value) for (const view of entityViews(c.params, views.value)) requestRows(view);
});

/** Each widget's body as its preview draws it, read from its sample args. */
function preview(c: ComponentInfo) {
  const args = sampleArgs(c.params, rowOf.value);
  return c.body.map((b) => previewNode(b, args));
}

function defaultText(value: unknown): string {
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/** Shows a use site: its page in the UI tab, with the overlay it sits in open, and the site
 *  selected. A site on no page (inside another widget) is only selected. */
function openUse(u: UseSite): void {
  const target = useSiteTarget(u);
  selectNearest(target.select);
  if (!target.page) return;
  showPage(target.page);
  if (target.overlay) state.openOverlay = target.overlay;
  state.view = 'ui';
}
</script>

<template>
  <div class="text-view components-view">
    <div class="text-view-bar">
      <input v-model="query" type="text" class="components-search" placeholder="search name, summary, params" aria-label="search components" />
      <span class="spacer"></span>
      <span class="muted small">{{ shown.length }} of {{ all.length }}</span>
    </div>
    <p v-if="!all.length" class="empty components-empty">No components yet. Select <code>/</code> and say 'make a reusable … card'.</p>
    <p v-else-if="!shown.length" class="empty components-empty">No component matches “{{ query }}”.</p>
    <div v-else class="comp-gallery">
      <div
        v-for="c in shown"
        :key="c.path"
        class="comp card node"
        :class="[marks(c.path), { removed: isRemoved(c.path) }]"
        :style="tint(c.path)"
        :data-path="c.path"
        @click.stop="select(c.path)"
      >
        <div class="comp-preview" :class="`arrange-${c.arrange}`">
          <template v-for="b in preview(c)" :key="b.path">
            <PrimitiveView v-if="isPrimitive(b)" :node="b" />
            <CompositeView v-else :node="b" />
          </template>
          <p v-if="!c.body.length" class="empty">empty body</p>
        </div>

        <div class="comp-head">
          <h3 class="card-title">{{ c.name }}</h3>
          <span class="muted small">{{ c.arrange }}</span>
        </div>
        <p class="comp-summary">{{ c.summary }}</p>

        <details class="comp-details" @click.stop>
          <summary class="small muted">
            {{ c.params.length }} {{ c.params.length === 1 ? 'param' : 'params' }} ·
            {{ c.uses.length ? `used ${c.uses.length} ${c.uses.length === 1 ? 'time' : 'times'}` : 'not used yet' }}
          </summary>
          <table v-if="c.params.length" class="table comp-params">
            <thead>
              <tr><th>param</th><th>type</th><th>required</th><th>default</th></tr>
            </thead>
            <tbody>
              <tr v-for="p in c.params" :key="p.name">
                <td><code>{{ p.name }}</code></td>
                <td><code>{{ p.typeLabel }}</code></td>
                <td>{{ p.required ? 'yes' : 'no' }}</td>
                <td><code v-if="p.hasDefault">{{ defaultText(p.default) }}</code><span v-else class="muted">-</span></td>
              </tr>
            </tbody>
          </table>
          <p v-else class="muted small">No params.</p>

          <div class="comp-uses small">
            <template v-if="c.uses.length">
              <span>used at:</span>
              <a
                v-for="(u, i) in c.uses"
                :key="`${u.path}#${u.trail ?? ''}#${i}`"
                href="#"
                class="comp-use"
                :title="useSiteTarget(u).page ? `show ${useSiteTarget(u).page} in the UI tab` : 'select it'"
                @click.prevent.stop="openUse(u)"
              >
                <code>{{ u.path }}</code><span v-if="u.trail" class="muted"> ({{ u.trail }})</span>
              </a>
            </template>
            <span v-else class="muted">Not used yet.</span>
          </div>
          <div class="card-label">{{ c.path }}</div>
        </details>
      </div>
    </div>
  </div>
</template>

<style scoped>
.comp-gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
  padding: 12px;
  align-items: start;
}

.components-view .comp-gallery .comp {
  margin: 0;
}

.comp-gallery .comp-preview {
  margin-bottom: 10px;
}

.comp-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.comp-head .card-title {
  margin: 0;
}

.comp-details > summary {
  cursor: pointer;
}

.comp-details[open] > summary {
  margin-bottom: 6px;
}

.comp-details .comp-params {
  width: auto;
}

.comp-use code {
  text-decoration: underline;
}
</style>
