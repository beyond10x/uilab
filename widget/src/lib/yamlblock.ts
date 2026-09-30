// Finds the lines of a `ui-spec/1` YAML document that hold the node at a path, by walking the
// indentation. Best effort: it reads block-style maps and the navigation's list of sections, which
// is how the server writes documents; a node written in flow style is found only as its key line.

import { segments } from './outline.ts';

/** Lines `[start, end)`, 0-based. */
export interface LineRange {
  start: number;
  end: number;
}

/** The keys a path segment spells in the document, per layer. */
const KEYS: Record<string, string> = {
  shell: 'shells',
  region: 'regions',
  page: 'pages',
  section: 'sections',
  overlay: 'overlays',
  widget: 'widgets',
  item: 'item',
};

function indentOf(line: string): number {
  return line.length - line.trimStart().length;
}

/** Blank lines and comments do not end a block. */
function isContent(line: string): boolean {
  const t = line.trim();
  return t !== '' && !t.startsWith('#');
}

/** End of the block whose header is `start`: the next content line indented at most `indent`. */
function blockEnd(lines: string[], start: number, end: number, indent: number): number {
  let k = start + 1;
  while (k < end && (!isContent(lines[k]) || indentOf(lines[k]) > indent)) k++;
  // Trailing blank lines and comments belong to what follows.
  while (k - 1 > start && !isContent(lines[k - 1])) k--;
  return k;
}

function escapeRe(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** The key `key` directly inside `range` (a header line and its body, or the whole document). */
function childKey(lines: string[], range: LineRange, key: string, top: boolean): LineRange | null {
  const from = top ? range.start : range.start + 1;
  let childIndent = -1;
  for (let k = from; k < range.end; k++) {
    if (!isContent(lines[k])) continue;
    childIndent = indentOf(lines[k]);
    break;
  }
  if (childIndent < 0) return null;
  const re = new RegExp(`^ {${childIndent}}(?:${escapeRe(key)}|"${escapeRe(key)}"|'${escapeRe(key)}')\\s*:(?:\\s|$)`);
  for (let k = from; k < range.end; k++) {
    if (!isContent(lines[k]) || indentOf(lines[k]) !== childIndent) continue;
    if (re.test(lines[k])) return { start: k, end: blockEnd(lines, k, range.end, childIndent) };
  }
  return null;
}

/** The entry of the list under the header `range` whose `name` is `name`. */
function namedEntry(lines: string[], range: LineRange, name: string): LineRange | null {
  const nameRe = new RegExp(`(?:^|[{,\\s])name\\s*:\\s*(?:${escapeRe(name)}|"${escapeRe(name)}"|'${escapeRe(name)}')\\s*(?:[,}]|$)`);
  let dashIndent = -1;
  for (let k = range.start + 1; k < range.end; k++) {
    const m = /^(\s*)-(\s|$)/.exec(lines[k]);
    if (!m) continue;
    if (dashIndent < 0) dashIndent = m[1].length;
    if (m[1].length !== dashIndent) continue;
    const end = blockEnd(lines, k, range.end, dashIndent);
    // `- name: x` on the dash line, or `name: x` among the entry's own keys.
    for (let j = k; j < end; j++) {
      const text = j === k ? lines[j].slice(dashIndent + 1) : lines[j];
      if (j > k && indentOf(lines[j]) !== dashIndent + 2) continue;
      if (nameRe.test(text)) return { start: k, end };
    }
  }
  return null;
}

/** The lines of the node at `path`, or `null` for the root or a node the walk cannot find. */
export function yamlBlock(text: string, path: string): LineRange | null {
  const lines = text.replace(/\r\n?/g, '\n').split('\n');
  const segs = segments(path);
  if (!segs.length) return null;
  let range: LineRange = { start: 0, end: lines.length };
  let top = true;
  const step = (key: string): boolean => {
    const next = childKey(lines, range, key, top);
    if (!next) return false;
    range = next;
    top = false;
    return true;
  };
  for (const seg of segs) {
    if (seg === 'nav') {
      if (!step('navigation')) return null;
      continue;
    }
    const colon = seg.indexOf(':');
    if (colon < 0) return null;
    const layer = seg.slice(0, colon);
    const name = seg.slice(colon + 1);
    if (layer === 'nav_section') {
      if (!step('sections')) return null;
      const entry = namedEntry(lines, range, name);
      if (!entry) return null;
      range = entry;
      continue;
    }
    const key = KEYS[layer];
    if (!key || !step(key) || !step(name)) return null;
  }
  return range;
}
