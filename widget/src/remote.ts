import { shallowReactive, watch } from 'vue';
import { state } from './store.ts';

/** Text the server generates from the document: `/api/document.yaml`, `/api/docs.md`, `/api/help.md`. */
export interface RemoteText {
  text: string | null;
  error: string | null;
  loading: boolean;
  reload(): void;
}

export const API = {
  yaml: '/api/document.yaml',
  yamlDownload: '/api/document.yaml?download',
  docs: '/api/docs.md',
  help: '/api/help.md',
} as const;

/** `url`, fetched on each `reload`; a slower earlier answer never overwrites a later one. */
export function onDemand(url: string): RemoteText {
  let seq = 0;
  const r: RemoteText = shallowReactive({
    text: null,
    error: null,
    loading: false,
    reload: () => void load(),
  });
  async function load(): Promise<void> {
    const mine = ++seq;
    r.loading = true;
    try {
      const res = await fetch(url, { cache: 'no-store' });
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`.trim());
      const text = await res.text();
      if (mine !== seq) return;
      r.text = text;
      r.error = null;
    } catch (err) {
      if (mine !== seq) return;
      r.error = `could not load ${url}: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      if (mine === seq) r.loading = false;
    }
  }
  return r;
}

/**
 * `url` fetched while `active()` holds, and again whenever the document's revision moves (a
 * `document` or an applied `changed`). An inactive view is fetched when it becomes active.
 */
export function followDocument(url: string, active: () => boolean): RemoteText {
  const r = onDemand(url);
  let fetchedAt: number | null | undefined;
  watch(
    [() => state.revision, active],
    ([revision, on]) => {
      if (!on || revision === fetchedAt) return;
      fetchedAt = revision;
      r.reload();
    },
    { immediate: true },
  );
  return r;
}

/** The document as YAML, while the YAML view is shown. */
export const yamlText = followDocument(API.yaml, () => state.view === 'yaml');
/** The generated documentation, while the Docs view is shown. */
export const docsText = followDocument(API.docs, () => state.view === 'docs');
/** What uilab and its agent can do; fetched when help opens. */
export const helpText = onDemand(API.help);
