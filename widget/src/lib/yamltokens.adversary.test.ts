import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { yamlLines, yamlTokens, type TokenKind } from './yamltokens.ts';

function shown(line: string): string[] {
  return yamlTokens(line)[0]
    .filter((t) => t.kind !== 'space')
    .map((t) => `${t.kind}:${t.text}`);
}

function ofKind(text: string, kind: TokenKind): string[] {
  return yamlTokens(text)
    .flat()
    .filter((t) => t.kind === kind)
    .map((t) => t.text);
}

function spells(text: string, what: string): void {
  const lines = yamlLines(text);
  const tokens = yamlTokens(text);
  assert.equal(tokens.length, lines.length, `${what}: line count`);
  tokens.forEach((line, i) => {
    assert.equal(line.map((t) => t.text).join(''), lines[i], `${what}: line ${i + 1}`);
    line.forEach((t) => assert.ok(t.text.length > 0, `${what}: empty token on line ${i + 1}`));
  });
}

test('adversary: a multi-line string in a list (`- |-`, what serde_yaml writes) is a string to its end', () => {
  const doc = ['notes:', '- |-', '  kind: not a key', '  # not a comment', '- plain', 'next: 1'].join('\n');
  assert.deepEqual(ofKind(doc, 'key'), ['notes', 'next']);
  assert.deepEqual(ofKind(doc, 'comment'), []);
  assert.deepEqual(ofKind(doc, 'string'), ['kind: not a key', '# not a comment', 'plain']);
});

test('adversary: a folded scalar as a nested list entry is a string to its end', () => {
  const doc = ['    hidden:', '    - >', '      title: folded', '    - x'].join('\n');
  assert.deepEqual(ofKind(doc, 'key'), ['hidden']);
  assert.deepEqual(ofKind(doc, 'string'), ['title: folded', 'x']);
});

test('adversary: a block scalar under a list entry key ends at the entry sibling key', () => {
  const doc = ['body:', '- name: t', '  text: |-', '    a: b', '  kind: x'].join('\n');
  assert.deepEqual(ofKind(doc, 'key'), ['body', 'name', 'text', 'kind']);
  assert.deepEqual(ofKind(doc, 'string'), ['t', 'a: b', 'x']);
});

test('adversary: plain scalars the server reads as strings are not coloured numbers', () => {
  assert.deepEqual(shown('code: 1_000'), ['key:code', 'punct::', 'string:1_000']);
  assert.deepEqual(shown('code: 12__'), ['key:code', 'punct::', 'string:12__']);
});

test('adversary: numbers, versions, literals', () => {
  assert.deepEqual(shown('a: 0x1f'), ['key:a', 'punct::', 'number:0x1f']);
  assert.deepEqual(shown('a: 1e3'), ['key:a', 'punct::', 'number:1e3']);
  assert.deepEqual(shown('a: -5'), ['key:a', 'punct::', 'number:-5']);
  assert.deepEqual(shown('- -5'), ['punct:-', 'number:-5']);
  assert.deepEqual(shown('a: 1.2.3'), ['key:a', 'punct::', 'string:1.2.3']);
  assert.deepEqual(shown('a: v1.2'), ['key:a', 'punct::', 'string:v1.2']);
  assert.deepEqual(shown('a: .inf'), ['key:a', 'punct::', 'number:.inf']);
  assert.deepEqual(shown("a: '5'"), ['key:a', 'punct::', "string:'5'"]);
  assert.deepEqual(shown('a: ~'), ['key:a', 'punct::', 'literal:~']);
  assert.deepEqual(shown('a: yes'), ['key:a', 'punct::', 'string:yes']);
  assert.deepEqual(shown('a: truth'), ['key:a', 'punct::', 'string:truth']);
});

