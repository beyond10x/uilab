import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';

/** A path with surrounding slashes and whitespace removed; the root is `/`. */
export function normalize(path: string): string {
  const trimmed = path.trim().replace(/^\/+|\/+$/g, '');
  return trimmed === '' ? '/' : trimmed;
}

/** The `/`-separated segments of a path; none for the root. */
export function segments(path: string): string[] {
  const p = normalize(path);
  return p === '/' ? [] : p.split('/');
}

/** Whether `path` is `ancestor` or sits below it. */
export function isWithin(path: string, ancestor: string): boolean {
  const p = normalize(path);
  const a = normalize(ancestor);
  return a === '/' || p === a || p.startsWith(a + '/');
}

/** The parent path, or `null` at the root. */
export function parentPath(path: string): string | null {
  const segs = segments(path);
  if (segs.length === 0) return null;
  return segs.length === 1 ? '/' : segs.slice(0, -1).join('/');
}

/** The node at `path`, or `null` when the outline has none. */
export function findNode(root: OutlineNode, path: string): OutlineNode | null {
  return lineage(root, path)?.at(-1) ?? null;
}

/** The nodes from the root down to the one at `path`, or `null` when there is no such node. */
export function lineage(root: OutlineNode, path: string): OutlineNode[] | null {
  const target = normalize(path);
  const chain: OutlineNode[] = [root];
  let node = root;
  while (normalize(node.path) !== target) {
    const next = node.children.find((c) => isWithin(target, c.path));
    if (!next) return null;
    chain.push(next);
    node = next;
  }
  return chain;
}

/** The path of the deepest node of the outline that `path` is or sits under; the root at worst. */
export function nearestExisting(root: OutlineNode, path: string): string {
  for (let p: string | null = normalize(path); p !== null; p = parentPath(p)) {
    const node = findNode(root, p);
    if (node) return node.path;
  }
  return root.path;
}

/** The page path (`page:<name>`) a path sits on, or `null` when it is not under a page. */
export function pageOf(path: string): string | null {
  const first = segments(path)[0];
  return first?.startsWith('page:') ? first : null;
}

/** The overlay path a path sits in (a page overlay or a shell overlay), or `null`. */
export function overlayOf(path: string): string | null {
  const segs = segments(path);
  const i = segs.findIndex((s) => s.startsWith('overlay:'));
  return i < 0 ? null : segs.slice(0, i + 1).join('/');
}

/** Direct children of `node` of one layer. */
export function childrenOf(node: OutlineNode | null | undefined, layer: string): OutlineNode[] {
  return node ? node.children.filter((c) => c.layer === layer) : [];
}

/** The pages of a document, in document order. */
export function pagesOf(root: OutlineNode): OutlineNode[] {
  return childrenOf(root, 'page');
}

/** One menu entry: a page, or a page opened per row of a view (a dynamic section). */
export interface NavEntry {
  page: OutlineNode;
  fromView?: string;
}

/** A menu group; `section` is `null` for the fallback group of every page. */
export interface NavGroup {
  section: OutlineNode | null;
  entries: NavEntry[];
}

/**
 * The menu as the document places it. A section lists its pages in `props.pages`, or names one
 * page opened per row of `props.from_view`; a page in no section is not in the menu. When no
 * section says which pages it holds, the sections are listed as headings and every page follows.
 */
export function navLayout(root: OutlineNode): NavGroup[] {
  const nav = childrenOf(root, 'nav')[0];
  const sections = childrenOf(nav, 'nav_section');
  const pages = pagesOf(root);
  const byName = new Map(pages.map((p) => [p.name, p]));
  const placed = sections.some((s) => {
    const p = propsOf(s);
    return Array.isArray(p.pages) || typeof p.page === 'string';
  });
  if (!placed) {
    const groups: NavGroup[] = sections.map((s) => ({ section: s, entries: [] }));
    if (pages.length) groups.push({ section: null, entries: pages.map((page) => ({ page })) });
    return groups;
  }
  return sections.map((s) => {
    const p = propsOf(s);
    const entries: NavEntry[] = [];
    if (Array.isArray(p.pages)) {
      for (const name of p.pages) {
        const page = byName.get(String(name));
        if (page) entries.push({ page });
      }
    } else if (typeof p.page === 'string') {
      const page = byName.get(p.page);
      if (page) entries.push({ page, fromView: typeof p.from_view === 'string' ? p.from_view : undefined });
    }
    return { section: s, entries };
  });
}

/** The page the app opens at: `props.home` of the nav, else the first page. */
export function homePage(root: OutlineNode): OutlineNode | null {
  const home = propsOf(childrenOf(root, 'nav')[0] ?? root).home;
  const pages = pagesOf(root);
  return (typeof home === 'string' ? pages.find((p) => p.name === home) : undefined) ?? pages[0] ?? null;
}

/** The shell a page renders in: its `props.shell`, else the first shell. */
export function shellOf(root: OutlineNode, page: OutlineNode | null): OutlineNode | null {
  const shells = childrenOf(root, 'shell');
  const named = page ? propsOf(page).shell : undefined;
  return (typeof named === 'string' ? shells.find((s) => s.name === named) : undefined) ?? shells[0] ?? null;
}

/**
 * The mark of a node the author did not write: a section or overlay its page kind contributes, or
 * a node of a widget instance's body. ESS renders it, so the outline and the canvas show it, but
 * it is not in the document. `null` for a node the author wrote.
 */
