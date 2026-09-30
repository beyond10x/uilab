import type { UilabWireWorkspace as Workspace } from '../generated/types.ts';

/**
 * The workspace an instruction comes from. On the Components tab the operator is building
 * reusable widgets, so the server and the agent are told; every other view is the app canvas.
 */
export function workspaceOf(view: string): Workspace | undefined {
  return view === 'components' ? 'components' : undefined;
}

/**
 * Where an instruction goes, and which page to select first.
 *
 * - Components tab: a selected widget or one of its body nodes; otherwise the root, where
 *   widgets are declared. A page left selected in the canvas does not count.
 * - Anywhere else: the selection, unless nothing below the root is selected while a page is on
 *   screen. Then that page is selected first, so "add a header" lands on the page the operator is
 *   looking at rather than at the root, where only pages fit.
 */
export function targetIn(
  view: string,
  selected: string | undefined,
  pageOnScreen: string | undefined,
): { target: string | undefined; selectPage?: string } {
  if (workspaceOf(view)) {
    return { target: selected?.startsWith('component:') ? selected : '/' };
  }
  if ((!selected || selected === '/') && pageOnScreen) {
    return { target: pageOnScreen, selectPage: pageOnScreen };
  }
  return { target: selected };
}