test('adversary: keys with colons and quotes, URLs, comments inside and after strings', () => {
  assert.deepEqual(shown('a:b: c'), ['key:a:b', 'punct::', 'string:c']);
  assert.deepEqual(shown("'a: b': c"), ["key:'a: b'", 'punct::', 'string:c']);
  assert.deepEqual(shown('"a\\"b": c'), ['key:"a\\"b"', 'punct::', 'string:c']);
  assert.deepEqual(shown("'it''s': c"), ["key:'it''s'", 'punct::', 'string:c']);
  assert.deepEqual(shown('- http://x.test:8080/a'), ['punct:-', 'string:http://x.test:8080/a']);
  assert.deepEqual(shown("a: 'x # y' # z"), ['key:a', 'punct::', "string:'x # y'", 'comment:# z']);
  assert.deepEqual(shown('a: x#y'), ['key:a', 'punct::', 'string:x#y']);
  assert.deepEqual(shown('a:# not a comment'), ['string:a:# not a comment']);
  assert.deepEqual(shown('- # c'), ['punct:-', 'comment:# c']);
  assert.deepEqual(shown('a: # c'), ['key:a', 'punct::', 'comment:# c']);
});

test('adversary: flow collections nest and keep URLs whole', () => {
  assert.deepEqual(shown('m: {a: 1, b: [x, y]}'), [
    'key:m', 'punct::', 'punct:{', 'key:a', 'punct::', 'number:1', 'punct:,', 'key:b', 'punct::',
    'punct:[', 'string:x', 'punct:,', 'string:y', 'punct:]', 'punct:}',
  ]);
  assert.deepEqual(shown('l: [http://x.test/a, "b, c"] # d'), [
    'key:l', 'punct::', 'punct:[', 'string:http://x.test/a', 'punct:,', 'string:"b, c"', 'punct:]', 'comment:# d',
  ]);
  assert.deepEqual(shown('e: []'), ['key:e', 'punct::', 'punct:[', 'punct:]']);
});

test('adversary: empty values, bare dashes, document markers', () => {
  assert.deepEqual(shown('a:'), ['key:a', 'punct::']);
  assert.deepEqual(shown('a: '), ['key:a', 'punct::']);
  assert.deepEqual(shown('-'), ['punct:-']);
  assert.deepEqual(shown('- '), ['punct:-']);
  assert.deepEqual(shown("a: ''"), ['key:a', 'punct::', "string:''"]);
  assert.deepEqual(yamlTokens(''), [[]]);
});

test('adversary: unicode keys and values keep their characters', () => {
  assert.deepEqual(shown('名前: 値 # 注'), ['key:名前', 'punct::', 'string:値', 'comment:# 注']);
  assert.deepEqual(shown('- 😀: "🙂 # x"'), ['punct:-', 'key:😀', 'punct::', 'string:"🙂 # x"']);
});

test('adversary: CRLF, lone CR, tabs, unclosed quotes and odd input all spell their lines', () => {
  const odd = [
    'a: 1\r\nb: |\r\n  x: y\r\nc: "unclosed',
    "k: 'also unclosed\n- \"x\\",
    'a:\tb\n\t- c\n  - - - d: e\n? complex\n: value',
    '{a: 1, b: [x, {c: d}]}\n[1, 2]: z\n{"a":1}\n--- |\n...\n',
    'x: &anchor 1\ny: *anchor\nz: !!str 5\nw: !tag |\n  body',
    '\n\n\n',
    '   \n  # c\n- |+\n\n  a\n\n',
  ];
  odd.forEach((text, i) => spells(text, `odd ${i}`));
});

test('adversary: every example document and fixture spells its lines', () => {
  const root = fileURLToPath(new URL('../../../examples', import.meta.url));
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const name of readdirSync(dir)) {
      const p = join(dir, name);
      if (statSync(p).isDirectory()) walk(p);
      else if (/\.ya?ml$/.test(name)) files.push(p);
    }
  };
  walk(root);
  assert.ok(files.length >= 4, `found ${files.length} example files`);
  for (const f of files) spells(readFileSync(f, 'utf8'), f);
});

test('adversary: a very long line is cut in linear time', () => {
  const flow = `k: [${Array.from({ length: 50_000 }, (_, i) => `v${i}`).join(', ')}]`;
  const plain = `k: ${'x '.repeat(200_000)}# end`;
  const t0 = performance.now();
  spells(`${flow}\n${plain}`, 'long');
  assert.ok(performance.now() - t0 < 2000, 'took more than 2 s');
  assert.equal(ofKind(flow, 'string').length, 50_000);
});
