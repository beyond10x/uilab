import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { yamlLines, yamlTokens, type TokenKind } from './yamltokens.ts';

/** The tokens of one line that are not whitespace, as `kind:text`. */
function shown(line: string): string[] {
  return yamlTokens(line)[0]
    .filter((t) => t.kind !== 'space')
    .map((t) => `${t.kind}:${t.text}`);
}

/** The text of every token of `kind` in the document. */
function ofKind(text: string, kind: TokenKind): string[] {
  return yamlTokens(text)
    .flat()
    .filter((t) => t.kind === kind)
    .map((t) => t.text);
}

test('a key and a plain value', () => {
  assert.deepEqual(shown('    kind: dashboard_page'), ['key:kind', 'punct::', 'string:dashboard_page']);
  assert.deepEqual(shown('shells:'), ['key:shells', 'punct::']);
});

test('numbers and literals are told apart from strings', () => {
  assert.deepEqual(shown('size: 5'), ['key:size', 'punct::', 'number:5']);
  assert.deepEqual(shown('ratio: -1.5e3'), ['key:ratio', 'punct::', 'number:-1.5e3']);
  assert.deepEqual(shown('collapsible: true'), ['key:collapsible', 'punct::', 'literal:true']);
  assert.deepEqual(shown('default: null'), ['key:default', 'punct::', 'literal:null']);
  assert.deepEqual(shown('version: 1.2.3'), ['key:version', 'punct::', 'string:1.2.3']);
  assert.deepEqual(shown("subtitle: ''"), ['key:subtitle', 'punct::', "string:''"]);
});

test('comments, whole-line and trailing, but not inside a quoted string or a URL', () => {
  assert.deepEqual(shown('  # a comment'), ['comment:# a comment']);
  assert.deepEqual(shown('title: Loans  # shown in the menu'), ['key:title', 'punct::', 'string:Loans', 'comment:# shown in the menu']);
  assert.deepEqual(shown('title: "a # b"'), ['key:title', 'punct::', 'string:"a # b"']);
  assert.deepEqual(shown('link: http://x.test/#top'), ['key:link', 'punct::', 'string:http://x.test/#top']);
});

test('list entries and quoted keys', () => {
  assert.deepEqual(shown('  - name: circulation'), ['punct:-', 'key:name', 'punct::', 'string:circulation']);
  assert.deepEqual(shown('    - overview'), ['punct:-', 'string:overview']);
  assert.deepEqual(shown('  "members":'), ['key:"members"', 'punct::']);
  assert.deepEqual(shown("text: 'a: b'"), ['key:text', 'punct::', "string:'a: b'"]);
});

test('flow maps and lists colour their own keys and scalars', () => {
  assert.deepEqual(shown('reads: {view: staff.Me, size: 3}'), [
    'key:reads', 'punct::', 'punct:{', 'key:view', 'punct::', 'string:staff.Me', 'punct:,', 'key:size', 'punct::', 'number:3', 'punct:}',
  ]);
  assert.deepEqual(shown('columns: [{field: title}, "due"]'), [
    'key:columns', 'punct::', 'punct:[', 'punct:{', 'key:field', 'punct::', 'string:title', 'punct:}', 'punct:,', 'string:"due"', 'punct:]',
  ]);
});

test('a block scalar is a string to its end, whatever its lines look like', () => {
  const doc = ['text: |-', '  kind: not a key', '  # not a comment', '', '  42', 'next: 1'].join('\n');
  assert.deepEqual(ofKind(doc, 'key'), ['text', 'next']);
  assert.deepEqual(ofKind(doc, 'comment'), []);
  assert.deepEqual(ofKind(doc, 'string'), ['kind: not a key', '# not a comment', '42']);
});

test('the tokens of a line spell the line exactly, for the whole example document', () => {
  const text = readFileSync(new URL('../../../examples/library/library.ui.yaml', import.meta.url), 'utf8');
  const lines = yamlLines(text);
  const tokens = yamlTokens(text);
  assert.equal(tokens.length, lines.length);
  tokens.forEach((line, i) => assert.equal(line.map((t) => t.text).join(''), lines[i], `line ${i + 1}`));
  assert.ok(ofKind(text, 'key').includes('placement_profile'));
  assert.ok(ofKind(text, 'string').includes('ui-spec/1'));
});

test('lines split the way the view numbers them', () => {
  assert.deepEqual(yamlLines('a: 1\r\nb: 2\n'), ['a: 1', 'b: 2']);
  assert.deepEqual(yamlLines(''), ['']);
});
