import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { childrenOf, isDraftView, overlayOf, pageOf, propsOf } from './outline.ts';

/** One declared param of a widget, as the outline carries it in the widget's `props.params`. */
export interface Param {
  name: string;
  /** The type as written: a primitive name, a named type or a constructor map. */
  type: unknown;
  typeLabel: string;
  required: boolean;
  /** Whether the declaration writes `default`, `null` included. */
  hasDefault: boolean;
  default: unknown;
}

/** Where a widget is instantiated: the node that holds the instance, and the way through that
 *  node's untyped props when the instance is not the node itself. */
export interface UseSite {
  path: string;
  trail?: string;
}

/** A widget of the document, as the Components tab lists it. */
export interface ComponentInfo {
  name: string;
  path: string;
  summary: string;
  arrange: string;
  node: OutlineNode;
  params: Param[];
  body: OutlineNode[];
  uses: UseSite[];
}

/** The nine primitive kinds of `ui-spec/1`. */
export const PRIMITIVE_KINDS = ['text', 'badge', 'icon', 'button', 'link', 'input', 'toggle', 'image', 'divider'];

/** Every widget of the document (layer `component` under the root), in document order. */
export function componentsOf(root: OutlineNode): ComponentInfo[] {
  return childrenOf(root, 'component').map((node) => {
    const arrange = propsOf(node).arrange;
    return {
      name: node.name,
      path: node.path,
      summary: node.title ?? '',
      arrange: typeof arrange === 'string' ? arrange : 'column',
      node,
      params: paramsOf(node),
      body: childrenOf(node, 'node'),
      uses: useSites(root, node.name),
    };
  });
}

