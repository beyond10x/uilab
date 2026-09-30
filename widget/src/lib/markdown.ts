// A small Markdown renderer for the documents the server generates: headings, paragraphs, bullet
// and numbered lists (nested by indentation), pipe tables, code fences, horizontal rules, inline
// code and bold. Everything else is text. All input is escaped; the output is safe for v-html.

/** `&`, `<`, `>`, `"` and `'` as entities. */
export function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
}

/** Inline markup: `code` spans first (their content is literal), then **bold** in the rest. */
export function renderInline(text: string): string {
  let out = '';
  let rest = text;
  for (;;) {
    const m = /(`+)([\s\S]*?[^`])\1(?!`)/.exec(rest);
    if (!m) break;
    out += bold(escapeHtml(rest.slice(0, m.index)));
    out += `<code>${escapeHtml(m[2].trim() || m[2])}</code>`;
    rest = rest.slice(m.index + m[0].length);
  }
  return out + bold(escapeHtml(rest));
}

function bold(escaped: string): string {
  return escaped.replace(/\*\*(?=\S)([\s\S]*?\S)\*\*/g, '<strong>$1</strong>');
}

const FENCE = /^\s*(```+|~~~+)\s*([\w.+-]*)\s*$/;
const HEADING = /^(#{1,6})\s+(.*?)\s*#*\s*$/;
const LIST_ITEM = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;
const RULE = /^\s*([-*_])(\s*\1){2,}\s*$/;
const TABLE_SEPARATOR = /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/;

function isTableRow(line: string): boolean {
  return line.trim().startsWith('|');
}

/** Cells of a pipe-table row; `\|` is a literal pipe inside a cell. */
export function tableCells(line: string): string[] {
  let t = line.trim();
  if (t.startsWith('|')) t = t.slice(1);
  if (t.endsWith('|') && !t.endsWith('\\|')) t = t.slice(0, -1);
  const cells: string[] = [];
  let cell = '';
  for (let i = 0; i < t.length; i++) {
    if (t[i] === '\\' && t[i + 1] === '|') {
      cell += '|';
      i++;
    } else if (t[i] === '|') {
      cells.push(cell.trim());
      cell = '';
    } else cell += t[i];
  }
  cells.push(cell.trim());
  return cells;
}

function alignments(separator: string): (string | null)[] {
  return tableCells(separator).map((c) => {
    const left = c.startsWith(':');
    const right = c.endsWith(':');
    return left && right ? 'center' : right ? 'right' : left ? 'left' : null;
  });
}

function renderTable(header: string, separator: string, rows: string[]): string {
  const align = alignments(separator);
  const cell = (tag: string, text: string, i: number) =>
    `<${tag}${align[i] ? ` style="text-align:${align[i]}"` : ''}>${renderInline(text)}</${tag}>`;
  const head = tableCells(header);
  let html = '<table><thead><tr>' + head.map((c, i) => cell('th', c, i)).join('') + '</tr></thead>';
  if (rows.length) {
    html += '<tbody>';
    for (const r of rows) {
      const cells = tableCells(r);
      html += '<tr>' + head.map((_, i) => cell('td', cells[i] ?? '', i)).join('') + '</tr>';
    }
    html += '</tbody>';
  }
  return html + '</table>';
}

interface ListLine {
  indent: number;
  ordered: boolean;
  text: string;
}

/** Nested lists from items with their indentation; continuation lines are already folded in. */
function renderList(items: ListLine[]): string {
  let html = '';
  const stack: { indent: number; tag: string }[] = [];
  for (const it of items) {
    const tag = it.ordered ? 'ol' : 'ul';
    while (stack.length && it.indent < stack.at(-1)!.indent) html += `</li></${stack.pop()!.tag}>`;
    const top = stack.at(-1);
    if (!top || it.indent > top.indent) {
      html += `<${tag}>`;
      stack.push({ indent: it.indent, tag });
    } else {
      html += '</li>';
      if (top.tag !== tag) {
        // Same depth, other list kind: close one list and open the other.
        html += `</${stack.pop()!.tag}><${tag}>`;
        stack.push({ indent: it.indent, tag });
      }
    }
    html += `<li>${renderInline(it.text)}`;
  }
  while (stack.length) html += `</li></${stack.pop()!.tag}>`;
  return html;
}

/** Markdown to HTML. */
export function renderMarkdown(source: string): string {
  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const out: string[] = [];
  let i = 0;
  const blank = (l: string | undefined) => l === undefined || l.trim() === '';
  const startsBlock = (k: number) => {
    const l = lines[k];
    return (
      FENCE.test(l) ||
      HEADING.test(l) ||
      LIST_ITEM.test(l) ||
      RULE.test(l) ||
      (isTableRow(l) && k + 1 < lines.length && TABLE_SEPARATOR.test(lines[k + 1]))
    );
  };

  while (i < lines.length) {
    const line = lines[i];
    if (blank(line)) {
      i++;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const marker = fence[1];
      const body: string[] = [];
      i++;
      while (i < lines.length && !(lines[i].trim().startsWith(marker[0].repeat(marker.length)) && lines[i].trim().replace(/[`~]/g, '') === '')) {
        body.push(lines[i]);
        i++;
      }
      i++; // the closing fence, if any
      const lang = fence[2] ? ` class="language-${escapeHtml(fence[2])}"` : '';
      out.push(`<pre><code${lang}>${escapeHtml(body.join('\n'))}</code></pre>`);
      continue;
    }
    const heading = HEADING.exec(line);
    if (heading) {
      const level = heading[1].length;
      out.push(`<h${level}>${renderInline(heading[2])}</h${level}>`);
      i++;
      continue;
    }
    if (RULE.test(line)) {
      out.push('<hr>');
      i++;
      continue;
    }
    if (isTableRow(line) && i + 1 < lines.length && TABLE_SEPARATOR.test(lines[i + 1])) {
      const header = line;
      const separator = lines[i + 1];
      i += 2;
      const rows: string[] = [];
      while (i < lines.length && isTableRow(lines[i])) rows.push(lines[i++]);
      out.push(renderTable(header, separator, rows));
      continue;
    }
    if (LIST_ITEM.test(line)) {
      const items: ListLine[] = [];
      while (i < lines.length) {
        const m = LIST_ITEM.exec(lines[i]);
        if (m) {
          items.push({ indent: m[1].replace(/\t/g, '    ').length, ordered: /\d/.test(m[2]), text: m[3] });
          i++;
        } else if (!blank(lines[i]) && /^\s+/.test(lines[i]) && !startsBlock(i)) {
          // An indented continuation of the item above.
          items[items.length - 1].text += ' ' + lines[i].trim();
          i++;
        } else if (blank(lines[i]) && i + 1 < lines.length && LIST_ITEM.test(lines[i + 1])) {
          i++; // a loose list: blank lines between items
        } else break;
      }
      out.push(renderList(items));
      continue;
    }
    const para: string[] = [];
    while (i < lines.length && !blank(lines[i]) && (para.length === 0 || !startsBlock(i))) para.push(lines[i++].trim());
    out.push(`<p>${renderInline(para.join(' '))}</p>`);
  }
  return out.join('\n');
}
