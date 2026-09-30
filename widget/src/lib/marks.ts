import type { EssJsonValue, UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { childrenOf, normalize, pagesOf, propsOf } from './outline.ts';

/** What a proposal does to one node: brings it in, alters it, or takes it out. */
export type Mark = 'added' | 'changed' | 'removed';

/** Whether two JSON values are equal, object keys in any order. */
function sameJson(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (Array.isArray(a) || Array.isArray(b)) {
    return Array.isArray(a) && Array.isArray(b) && a.length === b.length && a.every((x, i) => sameJson(x, b[i]));
  }
  if (!a || !b || typeof a !== 'object' || typeof b !== 'object') return false;
  const ra = a as Record<string, unknown>;
  const rb = b as Record<string, unknown>;
  const ka = Object.keys(ra).filter((k) => ra[k] !== undefined);
  const kb = Object.keys(rb).filter((k) => rb[k] !== undefined);
  return ka.length === kb.length && ka.every((k) => k in rb && sameJson(ra[k], rb[k]));
}

/**
 * Props the server derives from other nodes rather than reads from the node itself, by layer: a
 * widget's `uses` (its instances) and a menu section's `pages` (a page Remove drops the page from
 * them). A change to one of these is a change elsewhere, so it does not make the node changed.
 */
export const DERIVED_PROPS: Readonly<Record<string, readonly string[]>> = {
  component: ['uses'],
  nav_section: ['pages'],
};

/** The node's own props: its props without the ones derived for its layer. */
function ownProps(node: OutlineNode): unknown {
  const derived = DERIVED_PROPS[node.layer];
  const p = node.props ?? null;
  if (!derived || !p || typeof p !== 'object' || Array.isArray(p)) return p;
  return Object.fromEntries(Object.entries(p).filter(([k]) => !derived.includes(k)));
}

/** Whether the node itself differs, children aside: its own props, title, kind or view. */
function nodeDiffers(a: OutlineNode, b: OutlineNode): boolean {
  return (
    a.kind !== b.kind ||
    (a.title ?? null) !== (b.title ?? null) ||
    (a.view ?? null) !== (b.view ?? null) ||
    !sameJson(ownProps(a), ownProps(b))
  );
}

function markSubtree(node: OutlineNode, mark: Mark, out: Map<string, Mark>): void {
  out.set(normalize(node.path), mark);
  for (const c of node.children) markSubtree(c, mark, out);
}

function byPath(nodes: OutlineNode[]): Map<string, OutlineNode> {
  return new Map(nodes.map((c) => [normalize(c.path), c]));
}

function compare(doc: OutlineNode, proposal: OutlineNode, out: Map<string, Mark>): void {
  if (nodeDiffers(doc, proposal)) out.set(normalize(proposal.path), 'changed');
  const docKids = byPath(doc.children);
  const proposalKids = byPath(proposal.children);
  for (const c of proposal.children) {
    const before = docKids.get(normalize(c.path));
    if (before) compare(before, c, out);
    else markSubtree(c, 'added', out);
  }
  for (const c of doc.children) if (!proposalKids.has(normalize(c.path))) markSubtree(c, 'removed', out);
}

/**
 * Normalized path → mark, from comparing the proposal's outline with the document's: a path only
 * in the proposal is added, only in the document removed, in both with other props, title, kind or
 * view changed. Unmarked paths are untouched.
 */
export function outlineMarks(doc: OutlineNode, proposal: OutlineNode): Map<string, Mark> {
  const out = new Map<string, Mark>();
  compare(doc, proposal, out);
  return out;
}

/**
 * `kept` with each item of `before` that `restore` picks and `kept` lacks put back after the
 * nearest item before it in `before` that the result has, else first. `kept` itself when none is.
 */
function restoreInOrder<T>(before: readonly T[], kept: readonly T[], key: (x: T) => string, restore: (x: T) => boolean): T[] {
  const out = [...kept];
  const has = new Set(kept.map(key));
  before.forEach((x, i) => {
    if (has.has(key(x)) || !restore(x)) return;
    let at = 0;
    for (let j = i - 1; j >= 0; j--) {
      const idx = out.findIndex((k) => key(k) === key(before[j]));
      if (idx >= 0) {
        at = idx + 1;
        break;
      }
    }
    out.splice(at, 0, x);
  });
  return out.length === kept.length ? (kept as T[]) : out;
}

const pathKey = (n: OutlineNode) => normalize(n.path);

function mergeRemoved(doc: OutlineNode, proposal: OutlineNode): OutlineNode {
  const docKids = byPath(doc.children);
  let touched = false;
  const merged = proposal.children.map((c) => {
    const before = docKids.get(normalize(c.path));
    const m = before ? mergeRemoved(before, c) : c;
    if (m !== c) touched = true;
    return m;
  });
  const kids = restoreInOrder(doc.children, merged, pathKey, () => true);
  return touched || kids !== merged ? { ...proposal, children: kids } : proposal;
}

/**
 * The shown outline's menu sections with each page the proposal removes listed again where the
 * document's section listed it, so the menu shows it struck through; pages the proposal adds stay.
 */
function relistRemovedPages(doc: OutlineNode, proposal: OutlineNode, shown: OutlineNode): OutlineNode {
  const remaining = new Set(pagesOf(proposal).map((p) => p.name));
  const removed = new Set(pagesOf(doc).map((p) => p.name).filter((name) => !remaining.has(name)));
  const docNav = childrenOf(doc, 'nav')[0];
  const shownNav = childrenOf(shown, 'nav')[0];
  if (!removed.size || !docNav || !shownNav) return shown;
  const docSections = byPath(childrenOf(docNav, 'nav_section'));
  const sections = shownNav.children.map((s) => {
    const d = s.layer === 'nav_section' ? docSections.get(normalize(s.path)) : undefined;
    const listed = d ? propsOf(d).pages : undefined;
    const props = propsOf(s);
    if (!Array.isArray(listed) || !Array.isArray(props.pages)) return s;
    const pages = restoreInOrder(listed as EssJsonValue[], props.pages as EssJsonValue[], String, (x) => removed.has(String(x)));
    return pages === props.pages ? s : { ...s, props: { ...(props as { [key: string]: EssJsonValue }), pages } };
  });
  if (sections.every((s, i) => s === shownNav.children[i])) return shown;
  const nav = { ...shownNav, children: sections };
  return { ...shown, children: shown.children.map((c) => (c === shownNav ? nav : c)) };
}

/**
 * The proposal's outline with every node it removes put back from the document's, each after the
 * nearest document sibling before it, and every page it removes listed again in its menu section,
 * so a preview can draw what goes away. The proposal itself when it removes nothing.
 */
export function withRemoved(doc: OutlineNode, proposal: OutlineNode): OutlineNode {
  return relistRemovedPages(doc, proposal, mergeRemoved(doc, proposal));
}

/**
 * The document outline a pending proposal is previewed against, after one server message: the
 * document current when the proposal arrived, kept while that proposal stays pending, so a document
 * change that leaves the card open (another client's undo) is not shown as the proposal's doing.
 * `before`/`after` are the pending proposal before and after the message, `base` the outline kept
 * so far, `doc` the current document's. None without a pending proposal.
 */
export function previewBase(
  before: { proposal_id: string } | null,
  after: { proposal_id: string } | null,
  base: OutlineNode | null,
  doc: OutlineNode | null,
): OutlineNode | null {
  if (!after) return null;
  if (base && before && before.proposal_id === after.proposal_id) return base;
  return doc;
}

/** The CSS classes of a mark; each mark sets exactly one. */
export function markClasses(mark: Mark | undefined): Record<string, boolean> {
  return { 'hl-insert': mark === 'added', 'hl-replace': mark === 'changed', removed: mark === 'removed' };
}
