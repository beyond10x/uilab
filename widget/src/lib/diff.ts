/** One line of a line diff: kept (` `), removed (`-`) or added (`+`). */
export interface DiffLine {
  op: ' ' | '-' | '+';
  text: string;
  /** 1-based line number in the old text, for kept and removed lines. */
  a?: number;
  /** 1-based line number in the new text, for kept and added lines. */
  b?: number;
}

/** A run of changed lines with surrounding context, as in a unified diff. */
export interface Hunk {
  header: string;
  lines: DiffLine[];
}

function splitLines(text: string): string[] {
  if (text === '') return [];
  const lines = text.split('\n');
  if (lines.at(-1) === '') lines.pop();
  return lines;
}

/** A minimal line diff of `before` against `after` (longest common subsequence). */
export function diffLines(before: string, after: string): DiffLine[] {
  const a = splitLines(before);
  const b = splitLines(after);
  // Common prefix and suffix are kept outright; the table covers only the middle.
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;
  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) {
    endA--;
    endB--;
  }
  const n = endA - start;
  const m = endB - start;
  const width = m + 1;
  const lcs = new Uint32Array((n + 1) * (m + 1));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      lcs[i * width + j] =
        a[start + i] === b[start + j]
          ? lcs[(i + 1) * width + j + 1] + 1
          : Math.max(lcs[(i + 1) * width + j], lcs[i * width + j + 1]);
    }
  }
  const out: DiffLine[] = [];
  for (let k = 0; k < start; k++) out.push({ op: ' ', text: a[k], a: k + 1, b: k + 1 });
  let i = 0;
  let j = 0;
  while (i < n || j < m) {
    if (i < n && j < m && a[start + i] === b[start + j]) {
      out.push({ op: ' ', text: a[start + i], a: start + i + 1, b: start + j + 1 });
      i++;
      j++;
    } else if (i < n && (j === m || lcs[(i + 1) * width + j] >= lcs[i * width + j + 1])) {
      out.push({ op: '-', text: a[start + i], a: start + i + 1 });
      i++;
    } else {
      out.push({ op: '+', text: b[start + j], b: start + j + 1 });
      j++;
    }
  }
  for (let k = 0; k < a.length - endA; k++) {
    out.push({ op: ' ', text: a[endA + k], a: endA + k + 1, b: endB + k + 1 });
  }
  return out;
}

/** Group a diff into hunks keeping `context` unchanged lines around each change. */
export function hunks(lines: DiffLine[], context = 3): Hunk[] {
  const changed = lines.map((l, idx) => (l.op === ' ' ? -1 : idx)).filter((idx) => idx >= 0);
  if (changed.length === 0) return [];
  const ranges: [number, number][] = [];
  for (const idx of changed) {
    const lo = Math.max(0, idx - context);
    const hi = Math.min(lines.length - 1, idx + context);
    const last = ranges.at(-1);
    if (last && lo <= last[1] + 1) last[1] = Math.max(last[1], hi);
    else ranges.push([lo, hi]);
  }
  return ranges.map(([lo, hi]) => {
    const slice = lines.slice(lo, hi + 1);
    const aLines = slice.filter((l) => l.op !== '+');
    const bLines = slice.filter((l) => l.op !== '-');
    const aStart = aLines[0]?.a ?? precedingA(lines, lo);
    const bStart = bLines[0]?.b ?? precedingB(lines, lo);
    return {
      header: `@@ -${aStart},${aLines.length} +${bStart},${bLines.length} @@`,
      lines: slice,
    };
  });
}

function precedingA(lines: DiffLine[], idx: number): number {
  for (let k = idx - 1; k >= 0; k--) if (lines[k].a !== undefined) return lines[k].a!;
  return 0;
}

function precedingB(lines: DiffLine[], idx: number): number {
  for (let k = idx - 1; k >= 0; k--) if (lines[k].b !== undefined) return lines[k].b!;
  return 0;
}
