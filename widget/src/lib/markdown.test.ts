import { test } from 'node:test';
import assert from 'node:assert/strict';
import { escapeHtml, renderInline, renderMarkdown, tableCells } from './markdown.ts';

test('headings of every level, trailing hashes dropped', () => {
  assert.equal(renderMarkdown('# Title'), '<h1>Title</h1>');
  assert.equal(renderMarkdown('### Pages ###'), '<h3>Pages</h3>');
  assert.equal(renderMarkdown('###### six'), '<h6>six</h6>');
  assert.equal(renderMarkdown('#nospace'), '<p>#nospace</p>');
});

test('paragraphs join their lines and split on blank lines', () => {
  assert.equal(renderMarkdown('one\ntwo\n\nthree'), '<p>one two</p>\n<p>three</p>');
  assert.equal(renderMarkdown('text\n# Heading'), '<p>text</p>\n<h1>Heading</h1>');
});

test('HTML is escaped everywhere', () => {
  assert.equal(escapeHtml(`<a href="x">'&'</a>`), '&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;');
  assert.equal(renderMarkdown('<script>alert(1)</script>'), '<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>');
  assert.equal(renderMarkdown('# <b>x</b>'), '<h1>&lt;b&gt;x&lt;/b&gt;</h1>');
  assert.equal(renderMarkdown('```\n<b>&</b>\n```'), '<pre><code>&lt;b&gt;&amp;&lt;/b&gt;</code></pre>');
  assert.equal(renderMarkdown('| <i> |\n|---|\n| `<x>` |'), '<table><thead><tr><th>&lt;i&gt;</th></tr></thead><tbody><tr><td><code>&lt;x&gt;</code></td></tr></tbody></table>');
});

test('inline code is literal; bold is not applied inside it', () => {
  assert.equal(renderInline('run `task check` now'), 'run <code>task check</code> now');
  assert.equal(renderInline('`**not bold**` and **bold**'), '<code>**not bold**</code> and <strong>bold</strong>');
  assert.equal(renderInline('``a ` b``'), '<code>a ` b</code>');
  assert.equal(renderInline('a lone ` tick'), 'a lone ` tick');
  assert.equal(renderInline('**a** and **b c**'), '<strong>a</strong> and <strong>b c</strong>');
  assert.equal(renderInline('2 ** 3'), '2 ** 3');
});

test('code fences keep their lines and language', () => {
  assert.equal(renderMarkdown('```yaml\npages:\n  loans:\n```\nafter'), '<pre><code class="language-yaml">pages:\n  loans:</code></pre>\n<p>after</p>');
  assert.equal(renderMarkdown('```\n# not a heading\n- not a list\n```'), '<pre><code># not a heading\n- not a list</code></pre>');
  assert.equal(renderMarkdown('```\nunclosed'), '<pre><code>unclosed</code></pre>');
});

test('bullet lists, nested by indentation, with continuation lines', () => {
  assert.equal(renderMarkdown('- a\n- b'), '<ul><li>a</li><li>b</li></ul>');
  assert.equal(renderMarkdown('* a\n  * a1\n  * a2\n* b'), '<ul><li>a<ul><li>a1</li><li>a2</li></ul></li><li>b</li></ul>');
  assert.equal(renderMarkdown('- a\n  more\n- b'), '<ul><li>a more</li><li>b</li></ul>');
  assert.equal(renderMarkdown('- a\n\n- b\n\npara'), '<ul><li>a</li><li>b</li></ul>\n<p>para</p>');
  assert.equal(renderMarkdown('1. one\n2. two'), '<ol><li>one</li><li>two</li></ol>');
  assert.equal(renderMarkdown('- **Loans**: `loans.All`'), '<ul><li><strong>Loans</strong>: <code>loans.All</code></li></ul>');
});

test('pipe tables with header, alignment, escaped pipes and short rows', () => {
  assert.deepEqual(tableCells('| a | b \\| c | d |'), ['a', 'b | c', 'd']);
  assert.equal(
    renderMarkdown('| view | draft |\n|:-----|------:|\n| loans.All | no |\n| x |'),
    '<table><thead><tr><th style="text-align:left">view</th><th style="text-align:right">draft</th></tr></thead>' +
      '<tbody><tr><td style="text-align:left">loans.All</td><td style="text-align:right">no</td></tr>' +
      '<tr><td style="text-align:left">x</td><td style="text-align:right"></td></tr></tbody></table>',
  );
  assert.equal(renderMarkdown('| not a table |\nnext'), '<p>| not a table | next</p>');
});

test('horizontal rules and a mixed document', () => {
  const md = '# Library\n\nA **lending** app.\n\n---\n\n## Pages\n\n| page | kind |\n|---|---|\n| loans | list_page |\n\n- one\n- two\n';
  assert.equal(
    renderMarkdown(md),
    [
      '<h1>Library</h1>',
      '<p>A <strong>lending</strong> app.</p>',
      '<hr>',
      '<h2>Pages</h2>',
      '<table><thead><tr><th>page</th><th>kind</th></tr></thead><tbody><tr><td>loans</td><td>list_page</td></tr></tbody></table>',
      '<ul><li>one</li><li>two</li></ul>',
    ].join('\n'),
  );
  assert.equal(renderMarkdown(''), '');
});
