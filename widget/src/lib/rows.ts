import { isDraftView } from './outline.ts';

/**
 * Whether the canvas asks the server for `view`'s rows now. `have`: rows for it are shown;
 * `askedAt`: the document revision the last request went out at, `undefined` when none did.
 *
 * Fixture rows do not change with the document, so they are asked for once. A `draft.` view has
 * no fixture: the server makes sample rows for it, shaped by the composites that read it, so they
 * are asked for again once the document is at another revision.
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
