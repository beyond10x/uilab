import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { PRIMITIVE_KINDS, displayValue, isPrimitive, paramsOf, previewNode, sampleValue, type Param, type RowOf } from './components.ts';
import { childrenOf, propsOf } from './outline.ts';

/** What an instance's `row` and `rows.…` args read: the item row it is drawn for, and the rows of
 *  the composite it sits in (its own, when it reads a view). */
export interface Scope {
  row?: Record<string, unknown>;
  rows?: Record<string, unknown>[];
}

/** The widget an instance draws, as the canvas needs it. */
export interface InstanceWidget {
  name: string;
  arrange: string;
  params: Param[];
  body: OutlineNode[];
}

/** The 14 built-in composite kinds of `ui-spec/1`: a composite of one of these is no instance. */
export const COMPOSITE_KINDS = [
  'collection',
  'record',
  'form',
  'choice',
  'filter_bar',
  'header',
  'overlay',
  'confirm',
  'metric',
  'chart',
  'board',
  'graph_editor',
  'rich_text',
  'references',
];

/** A composite's kind: an overlay's outline kind reads `<presentation> <composite>`. */
export function compositeKind(node: OutlineNode): string {
  return node.layer === 'overlay' ? node.kind.split(' ').at(-1)! : node.kind;
}

