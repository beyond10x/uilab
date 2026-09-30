import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { normalize } from './outline.ts';

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

/** Whether the node itself differs, children aside: its props, title, kind or view. */
function nodeDiffers(a: OutlineNode, b: OutlineNode): boolean {
  return (
    a.kind !== b.kind ||
    (a.title ?? null) !== (b.title ?? null) ||
    (a.view ?? null) !== (b.view ?? null) ||
    !sameJson(a.props ?? null, b.props ?? null)
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
 * The proposal's outline with every node it removes put back from the document's, each after the
 * nearest document sibling before it, so a preview can draw what goes away. The proposal itself
 * when it removes nothing.
 */
export function withRemoved(doc: OutlineNode, proposal: OutlineNode): OutlineNode {
  const docKids = byPath(doc.children);
  const kept = new Set(proposal.children.map((c) => normalize(c.path)));
  let touched = false;
  const kids = proposal.children.map((c) => {
    const before = docKids.get(normalize(c.path));
    const merged = before ? withRemoved(before, c) : c;
    if (merged !== c) touched = true;
    return merged;
  });
  doc.children.forEach((c, i) => {
    if (kept.has(normalize(c.path))) return;
    touched = true;
    let at = 0;
    for (let j = i - 1; j >= 0; j--) {
      const prev = normalize(doc.children[j].path);
      const idx = kids.findIndex((k) => normalize(k.path) === prev);
      if (idx >= 0) {
        at = idx + 1;
        break;
      }
    }
    kids.splice(at, 0, c);
  });
  return touched ? { ...proposal, children: kids } : proposal;
}

/** The CSS classes of a mark; each mark sets exactly one. */
export function markClasses(mark: Mark | undefined): Record<string, boolean> {
  return { 'hl-insert': mark === 'added', 'hl-replace': mark === 'changed', removed: mark === 'removed' };
}
