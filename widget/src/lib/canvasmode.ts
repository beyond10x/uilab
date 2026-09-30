import { ref, type Ref } from 'vue';

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
