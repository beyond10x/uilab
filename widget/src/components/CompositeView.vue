<script setup lang="ts">
import { computed, watchEffect } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { canvasMode, emptyLine as emptyLineText } from '../lib/canvasmode.ts';
import { entityViews, fixtureRowOf, viewsRead } from '../lib/components.ts';
import { bodyNodes, compositeKind, drawsAsPrimitive, instanceBody, itemScopes, missingReference, rowNode, widgetOfInstance, type Scope } from '../lib/instance.ts';
import { columnsOf, fieldsOf, isDraftView, nodeClasses, propsOf } from '../lib/outline.ts';
import { isSample } from '../lib/rows.ts';
import { marks, requestRows, select, shownOutline, state, tint } from '../store.ts';
import PrimitiveView from './PrimitiveView.vue';

/** `scope`: what a widget instance's `row` and `rows.…` args read here, given by the composite
 *  this one sits in. `within`: the widgets it is drawn inside, which are not expanded again. */
const props = defineProps<{ node: OutlineNode; scope?: Scope; within?: string[] }>();

const kind = computed(() => compositeKind(props.node));
const p = computed(() => propsOf(props.node));
const view = computed(() => props.node.view);
const draft = computed(() => isDraftView(view.value));
/** Preview drops the `name · kind · view` label; the sample-data tag stays in both modes. */
const preview = computed(() => canvasMode.mode.value === 'preview');
const root = computed(() => shownOutline.value);
const within = computed(() => props.within ?? []);
/** The widget this node instantiates, drawn as its body with the node's args bound. */
const instance = computed(() => (root.value ? widgetOfInstance(root.value, props.node, within.value) : null));
const needsRows = computed(() => ['collection', 'metric', 'record', 'chart'].includes(kind.value) || !!instance.value);
const rows = computed(() => (view.value ? state.rows[view.value] : undefined));
const rowObjects = computed<Record<string, unknown>[]>(() => {
  const all: unknown[] = rows.value?.rows ?? [];
  return all.filter((r) => !!r && typeof r === 'object' && !Array.isArray(r)) as Record<string, unknown>[];
});
/** The rows `rows.…` reads here: this composite's own when it reads a view, else those it sits in. */
const scopeRows = computed(() => (view.value ? rowObjects.value : props.scope?.rows));
const childScope = computed<Scope>(() => ({ rows: scopeRows.value }));
const items = computed(() => props.node.children.filter((c) => c.layer === 'item'));
/** Children drawn after the composite: not its items, and not an instance's body, which is drawn as
 *  the instance. */
const others = computed(() => {
  const body = new Set(bodyNodes(props.node));
  return props.node.children.filter((c) => c.layer !== 'item' && !body.has(c));
});
const scopes = computed(() => itemScopes(kind.value, scopeRows.value ?? []));

const views = computed(() => (root.value ? viewsRead(root.value) : []));
const body = computed(() => {
  if (!instance.value) return [];
  const scope: Scope = { row: props.scope?.row, rows: scopeRows.value };
  return instanceBody(instance.value, props.node, scope, fixtureRowOf(views.value, state.rows));
});
const bodyWithin = computed(() => (instance.value ? [...within.value, instance.value.name] : within.value));
/** An instance whose `row`/`rows` args have nothing to read here: shown as the empty line, not as
 *  a body of sample args that would read as real data. */
const unbound = computed(() => !!instance.value && missingReference(props.node, { row: props.scope?.row, rows: scopeRows.value }));
const unboundLine = computed(() => (view.value && !rows.value ? (preview.value ? 'loading…' : `loading ${view.value}…`) : 'no data yet'));

/** Rows the server made up for a read no fixture answers (`Rows.sample`), or rows of a `draft.` view. */
const sampled = computed(() => isSample(rows.value) || (draft.value && rowObjects.value.length > 0));

watchEffect(() => {
  if (needsRows.value) requestRows(view.value);
  if (instance.value) for (const v of entityViews(instance.value.params, views.value)) requestRows(v);
});

const emptyLine = computed(() =>
  emptyLineText({
    view: view.value,
    loaded: !!rows.value,
    count: rowObjects.value.length,
    connOpen: state.conn === 'open',
    preview: preview.value,
  }),
);

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
    :class="[marks(node.path), nodeClasses(node), { board: kind === 'board' }]" :style="tint(node.path)"
    :data-path="node.path"
    @click.stop="select(node.path)"
  >
    <div v-if="!preview || sampled" class="card-label"><template v-if="!preview">{{ node.name }} · {{ node.kind }}<span v-if="view" class="muted"> · {{ view }}</span></template><span v-if="sampled" class="sample-tag" title="made-up rows: no fixture answers this read yet">sample data</span></div>
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
          <span v-if="does && !preview" class="muted small">→ {{ does }}</span>
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

    <template v-else-if="instance">
      <p v-if="unbound" class="empty">{{ unboundLine }}</p>
      <div v-else class="instance-body" :class="`arrange-${instance.arrange}`" :data-widget="instance.name">
        <template v-for="b in body" :key="b.path">
          <PrimitiveView v-if="drawsAsPrimitive(b)" :node="b" />
          <CompositeView v-else :node="b" :scope="{ row: scope?.row, rows: scopeRows }" :within="bodyWithin" />
        </template>
        <p v-if="!body.length" class="empty">empty body</p>
      </div>
    </template>

    <template v-else-if="kind !== 'board'">
      <div class="placeholder">{{ kind }}<span v-if="view && !preview"> · {{ view }}</span></div>
    </template>

    <div v-if="items.length" class="item-rows">
      <div v-for="(s, i) in scopes" :key="i" class="item-row">
        <template v-for="c in items" :key="c.path">
          <PrimitiveView v-if="drawsAsPrimitive(c)" :node="s.row ? rowNode(c, s.row, s.rows) : c" />
          <CompositeView v-else :node="s.row ? rowNode(c, s.row, s.rows) : c" :scope="s" :within="within" />
        </template>
      </div>
    </div>
    <div v-if="others.length" class="children" :class="{ grid: kind === 'board' }">
      <template v-for="c in others" :key="c.path">
        <PrimitiveView v-if="drawsAsPrimitive(c)" :node="c" />
        <CompositeView v-else :node="c" :scope="childScope" :within="within" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.instance-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.instance-body.arrange-row {
  flex-direction: row;
  flex-wrap: wrap;
  align-items: flex-start;
}

.instance-body.arrange-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}

.instance-body > .card {
  margin-bottom: 0;
}

.item-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.item-row {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 8px;
  padding: 8px;
  border-top: 1px solid var(--line);
}

.item-row:first-child {
  border-top: 0;
}

.item-row > .card {
  margin-bottom: 0;
  flex: 1 1 220px;
}
</style>