function record(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

/** A widget's params in declaration order. */
export function paramsOf(widget: OutlineNode): Param[] {
  const params = record(propsOf(widget).params);
  if (!params) return [];
  return Object.entries(params).map(([name, raw]) => {
    const p = record(raw) ?? {};
    const hasDefault = Object.prototype.hasOwnProperty.call(p, 'default');
    return {
      name,
      type: p.type,
      typeLabel: typeLabel(p.type),
      required: p.required === true,
      hasDefault,
      default: hasDefault ? p.default : undefined,
    };
  });
}

/** A param type in one short line: `Loan`, `list<Loan>`, `enum<open | late>`. */
export function typeLabel(type: unknown): string {
  if (typeof type === 'string') return type;
  const map = record(type);
  if (!map) return type === undefined || type === null ? '?' : JSON.stringify(type);
  const entries = Object.entries(map);
  if (entries.length !== 1) return JSON.stringify(type);
  const [ctor, inner] = entries[0];
  const shown = Array.isArray(inner) ? inner.map((i) => (typeof i === 'string' ? i : typeLabel(i))).join(' | ') : typeLabel(inner);
  return `${ctor}<${shown}>`;
}

const TEXT_TYPES = ['string', 'text', 'markdown', 'date', 'datetime', 'time', 'url', 'email', 'id', 'uuid'];
const NUMBER_TYPES = ['number', 'integer', 'int', 'float', 'decimal'];
const BOOLEAN_TYPES = ['boolean', 'bool'];
const LIST_CTORS = ['list', 'array', 'many', 'set'];
const CHOICE_CTORS = ['enum', 'one_of'];
const WRAP_CTORS = ['optional', 'maybe', 'nullable'];

/**
 * A value to preview a param with. A declared `default` is used as written. An entity (a named
 * type such as `Member`) takes the row `rowOf` gives for it, the first fixture row of its view,
 * when there is one. Otherwise the value is shaped by the type and named after the param: text
 * types give the param name, number types 3, boolean types `true`, a list one sample element, an
 * enum its first member, and a record or entity (a named type, or any other constructor) an object
 * `{name: <param>}`.
 */
export function sampleValue(param: { name: string; type: unknown; hasDefault?: boolean; default?: unknown }, rowOf?: RowOf): unknown {
  if (param.hasDefault) return param.default;
  return shaped(param.name, param.type, rowOf);
}

function shaped(name: string, type: unknown, rowOf?: RowOf): unknown {
  if (typeof type === 'string') {
    const t = type.toLowerCase();
    if (TEXT_TYPES.includes(t)) return name;
    if (NUMBER_TYPES.includes(t)) return 3;
    if (BOOLEAN_TYPES.includes(t)) return true;
    if (!/^[A-Z]|\./.test(type)) return name;
  }
  const entity = entityOf(type);
  const row = entity && rowOf ? rowOf(entity) : undefined;
  const map = record(type);
  const entry = map && Object.entries(map).length === 1 ? Object.entries(map)[0] : null;
  if (entry) {
    const [ctor, inner] = entry;
    if (LIST_CTORS.includes(ctor)) return [shaped(name, inner, rowOf)];
    if (CHOICE_CTORS.includes(ctor) && Array.isArray(inner)) return inner[0] ?? name;
    if (WRAP_CTORS.includes(ctor)) return shaped(name, inner, rowOf);
  }
  if (row) return row;
  return type === undefined || type === null ? name : { name };
}

/** A row to preview an entity with, by entity name (`Member`), or none. */
export type RowOf = (entity: string) => Record<string, unknown> | undefined;

/**
 * The entity a param type refers to: a named type (`Member`, `library.Member` → `Member`), also
 * through a list, a wrapper or any other one-constructor map naming one (`{list: Loan}`,
 * `{record: Member}`); `null` for primitives, choices and anything else.
 */
export function entityOf(type: unknown): string | null {
  if (typeof type === 'string') {
    const t = type.toLowerCase();
    if (TEXT_TYPES.includes(t) || NUMBER_TYPES.includes(t) || BOOLEAN_TYPES.includes(t)) return null;
    if (!/^[A-Z]|\./.test(type)) return null;
    return type.split('.').at(-1) || null;
  }
  const map = record(type);
  const entries = map ? Object.entries(map) : [];
  if (entries.length !== 1) return null;
  const [ctor, inner] = entries[0];
  return CHOICE_CTORS.includes(ctor) ? null : entityOf(inner);
}

/** Plurals no suffix rule forms, by the word they end an entity name with. */
const IRREGULAR_PLURALS: Record<string, string> = { person: 'people', child: 'children' };

/** A view prefix or entity name as compared: lower case, underscores removed. */
function stemKey(name: string): string {
  return name.toLowerCase().replace(/_/g, '');
}

/** The view prefixes an entity's rows go by, as [`stemKey`] gives them: `Member` → `members`,
 *  `member`; `Category` → `categories`; `Box` → `boxes`; `LoanRequest` → `loanrequests`;
 *  `Person` → `people`. */
function viewStems(entity: string): string[] {
  const base = stemKey(entity);
  const irregular = Object.keys(IRREGULAR_PLURALS).find((word) => base.endsWith(word));
  const plural = irregular
    ? `${base.slice(0, -irregular.length)}${IRREGULAR_PLURALS[irregular]}`
    : /[^aeiou]y$/.test(base)
      ? `${base.slice(0, -1)}ies`
      : /(s|x|z|ch|sh)$/.test(base)
        ? `${base}es`
        : `${base}s`;
  return [plural, base];
}

/**
 * The view whose rows carry `entity`, among `views`: one whose first segment is the entity's name
 * in the plural or singular, case-insensitively and whatever its underscores (`Member` →
 * `members.*`, `LoanRequest` → `loan_requests.*`), `<prefix>.All` first, else the first such view
 * listed; `null` when none is.
 */
export function viewForEntity(entity: string, views: string[]): string | null {
  const stems = viewStems(entity);
  const matching = views.filter((v) => stems.includes(stemKey(v.split('.')[0])));
  return matching.find((v) => v.split('.').slice(1).join('.') === 'All') ?? matching[0] ?? null;
}

/**
 * Every view the outline's nodes read, once each, in document order; draft views (no rows) left
 * out. A node's `view` (a composite's, an overlay's, a shell region's) and a dynamic menu section's
 * `props.from_view`.
 */
export function viewsRead(root: OutlineNode): string[] {
  const out: string[] = [];
  const add = (view: unknown) => {
    if (typeof view === 'string' && view && !isDraftView(view) && !out.includes(view)) out.push(view);
  };
  const walk = (node: OutlineNode) => {
    add(node.view);
    if (node.layer === 'nav_section') add(propsOf(node).from_view);
    node.children.forEach(walk);
  };
  walk(root);
  return out;
}

/** The views, once each, whose rows the samples of `params` would read. */
export function entityViews(params: Param[], views: string[]): string[] {
  const out: string[] = [];
  for (const p of params) {
    const entity = entityOf(p.type);
    const view = entity ? viewForEntity(entity, views) : null;
    if (view && !out.includes(view)) out.push(view);
  }
  return out;
}

/** An entity's first fixture row: the first object row loaded for its view among `views`. */
export function fixtureRowOf(views: string[], rows: Record<string, { rows: unknown[] } | undefined>): RowOf {
  return (entity) => {
    const view = viewForEntity(entity, views);
    const loaded = view ? rows[view]?.rows : undefined;
    return (loaded ?? []).map(record).find((r) => r !== null) ?? undefined;
  };
}

/** Sample args for a widget: one per param, by [`sampleValue`]. */
export function sampleArgs(params: Param[], rowOf?: RowOf): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const p of params) out[p.name] = sampleValue(p, rowOf);
  return out;
}

/** Where a use site shows: the node to select, the page it sits on and the overlay it sits in.
 *  A site that is not on a page (inside a widget, a page kind) has no page to show; a site in a
 *  shell overlay has none either, and shows as that overlay over whichever page is shown. */