export function inheritedMark(node: OutlineNode): string | null {
  return node.inherited === true ? 'inherited' : null;
}

/** The classes a node is drawn with for what it is: `inherited` when the author did not write it. */
export function nodeClasses(node: OutlineNode): Record<string, boolean> {
  return { inherited: node.inherited === true };
}

/** A node's display label: its title, else its name, else its layer. */
export function labelOf(node: OutlineNode): string {
  return node.title || node.name || (node.layer === 'root' ? 'document' : node.layer);
}

/** A props object as a record, or an empty one when props are absent or not an object. */
export function propsOf(node: OutlineNode): Record<string, unknown> {
  const p = node.props;
  return p && typeof p === 'object' && !Array.isArray(p) ? (p as Record<string, unknown>) : {};
}

/** A column of a collection or references: `columns: [{field, as?, label?}]`, a bare string
 *  meaning `{field}`. `label` only when the author wrote one. */
export interface Column {
  field: string;
  as?: string;
  label?: string;
}

export function columnsOf(node: OutlineNode): Column[] {
  const raw = propsOf(node).columns;
  if (!Array.isArray(raw)) return [];
  const out: Column[] = [];
  for (const c of raw) {
    if (typeof c === 'string') out.push({ field: c });
    else if (c && typeof c === 'object' && typeof (c as Record<string, unknown>).field === 'string') {
      const rec = c as Record<string, unknown>;
      const column: Column = { field: rec.field as string, as: typeof rec.as === 'string' ? rec.as : undefined };
      if (typeof rec.label === 'string') column.label = rec.label;
      out.push(column);
    }
  }
  return out;
}

/** A field of a form or record: `fields: [name | {field|name, label?}]`. */
export interface Field {
  field: string;
  label: string;
}

export function fieldsOf(node: OutlineNode, key = 'fields'): Field[] {
  return fieldList(propsOf(node)[key]);
}

/** The text of an ESS `Action`: its `label`, else its name, else what it opens or runs. */
export function actionLabel(action: unknown): string {
  const r = action && typeof action === 'object' && !Array.isArray(action) ? (action as Record<string, unknown>) : {};
  for (const k of ['label', 'name', 'opens', 'does']) if (typeof r[k] === 'string') return r[k] as string;
  return 'action';
}

/** The lists of ESS `Action`s a composite carries other than `row_actions` (drawn per row): on the
 *  whole collection, record, form or filter bar, on the selection, on each placed board widget,
 *  on a graph's nodes and edges, and a confirm's alternatives. */
const ACTION_LISTS = ['actions', 'bulk_actions', 'item_actions', 'node_actions', 'edge_actions', 'alternatives'];

/** The text of every action of [`ACTION_LISTS`] the node writes, in that order. */
export function actionsOf(node: OutlineNode): string[] {
  const p = propsOf(node);
  return ACTION_LISTS.flatMap((k) => (Array.isArray(p[k]) ? (p[k] as unknown[]).map(actionLabel) : []));
}

/** A tab of a record or form, or a group of a form: its text (`label`, else its name) and fields. */
export interface FieldGroup {
  name: string;
  label: string;
  fields: Field[];
}

/** The ESS `Tab`s (`key: 'tabs'`) or `FormGroup`s (`key: 'groups'`) the node writes. */
export function groupsOf(node: OutlineNode, key: 'tabs' | 'groups'): FieldGroup[] {
  const raw = propsOf(node)[key];
  if (!Array.isArray(raw)) return [];
  return raw.flatMap((g) => {
    if (!g || typeof g !== 'object' || Array.isArray(g)) return [];
    const r = g as Record<string, unknown>;
    const name = typeof r.name === 'string' ? r.name : '';
    return [{ name, label: typeof r.label === 'string' ? r.label : name, fields: fieldList(r.fields) }];
  });
}

/** The labels of a choice's fixed `options`: `{value, label}` records or bare strings. A named enum
 *  type (a string) has no labels the canvas can read. */
export function optionsOf(node: OutlineNode): string[] {
  const raw = propsOf(node).options;
  if (!Array.isArray(raw)) return [];
  return raw.flatMap((o) => {
    if (typeof o === 'string') return [o];
    const r = o && typeof o === 'object' ? (o as Record<string, unknown>) : {};
    if (typeof r.label === 'string') return [r.label];
    return r.value === undefined ? [] : [String(r.value)];
  });
}

/** A record field of `p` read as text, or null when absent or not text. */
export function textAt(p: Record<string, unknown>, ...keys: string[]): string | null {
  let at: unknown = p;
  for (const k of keys) {
    if (!at || typeof at !== 'object' || Array.isArray(at)) return null;
    at = (at as Record<string, unknown>)[k];
  }
  return typeof at === 'string' ? at : null;
}

/** ESS `Field`s written as a list: `name | {field|name, label?}`; anything else is none. */
function fieldList(raw: unknown): Field[] {
  if (!Array.isArray(raw)) return [];
  const out: Field[] = [];
  for (const f of raw) {
    if (typeof f === 'string') out.push({ field: f, label: f });
    else if (f && typeof f === 'object') {
      const rec = f as Record<string, unknown>;
      const field = typeof rec.field === 'string' ? rec.field : typeof rec.name === 'string' ? rec.name : null;
      if (field) out.push({ field, label: typeof rec.label === 'string' ? rec.label : field });
    }
  }
  return out;
}

/** Whether a view has no model binding yet and so no rows. */
export function isDraftView(view: string | undefined): boolean {
  return !!view && view.startsWith('draft.');
}