function record(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

/**
 * The widget `node` instantiates, or `null` when it is none: a built-in composite, a primitive, or
 * an instance of a widget in `within` (the widgets it is drawn inside), which is not expanded again.
 * A node of a widget body or an item list whose kind is a primitive kind is that primitive unless
 * it writes `args`: a primitive `badge` and an instance of a widget `badge` share the outline kind.
 */
export function widgetOfInstance(root: OutlineNode, node: OutlineNode, within: string[] = []): InstanceWidget | null {
  const kind = compositeKind(node);
  if (COMPOSITE_KINDS.includes(kind) || within.includes(kind)) return null;
  const leaf = node.layer === 'node' || node.layer === 'item';
  if (leaf && PRIMITIVE_KINDS.includes(kind) && !('args' in propsOf(node))) return null;
  const widget = childrenOf(root, 'component').find((c) => c.name === kind);
  if (!widget) return null;
  const arrange = propsOf(widget).arrange;
  return {
    name: widget.name,
    arrange: typeof arrange === 'string' ? arrange : 'column',
    params: paramsOf(widget),
    body: childrenOf(widget, 'node'),
  };
}

/** Whether a node of a widget body or an item list draws as a primitive: a primitive kind that
 *  writes no `args` (an instance of a widget named like a primitive writes them). */
export function drawsAsPrimitive(node: OutlineNode): boolean {
  return isPrimitive(node) && !('args' in propsOf(node));
}

/** The value at `fields` below `value`, or not found. */
function dig(value: unknown, fields: string[]): { bound: boolean; value?: unknown } {
  let at: unknown = value;
  for (const f of fields) {
    const map = record(at);
    if (!map || !Object.prototype.hasOwnProperty.call(map, f)) return { bound: false };
    at = map[f];
  }
  return { bound: true, value: at };
}

/**
 * One written arg as the instance binds it in `scope`. `row` (and `row.<field>…`) reads the item
 * row; `rows` the composite's rows, `rows.first` or `rows.<n>` (from 0) one of them, and a field
 * below it. `args` (and `args.<param>…`) is a reference to a holder's args: a widget body resolves
 * it before its instances bind (`previewNode`), so one left here has no holder, as on a page, and
 * is unbound. Anything else is a literal and binds as written. A reference with nothing to read
 * (no row, no rows, an index past the end, a missing field) is unbound.
 */
export function bindArg(written: unknown, scope: Scope): { bound: boolean; value?: unknown } {
  if (typeof written !== 'string') return { bound: true, value: written };
  const [head, ...rest] = written.split('.');
  if (head === 'args') return { bound: false };
  if (head === 'row') return scope.row ? dig(scope.row, rest) : { bound: false };
  if (head !== 'rows') return { bound: true, value: written };
  const rows = scope.rows;
  if (!rows) return { bound: false };
  if (!rest.length) return { bound: true, value: rows };
  const [pick, ...fields] = rest;
  const index = pick === 'first' ? 0 : /^\d+$/.test(pick) ? Number(pick) : -1;
  if (index < 0 || index >= rows.length) return { bound: false };
  return dig(rows[index], fields);
}

/** The args an instance draws its widget's body with: each param as its `args` bind it in
 *  `scope` (`bindArg`), else its sample value (`sampleValue`); args no param declares, as bound. */
export function instanceArgs(widget: InstanceWidget, node: OutlineNode, scope: Scope, rowOf?: RowOf): Record<string, unknown> {
  const written = record(propsOf(node).args) ?? {};
  const out: Record<string, unknown> = {};
  for (const p of widget.params) {
    const b = Object.prototype.hasOwnProperty.call(written, p.name) ? bindArg(written[p.name], scope) : { bound: false };
    out[p.name] = b.bound ? b.value : sampleValue(p, rowOf);
  }
  for (const [name, value] of Object.entries(written)) {
    if (name in out) continue;
    const b = bindArg(value, scope);
    if (b.bound) out[name] = b.value;
  }
  return out;
}

/** The instance's widget body as the Components tab previews it (`previewNode`), read from the
 *  instance's args (`instanceArgs`). */
export function instanceBody(widget: InstanceWidget, node: OutlineNode, scope: Scope, rowOf?: RowOf): OutlineNode[] {
  const args = instanceArgs(widget, node, scope, rowOf);
  return widget.body.map((b) => previewNode(b, args));
}

/** The scopes a composite's item list is drawn in: a collection's once per row, a record's for
 *  its first row; none while there are no rows, loaded or not. */
export function itemScopes(kind: string, rows: Record<string, unknown>[]): Scope[] {
  const shown = kind === 'record' ? rows.slice(0, 1) : rows;
  return shown.map((row) => ({ row, rows }));
}

const WHOLE_ROW = /^row((?:\.[A-Za-z_]\w*)*)$/;
const EMBEDDED_ROW = /(?<![\w.])row((?:\.[A-Za-z_]\w*)+)/g;

/** What `row.<field>…` reads from `row`, as `args.<param>.<field>…` reads a body's args: a field
 *  the row does not carry as its own reads as its name. */
function readRow(row: Record<string, unknown>, rest: string): unknown {
  let value: unknown = row;
  for (const field of rest.split('.').filter(Boolean)) {
    const map = record(value);
    if (map && Object.prototype.hasOwnProperty.call(map, field)) value = map[field];
    else return field;
  }
  return value;
}

const WHOLE_ROWS = /^rows\.(first|\d+)((?:\.[A-Za-z_]\w*)*)$/;
const EMBEDDED_ROWS = /(?<![\w.])rows\.(first|\d+)((?:\.[A-Za-z_]\w*)*)/g;

/** What `rows.<pick>(.<field>)*` reads from `rows`, as `row.…` reads one row: a row past the end
 *  reads as an empty row. */
function readRows(rows: Record<string, unknown>[], pick: string, rest: string): unknown {
  const row = rows[pick === 'first' ? 0 : Number(pick)] ?? {};
  return readRow(row, rest);
}

/** `value` with every `row(.<field>)*` read from `row` and every `rows.first|<n>(.<field>)*` from
 *  `rows` (when given), however deep, as `substitute` reads `args`: a string that is only a
 *  reference becomes the value it reads, one inside longer text that value as text
 *  (`displayValue`). Keys are not rewritten. */
function substituteRow(value: unknown, row: Record<string, unknown>, rows?: Record<string, unknown>[]): unknown {
  if (typeof value === 'string') {
    const wholeRows = rows ? WHOLE_ROWS.exec(value) : null;
    if (wholeRows && rows) return readRows(rows, wholeRows[1], wholeRows[2]);
    const whole = WHOLE_ROW.exec(value);
    if (whole) return readRow(row, whole[1]);
    const withRows = rows
      ? value.replace(EMBEDDED_ROWS, (_text, pick: string, rest: string) => displayValue(readRows(rows, pick, rest)))
      : value;
    return withRows.replace(EMBEDDED_ROW, (_text, rest: string) => displayValue(readRow(row, rest)));
  }
  if (Array.isArray(value)) return value.map((v) => substituteRow(v, row, rows));
  const map = record(value);
  if (map) return Object.fromEntries(Object.entries(map).map(([k, v]) => [k, substituteRow(v, row, rows)]));
  return value;
}

/** An item as its composite draws it for one row: `row` and `row.<field>`, and with `rows` given
 *  `rows.first|<n>.<field>`, in its props and title read from them, and its children's, the way
 *  `previewNode` reads a body's args. */
export function rowNode(node: OutlineNode, row: Record<string, unknown>, rows?: Record<string, unknown>[]): OutlineNode {
  const out: OutlineNode = { ...node, children: node.children.map((c) => rowNode(c, row, rows)) };
  if (node.props !== undefined) out.props = substituteRow(node.props, row, rows) as OutlineNode['props'];
  if (node.title !== undefined) out.title = displayValue(substituteRow(node.title, row, rows));
  return out;
}

/** Whether an instance writes a `rows` reference its `scope` has nothing for (no rows loaded, or
 *  an index past the end): its body would be drawn from sample args and read as real data, so the
 *  canvas shows the empty line. A `row` with no item row around it is a placeholder at design
 *  time and keeps the sample. */
export function missingReference(node: OutlineNode, scope: Scope): boolean {
  const written = record(propsOf(node).args) ?? {};
  return Object.values(written).some(
    (v) => typeof v === 'string' && /^rows(\.|$)/.test(v) && !bindArg(v, scope).bound,
  );
}
