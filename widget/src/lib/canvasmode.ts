import { ref, type Ref } from 'vue';
import type { UilabWireOutlineNode as OutlineNode } from '../generated/types.ts';
import { isDraftView } from './outline.ts';

/**
 * How the canvas draws the document. `structure` labels every composite `name · kind · view` and
 * shows shell regions as chips; `preview` hides the labels and draws the shell as app chrome.
 */
export type CanvasMode = 'structure' | 'preview';

export const CANVAS_MODE_KEY = 'uilab.canvas.mode';

/** The part of `localStorage` the mode reads and writes. */
export interface ModeStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/** The help line for the mode key; HelpModal lists it. */
export const MODE_HELP: [string[], string] = [['p'], 'switch the canvas between structure (labels) and preview (the app)'];

/** Whether a key press toggles the mode: `p` with no Ctrl, Meta or Alt. */
export function togglesMode(ev: { key: string; ctrlKey?: boolean; metaKey?: boolean; altKey?: boolean }): boolean {
  return !ev.ctrlKey && !ev.metaKey && !ev.altKey && ev.key.toLowerCase() === 'p';
}

/** Whether a key press toggles the mode on the given tab: the canvas is only on the UI tab. */
export function togglesModeOn(ev: Parameters<typeof togglesMode>[0], view: string): boolean {
  return view === 'ui' && togglesMode(ev);
}

/** The mode button: one fixed label, the mode carried by `aria-pressed`. */
export function modeButton(mode: CanvasMode): { label: string; pressed: boolean } {
  return { label: 'Preview', pressed: mode === 'preview' };
}

/** An action the canvas may offer on a node: `remove` is removing it from the document. */
export type CanvasAction = 'remove';

/**
 * The actions that apply to a node in `mode`. `remove` applies to a node the author wrote; a node
 * a page kind or a widget contributes (`inherited`) is not in the document to remove, and the root
 * and the menu are not removable. Preview applies none.
 */
export function nodeActions(node: OutlineNode, mode: CanvasMode): CanvasAction[] {
  if (mode !== 'structure' || node.inherited === true) return [];
  return node.layer === 'root' || node.layer === 'nav' ? [] : ['remove'];
}

/**
 * The line a composite shows while it has no rows, or null when it has some. Structure names the
 * view it reads; preview never names a view.
 */
export function emptyLine(c: { view: string | undefined; loaded: boolean; count: number; connOpen: boolean; preview: boolean }): string | null {
  if (c.loaded && c.count > 0) return null;
  if (c.preview) return !c.view || c.loaded || !c.connOpen ? 'no data yet' : 'loading…';
  if (!c.view) return 'no data yet (no view)';
  if (!c.loaded && c.connOpen) return `loading ${c.view}…`;
  return `no data yet (${c.view})`;
}

/**
 * The mode as a Vue ref, read from storage at creation and written back on every toggle. Storage
 * that is absent or throws (disabled, private mode, full) leaves the mode working on screen.
 */
export function createCanvasMode(storage: () => ModeStorage | null): { mode: Ref<CanvasMode>; toggle(): CanvasMode } {
  let stored: string | null = null;
  try {
    stored = storage()?.getItem(CANVAS_MODE_KEY) ?? null;
  } catch {
    stored = null;
  }
  const mode = ref<CanvasMode>(stored === 'preview' ? 'preview' : 'structure');
  function toggle(): CanvasMode {
    mode.value = mode.value === 'preview' ? 'structure' : 'preview';
    try {
      storage()?.setItem(CANVAS_MODE_KEY, mode.value);
    } catch {
      // Not remembered across reloads; the canvas still switches.
    }
    return mode.value;
  }
  return { mode, toggle };
}

/** The page's `localStorage`; none outside a browser. Reading it may throw, which the caller catches. */
function browserStorage(): ModeStorage | null {
  return (globalThis as { window?: { localStorage?: ModeStorage } }).window?.localStorage ?? null;
}

/** The canvas mode of this browser tab. */
export const canvasMode = createCanvasMode(browserStorage);

/**
 * The signed-in staff member's name for the account menu: the first row's `name`, else
 * `display_name`, `full_name` or `email`. Null when there is no such row or field.
 */
export function accountName(rows: unknown[] | undefined): string | null {
  const first = rows?.[0];
  if (!first || typeof first !== 'object' || Array.isArray(first)) return null;
  const row = first as Record<string, unknown>;
  for (const field of ['name', 'display_name', 'full_name', 'email']) {
    const v = row[field];
    if (typeof v === 'string' && v.trim()) return v.trim();
  }
  return null;
}

/**
 * What the account menu shows in preview: the name from the rows its view reads, else `fallback`.
 * A name read from a draft view is made up, so it is marked as sample data like every draft read.
 */
export function accountChrome(view: string | undefined, rows: unknown[] | undefined, fallback: string): { name: string; initial: string; sample: boolean } {
  const read = accountName(rows);
  const name = read ?? fallback;
  return { name, initial: name.charAt(0).toUpperCase(), sample: read !== null && isDraftView(view) };
}
