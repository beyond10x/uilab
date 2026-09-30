<script setup lang="ts">
import { computed, watchEffect } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { columnsOf, fieldsOf, isDraftView, propsOf } from '../lib/outline.ts';
import { marks, requestRows, select, state, tint } from '../store.ts';

const props = defineProps<{ node: OutlineNode }>();

/** The composite kind: an overlay's kind reads `<presentation> <composite>`. */
const kind = computed(() => (props.node.layer === 'overlay' ? props.node.kind.split(' ').at(-1)! : props.node.kind));
const p = computed(() => propsOf(props.node));
const view = computed(() => props.node.view);
const draft = computed(() => isDraftView(view.value));
const needsRows = computed(() => ['collection', 'metric', 'record', 'chart'].includes(kind.value));
const rows = computed(() => (view.value ? state.rows[view.value] : undefined));
const rowObjects = computed<Record<string, unknown>[]>(() => {
  const all: unknown[] = rows.value?.rows ?? [];
  return all.filter((r) => !!r && typeof r === 'object' && !Array.isArray(r)) as Record<string, unknown>[];
});

watchEffect(() => {
  if (needsRows.value) requestRows(view.value);
});

const emptyLine = computed(() => {
  if (!view.value) return 'no data yet (no view)';
  if (!rows.value) return state.conn === 'open' ? `loading ${view.value}…` : `no data yet (${view.value})`;
  if (rowObjects.value.length === 0) return `no data yet (${view.value})`;
  return null;
});

const columns = computed(() => columnsOf(props.node));
const rowActions = computed(() => {
  const raw = p.value.row_actions;
  if (!Array.isArray(raw)) return [];
  return raw.map((a) => {
    const r = (a ?? {}) as Record<string, unknown>;
    return String(r.label ?? r.opens ?? r.does ?? 'action');
  });
});
const fields = computed(() => fieldsOf(props.node));
const filterFields = computed(() => {
  const f = fieldsOf(props.node, 'fields');
  if (f.length) return f;
  const g = fieldsOf(props.node, 'filters');
  return g.length ? g : [{ field: 'search', label: 'search' }];
});
const recordFields = computed(() => {
  if (fields.value.length) return fields.value;
  const first = rowObjects.value[0];
  return first ? Object.keys(first).map((k) => ({ field: k, label: k })) : [];
});
const metricValue = computed(() => {
  const from = typeof p.value.from === 'string' ? p.value.from : null;
  const first = rowObjects.value[0];
  if (!from || !first) return '—';
  return display(first[from]);
});
const moreRows = computed(() => {
  const total = rows.value?.total;
  return typeof total === 'number' && total > rowObjects.value.length ? total - rowObjects.value.length : 0;
});
/** A chart as horizontal bars: the label is `x` (or the first text field), the value the first
 *  series field (or the first number). Enough to see the shape, not a charting library. */
const chartBars = computed(() => {
  const first = rowObjects.value[0];
  if (!first) return [];
  const series = fieldsOf(props.node, 'series')[0]?.field;
  const x = typeof p.value.x === 'string' ? p.value.x : Object.keys(first).find((k) => typeof first[k] === 'string');
  const y = series ?? Object.keys(first).find((k) => typeof first[k] === 'number');
  if (!x || !y) return [];
  const values = rowObjects.value.map((r) => Number(r[y]) || 0);
  const max = Math.max(1, ...values);
  return rowObjects.value.map((r, i) => ({ label: display(r[x]), value: values[i], pct: Math.round((values[i] / max) * 100) }));
});
const title = computed(() => props.node.title);
const does = computed(() => (typeof p.value.does === 'string' ? p.value.does : null));

function display(v: unknown): string {
  if (v === null || v === undefined) return '';
  return typeof v === 'object' ? JSON.stringify(v) : String(v);
}
</script>

<template>
  <div
    class="card node"
    :class="[marks(node.path), { board: kind === 'board' }]" :style="tint(node.path)"
    :data-path="node.path"
    @click.stop="select(node.path)"
  >
    <div class="card-label">{{ node.name }} · {{ node.kind }}<span v-if="view" class="muted"> · {{ view }}</span><span v-if="draft && rowObjects.length" class="sample-tag" title="made-up rows: this view has no model binding yet">sample data</span></div>
    <h3 v-if="title" class="card-title">{{ title }}</h3>

    <template v-if="kind === 'collection'">
      <table v-if="columns.length" class="table">
        <thead>
          <tr>
            <th v-for="c in columns" :key="c.field">{{ c.field }}</th>
            <th v-if="rowActions.length"></th>
          </tr>
        </thead>
        <tbody v-if="!emptyLine">
          <tr v-for="(r, i) in rowObjects" :key="i">
            <td v-for="c in columns" :key="c.field">
              <span v-if="c.as === 'tag'" class="tag">{{ display(r[c.field]) }}</span>
              <template v-else>{{ display(r[c.field]) }}</template>
            </td>
            <td v-if="rowActions.length" class="actions">
              <button v-for="a in rowActions" :key="a" class="link" tabindex="-1">{{ a }}</button>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="emptyLine" class="empty">{{ emptyLine }}</p>
      <p v-else-if="moreRows" class="muted small">… {{ moreRows }} more of {{ rows?.total }}</p>
    </template>

    <template v-else-if="kind === 'metric'">
      <div class="metric">{{ emptyLine && draft ? '—' : metricValue }}</div>
      <p v-if="emptyLine" class="empty">{{ emptyLine }}</p>
    </template>

    <template v-else-if="kind === 'form'">
      <form class="form" @submit.prevent>
        <label v-for="f in fields" :key="f.field">
          <span>{{ f.label }}</span>
          <input type="text" :name="f.field" tabindex="-1" />
        </label>
        <p v-if="!fields.length" class="empty">no fields</p>
        <div class="form-actions">
          <button type="button" disabled>Submit</button>
          <span v-if="does" class="muted small">→ {{ does }}</span>
        </div>
      </form>
    </template>

    <template v-else-if="kind === 'record'">
      <dl class="record">
        <template v-for="f in recordFields" :key="f.field">
          <dt>{{ f.label }}</dt>
          <dd>{{ display(rowObjects[0]?.[f.field]) || '—' }}</dd>
        </template>
      </dl>
      <p v-if="emptyLine" class="empty">{{ emptyLine }}</p>
    </template>

    <template v-else-if="kind === 'filter_bar'">
      <div class="filter-bar">
        <input v-for="f in filterFields" :key="f.field" type="text" :placeholder="f.label" tabindex="-1" />
      </div>
    </template>

    <template v-else-if="kind === 'chart' && (chartBars.length || emptyLine)">
      <div v-if="chartBars.length" class="chart-bars">
        <div v-for="(b, i) in chartBars" :key="i" class="chart-bar">
          <span class="chart-label">{{ b.label }}</span>
          <span class="chart-fill" :style="{ width: b.pct + '%' }"></span>
          <span class="chart-value">{{ b.value }}</span>
        </div>
      </div>
      <p v-else class="empty">{{ emptyLine }}</p>
    </template>

    <template v-else-if="kind !== 'board'">
      <div class="placeholder">{{ kind }}<span v-if="view"> · {{ view }}</span></div>
    </template>

    <div v-if="node.children.length" class="children" :class="{ grid: kind === 'board' }">
      <CompositeView v-for="c in node.children" :key="c.path" :node="c" />
    </div>
  </div>
</template>
