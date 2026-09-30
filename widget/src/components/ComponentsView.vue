<script setup lang="ts">
import { computed, ref } from 'vue';
import { componentsOf, filterComponents, isPrimitive, previewNode, sampleArgs, type ComponentInfo } from '../lib/components.ts';
import { isRemoved, marks, select, selectNearest, shownOutline, tint } from '../store.ts';
import CompositeView from './CompositeView.vue';
import PrimitiveView from './PrimitiveView.vue';

const query = ref('');
const all = computed(() => (shownOutline.value ? componentsOf(shownOutline.value) : []));
const shown = computed(() => filterComponents(all.value, query.value));

/** Each widget's body as its preview draws it, read from its sample args. */
function preview(c: ComponentInfo) {
  const args = sampleArgs(c.params);
  return c.body.map((b) => previewNode(b, args));
}

function defaultText(value: unknown): string {
  return typeof value === 'string' ? value : JSON.stringify(value);
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
    <div
      v-for="c in shown"
      :key="c.path"
      class="comp card node"
      :class="[marks(c.path), { removed: isRemoved(c.path) }]"
      :style="tint(c.path)"
      :data-path="c.path"
      @click.stop="select(c.path)"
    >
      <div class="card-label">{{ c.path }} · {{ c.arrange }}</div>
      <h3 class="card-title">{{ c.name }}</h3>
      <p class="comp-summary">{{ c.summary }}</p>

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
          <span>used {{ c.uses.length }} {{ c.uses.length === 1 ? 'time' : 'times' }}:</span>
          <button
            v-for="(u, i) in c.uses"
            :key="`${u.path}#${u.trail ?? ''}#${i}`"
            class="link comp-use"
            @click.stop="selectNearest(u.path)"
          >
            <code>{{ u.path }}</code><span v-if="u.trail" class="muted"> ({{ u.trail }})</span>
          </button>
        </template>
        <span v-else class="muted">Not used yet.</span>
      </div>

      <div class="card-label">preview</div>
      <div class="comp-preview" :class="`arrange-${c.arrange}`">
        <template v-for="b in preview(c)" :key="b.path">
          <PrimitiveView v-if="isPrimitive(b)" :node="b" />
          <CompositeView v-else :node="b" />
        </template>
        <p v-if="!c.body.length" class="empty">empty body</p>
      </div>
    </div>
  </div>
</template>
