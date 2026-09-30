import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { childrenOf, propsOf } from './outline.ts';

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
 * A value to preview a param with. A declared `default` is used as written; otherwise the value is
 * shaped by the type and named after the param: text types give the param name, number types 3,
 * boolean types `true`, a list one sample element, an enum its first member, and a record or
 * entity (a named type such as `Loan`, or any other constructor) an object `{name: <param>}`.
 */
export function sampleValue(param: { name: string; type: unknown; hasDefault?: boolean; default?: unknown }): unknown {
  if (param.hasDefault) return param.default;
  return shaped(param.name, param.type);
}

function shaped(name: string, type: unknown): unknown {
  if (typeof type === 'string') {
    const t = type.toLowerCase();
    if (TEXT_TYPES.includes(t)) return name;
    if (NUMBER_TYPES.includes(t)) return 3;
    if (BOOLEAN_TYPES.includes(t)) return true;
    return /^[A-Z]|\./.test(type) ? { name } : name;
  }
  const map = record(type);
  const entry = map && Object.entries(map).length === 1 ? Object.entries(map)[0] : null;
  if (entry) {
    const [ctor, inner] = entry;
    if (LIST_CTORS.includes(ctor)) return [shaped(name, inner)];
    if (CHOICE_CTORS.includes(ctor) && Array.isArray(inner)) return inner[0] ?? name;
    if (WRAP_CTORS.includes(ctor)) return shaped(name, inner);
  }
  return type === undefined || type === null ? name : { name };
}

/** Sample args for a widget: one per param, by [`sampleValue`]. */
export function sampleArgs(params: Param[]): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const p of params) out[p.name] = sampleValue(p);
  return out;
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

/** A value as one line of preview text: an object with a `name` shows its name. */
export function displayValue(value: unknown): string {
  if (value === null || value === undefined) return '';
  if (typeof value !== 'object') return String(value);
  const map = record(value);
  if (map && typeof map.name === 'string') return map.name;
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
 * Every place the outline instantiates the widget `name`, in document order: a node whose kind is
 * the widget (an overlay's kind reads `<presentation> <widget>`), then any object of the node's
 * untyped props whose `component` is the widget, with the trail of keys to it (list entries by
 * their `name`, else their index). `args` are data and are not searched. The widget's own
 * declaration is not a use.
 */
export function useSites(root: OutlineNode, name: string): UseSite[] {
  const out: UseSite[] = [];
  const visit = (node: OutlineNode): void => {
    if (node.layer !== 'root' && node.layer !== 'component') {
      const kind = node.layer === 'overlay' ? node.kind.split(' ').at(-1)! : node.kind;
      const typed = node.layer === 'overlay' ? kind === name.toLowerCase() : kind === name;
      if (typed) out.push({ path: node.path });
      for (const [key, value] of Object.entries(propsOf(node))) {
        if (key !== 'args') walk(value, key, node.path);
      }
    }
    node.children.forEach(visit);
  };
  const walk = (value: unknown, trail: string, path: string): void => {
    if (Array.isArray(value)) {
      value.forEach((item, i) => {
        const itemName = record(item)?.name;
        walk(item, `${trail}/${typeof itemName === 'string' ? itemName : i}`, path);
      });
      return;
    }
    const map = record(value);
    if (!map) return;
    if (map.component === name) out.push({ path, trail });
    for (const [key, v] of Object.entries(map)) {
      if (key !== 'args') walk(v, `${trail}/${key}`, path);
    }
  };
  visit(root);
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
