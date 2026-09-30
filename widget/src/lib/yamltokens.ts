// Colours for the YAML view: each line of a document cut into tokens whose texts spell the line
// exactly. Line by line, as the server writes documents (block maps and lists, flow maps and lists
// on one line, block scalars); anything it does not recognise stays a string.

export type TokenKind = 'key' | 'string' | 'number' | 'literal' | 'comment' | 'punct' | 'space';

export interface Token {
  text: string;
  kind: TokenKind;
}

/** The lines of a document as the YAML view numbers them: no trailing empty line. */
export function yamlLines(text: string): string[] {
  return text.replace(/\r\n?/g, '\n').replace(/\n$/, '').split('\n');
}

const NUMBER = /^(?:[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?|0x[0-9a-fA-F]+|0o[0-7]+|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$/;
const LITERAL = /^(?:true|True|TRUE|false|False|FALSE|null|Null|NULL|~)$/;
const BLOCK_SCALAR = /^[|>][-+0-9]*$/;
const FLOW_DELIM = '[]{},';

function scalarKind(text: string): TokenKind {
  if (NUMBER.test(text)) return 'number';
  if (LITERAL.test(text)) return 'literal';
  return 'string';
}

/** End of the quoted scalar opening at `at`, or the line's end when it is not closed. */
function quotedEnd(line: string, at: number): number {
  const q = line[at];
  let k = at + 1;
  while (k < line.length) {
    if (q === '"' && line[k] === '\\') {
      k += 2;
      continue;
    }
    if (line[k] === q) {
      if (q === "'" && line[k + 1] === "'") {
        k += 2;
        continue;
      }
      return k + 1;
    }
    k++;
  }
  return line.length;
}

/** Whether `:` at `at` separates a key from its value (followed by a space, the end, or in flow, a delimiter). */
function isMapColon(line: string, at: number, flow: boolean): boolean {
  if (line[at] !== ':') return false;
  const next = line[at + 1];
  return next === undefined || next === ' ' || next === '\t' || (flow && FLOW_DELIM.includes(next));
}

/** Whether a comment starts at `at`: a `#` at the line's start or after whitespace. */
function isComment(line: string, at: number): boolean {
  return line[at] === '#' && (at === 0 || line[at - 1] === ' ' || line[at - 1] === '\t');
}

class LineTokens {
  readonly out: Token[] = [];
  readonly line: string;
  pos = 0;
  constructor(line: string) {
    this.line = line;
  }

  push(end: number, kind: TokenKind): void {
    if (end <= this.pos) return;
    const last = this.out.at(-1);
    const text = this.line.slice(this.pos, end);
    if (last && last.kind === kind && kind === 'space') last.text += text;
    else this.out.push({ text, kind });
    this.pos = end;
  }

  space(): void {
    let k = this.pos;
    while (k < this.line.length && (this.line[k] === ' ' || this.line[k] === '\t')) k++;
    this.push(k, 'space');
  }

  done(): boolean {
    return this.pos >= this.line.length;
  }

  /** A comment to the line's end, when one starts here. */
  comment(): boolean {
    if (!isComment(this.line, this.pos)) return false;
    this.push(this.line.length, 'comment');
    return true;
  }

  /** End of a plain scalar starting here, before a map colon, a comment or (in flow) a delimiter. */
  plainEnd(flow: boolean): number {
    let k = this.pos;
    while (k < this.line.length) {
      if (isMapColon(this.line, k, flow) || isComment(this.line, k)) break;
      if (flow && FLOW_DELIM.includes(this.line[k])) break;
      k++;
    }
    while (k > this.pos && (this.line[k - 1] === ' ' || this.line[k - 1] === '\t')) k--;
    return k;
  }

  /** A scalar here, as a key when a map colon follows it; `true` when it was a key. */
  scalar(flow: boolean): boolean {
    const quoted = this.line[this.pos] === '"' || this.line[this.pos] === "'";
    const end = quoted ? quotedEnd(this.line, this.pos) : this.plainEnd(flow);
    let colon = end;
    while (this.line[colon] === ' ' || this.line[colon] === '\t') colon++;
    const key = isMapColon(this.line, colon, flow);
    const text = this.line.slice(this.pos, end);
    this.push(end, key ? 'key' : quoted ? 'string' : scalarKind(text));
    if (key) {
      this.space();
      this.push(this.pos + 1, 'punct');
    }
    return key;
  }

  /** A flow collection from here to the line's end or a comment. */
  flow(): void {
    while (!this.done()) {
      this.space();
      if (this.done() || this.comment()) return;
      const c = this.line[this.pos];
      if (FLOW_DELIM.includes(c) || isMapColon(this.line, this.pos, true)) {
        this.push(this.pos + 1, 'punct');
        continue;
      }
      const before = this.pos;
      this.scalar(true);
      if (this.pos === before) this.push(this.pos + 1, 'string');
    }
  }
}

/**
 * The tokens of each line of `text`, as `yamlLines` cuts it. A line that is part of a block scalar
 * (`key: |` and the lines indented under it) is one string after its indentation.
 */
export function yamlTokens(text: string): Token[][] {
  let scalarIndent: number | null = null;
  return yamlLines(text).map((line) => {
    const t = new LineTokens(line);
    t.space();
    if (scalarIndent !== null) {
      if (t.done() || t.pos > scalarIndent) {
        t.push(line.length, 'string');
        return t.out;
      }
      scalarIndent = null;
    }
    // The column of the node a block scalar belongs to: its key's, or the list dash's before it.
    let owner = t.pos;
    while (!t.done()) {
      if (t.comment()) return t.out;
      if (/^-(\s|$)/.test(line.slice(t.pos))) {
        owner = t.pos;
        t.push(t.pos + 1, 'punct');
        t.space();
        continue;
      }
      break;
    }
    if (t.done()) return t.out;
    // A block scalar as a list entry (`- |-`, how serde_yaml writes a multi-line string in a list)
    // has no key: it belongs to the dash.
    const entry = t.plainEnd(false);
    if (BLOCK_SCALAR.test(line.slice(t.pos, entry))) {
      t.push(entry, 'punct');
      scalarIndent = owner;
      t.space();
      if (!t.comment()) t.push(line.length, 'string');
      return t.out;
    }
    const c = line[t.pos];
    const keyAt = t.pos;
    if (c !== '[' && c !== '{' && t.scalar(false)) {
      owner = keyAt;
      t.space();
    }
    if (t.done() || t.comment()) return t.out;
    const value = line[t.pos];
    if (value === '[' || value === '{') {
      t.flow();
      return t.out;
    }
    const rest = t.plainEnd(false);
    if (BLOCK_SCALAR.test(line.slice(t.pos, rest))) {
      t.push(rest, 'punct');
      scalarIndent = owner;
    } else if (value === '"' || value === "'") {
      t.push(quotedEnd(line, t.pos), 'string');
    } else {
      t.push(rest, scalarKind(line.slice(t.pos, rest)));
    }
    t.space();
    if (!t.comment()) t.push(line.length, 'string');
    return t.out;
  });
}