export function useSiteTarget(site: UseSite): { select: string; page: string | null; overlay: string | null } {
  const page = pageOf(site.path);
  const onCanvas = page !== null || site.path.startsWith('shell:');
  return { select: site.path, page, overlay: onCanvas ? overlayOf(site.path) : null };
}

const WHOLE_REF = /^args\.([A-Za-z_]\w*)((?:\.[A-Za-z_]\w*)*)$/;
const EMBEDDED_REF = /(?<![\w.])args\.([A-Za-z_]\w*)((?:\.[A-Za-z_]\w*)*)/g;

/** What `args.<param>.<field>…` reads from `args`. A field the sample does not carry reads as its
 *  own name, so a preview shows `due` for `args.loan.due`; an undeclared param is not found. */
function resolve(args: Record<string, unknown>, param: string, rest: string): { found: boolean; value?: unknown } {
  if (!Object.prototype.hasOwnProperty.call(args, param)) return { found: false };
  let value: unknown = args[param];
  for (const field of rest.split('.').filter(Boolean)) {
    const map = record(value);
    if (map && Object.prototype.hasOwnProperty.call(map, field)) value = map[field];
    else return { found: true, value: field };
  }
  return { found: true, value };
}

/** A value as one line of preview text: an object shows its `name`, else its `title`, else its
 *  `label`. */
export function displayValue(value: unknown): string {
  if (value === null || value === undefined) return '';
  if (typeof value !== 'object') return String(value);
  const map = record(value);
  const shown = map ? [map.name, map.title, map.label].find((v) => typeof v === 'string') : undefined;
  if (typeof shown === 'string') return shown;
  return JSON.stringify(value);
}

/**
 * `value` with every `args.<param>(.<field>)*` replaced from `args`, however deep. A string that
 * is only a reference becomes the value it reads (keeping its type); a reference inside longer
 * text is replaced by that value as text. Keys are not rewritten; unknown params stay as written.
 */
export function substitute(value: unknown, args: Record<string, unknown>): unknown {
  if (typeof value === 'string') {
    const whole = WHOLE_REF.exec(value);
    if (whole) {
      const r = resolve(args, whole[1], whole[2]);
      return r.found ? r.value : value;
    }
    return value.replace(EMBEDDED_REF, (text, param: string, rest: string) => {
      const r = resolve(args, param, rest);
      return r.found ? displayValue(r.value) : text;
    });
  }
  if (Array.isArray(value)) return value.map((v) => substitute(v, args));
  const map = record(value);
  if (map) return Object.fromEntries(Object.entries(map).map(([k, v]) => [k, substitute(v, args)]));
  return value;
}

/** A body node, and its children, as a preview draws it: props and title read from `args`. */
export function previewNode(node: OutlineNode, args: Record<string, unknown>): OutlineNode {
  const out: OutlineNode = { ...node, children: node.children.map((c) => previewNode(c, args)) };
  if (node.props !== undefined) out.props = substitute(node.props, args) as OutlineNode['props'];
  if (node.title !== undefined) out.title = displayValue(substitute(node.title, args));
  return out;
}

/** Whether a node of a widget body or an item list is a primitive rather than a composite. */
export function isPrimitive(node: OutlineNode): boolean {
  return (node.layer === 'node' || node.layer === 'item') && PRIMITIVE_KINDS.includes(node.kind);
}

/**
 * Every place the document instantiates the widget `name`, as the server lists them on the
 * widget's outline node (`props.uses`, each `{path, trail?}`, in document order): the same walk
 * behind the widget checks, the in-use refusal of a removal and `/api/docs.md`, page headers and
 * page kinds included. The outline is not walked here: a primitive has its primitive's name as
 * kind, so a walk by kind cannot tell a primitive `badge` from an instance of a widget `badge`.
 */
export function useSites(root: OutlineNode, name: string): UseSite[] {
  const widget = childrenOf(root, 'component').find((c) => c.name === name);
  const uses = widget ? propsOf(widget).uses : undefined;
  if (!Array.isArray(uses)) return [];
  const out: UseSite[] = [];
  for (const raw of uses) {
    const site = record(raw);
    if (!site || typeof site.path !== 'string') continue;
    out.push(typeof site.trail === 'string' ? { path: site.path, trail: site.trail } : { path: site.path });
  }
  return out;
}

/**
 * The widgets matching a search: every word of the query appears, case-insensitively, in the
 * name, the summary or a param name. An empty query keeps every widget.
 */
export function filterComponents(list: ComponentInfo[], query: string): ComponentInfo[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return list;
  return list.filter((c) => {
    const fields = [c.name, c.summary, ...c.params.map((p) => p.name)].map((f) => f.toLowerCase());
    return words.every((w) => fields.some((f) => f.includes(w)));
  });
}
