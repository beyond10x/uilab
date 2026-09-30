import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { isDraftView, propsOf } from './outline.ts';

/**
 * Whether the canvas asks the server for `view`'s rows now. `have`: rows for it are shown;
 * `askedAt`: the document revision the last request went out at, `undefined` when none did.
 *
 * A `draft.` view has no fixture: the server makes sample rows for it, shaped by the composites
 * that read it, so they are asked for again once the document is at another revision. Every other
 * view is taken to be fixture-backed and asked for once. That is not always so: the server answers
 * a view without the prefix that has no fixture with sample rows too, and those are neither
 * reshaped at a new revision nor tagged as sample data.
 */
export function rowsDue(
  view: string,
  have: boolean,
  askedAt: number | null | undefined,
  revision: number | null,
): boolean {
  if (isDraftView(view)) return askedAt !== revision;
  return !have && askedAt === undefined;
}

/**
 * Every view a node of the outline reads, once each, `draft.` views included: a node's `view` (a
 * composite's, an overlay's, a shell region's) and a dynamic menu section's `props.from_view`.
 */
export function viewsOf(root: OutlineNode | null): Set<string> {
  const out = new Set<string>();
  const add = (view: unknown) => {
    if (typeof view === 'string' && view) out.add(view);
  };
  const walk = (node: OutlineNode) => {
    add(node.view);
    if (node.layer === 'nav_section') add(propsOf(node).from_view);
    node.children.forEach(walk);
  };
  if (root) walk(root);
  return out;
}

/**
 * The views whose last request, asked at the revision `asked` holds, has no answer that arrived
 * at that revision (`answered`: view → the revision the browser was at when its last rows
 * arrived). On a connection that dropped, those answers never come.
 */
export function unanswered(
  asked: ReadonlyMap<string, number | null>,
  answered: ReadonlyMap<string, number | null>,
): string[] {
  return [...asked].filter(([view, at]) => !answered.has(view) || answered.get(view) !== at).map(([view]) => view);
}
