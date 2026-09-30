import { test } from 'node:test';
import assert from 'node:assert/strict';
import { yamlBlock } from './yamlblock.ts';

const DOC = `format: ui-spec/1
app: library
title: Lending library
shells:
  app:
    regions:
      nav:
        kind: navigation
        props:
          collapsible: true
      main:
        kind: page_outlet
navigation:
  home: overview
  sections:
    - name: circulation
      label: Circulation
      pages: [overview, loans]
    - label: People
      name: people
      pages: [members]
    - {name: "staff", pages: [admin]}
pages:
  overview:
    kind: dashboard_page
    sections:
      list:
        component: metric
  loans:
    kind: list_page
    title: Loans
    sections:
      list:
        component: collection
        reads: {view: loans.All}
        columns: [{field: title}]

        # a comment inside the block
        row_actions: [{opens: edit}]
      board:
        component: board
        widgets:
          count:
            component: metric
    overlays:
      edit:
        kind: drawer
        component: form
        fields: [due]
  "members":
    kind: list_page
`;

const lines = DOC.split('\n');
/** The block as its first and last line, for readable failures. */
function block(path: string): [string, string] | null {
  const r = yamlBlock(DOC, path);
  return r ? [lines[r.start].trim(), lines[r.end - 1].trim()] : null;
}

test('a section is its key under sections under its page under pages', () => {
  const r = yamlBlock(DOC, 'page:loans/section:list')!;
  assert.equal(lines[r.start], '      list:');
  assert.equal(lines[r.end - 1], '        row_actions: [{opens: edit}]', 'blank lines and comments inside stay in the block');
  assert.deepEqual(block('page:overview/section:list'), ['list:', 'component: metric'], 'the same name on another page');
});

test('pages, overlays, widgets, shells and regions', () => {
  assert.deepEqual(block('page:loans'), ['loans:', 'fields: [due]']);
  assert.deepEqual(block('page:loans/overlay:edit'), ['edit:', 'fields: [due]']);
  assert.deepEqual(block('page:loans/section:board/widget:count'), ['count:', 'component: metric']);
  assert.deepEqual(block('shell:app/region:nav'), ['nav:', 'collapsible: true']);
  assert.deepEqual(block('shell:app/region:main'), ['main:', 'kind: page_outlet']);
  assert.deepEqual(block('page:members'), ['"members":', 'kind: list_page'], 'a quoted key');
});

test('the navigation and its sections, whichever line holds the name', () => {
  assert.deepEqual(block('nav'), ['navigation:', '- {name: "staff", pages: [admin]}']);
  assert.deepEqual(block('nav/nav_section:circulation'), ['- name: circulation', 'pages: [overview, loans]']);
  assert.deepEqual(block('nav/nav_section:people'), ['- label: People', 'pages: [members]']);
  assert.deepEqual(block('nav/nav_section:staff'), ['- {name: "staff", pages: [admin]}', '- {name: "staff", pages: [admin]}']);
});

test('the root and missing nodes have no block', () => {
  assert.equal(yamlBlock(DOC, '/'), null);
  assert.equal(yamlBlock(DOC, 'page:nope'), null);
  assert.equal(yamlBlock(DOC, 'page:loans/section:nope'), null);
  assert.equal(yamlBlock(DOC, 'nav/nav_section:nope'), null);
  assert.equal(yamlBlock(DOC, 'page:loans/section:list/item:x'), null);
  assert.equal(yamlBlock('', 'page:loans'), null);
});

test('a key does not match a longer key or a value', () => {
  const doc = 'pages:\n  loans_old:\n    kind: x\n  other:\n    title: loans:\n  loans:\n    kind: y\n';
  const r = yamlBlock(doc, 'page:loans')!;
  assert.deepEqual([r.start, r.end], [5, 7]);
});

test('CRLF line endings', () => {
  const r = yamlBlock(DOC.replace(/\n/g, '\r\n'), 'page:loans/overlay:edit')!;
  assert.equal(lines[r.start], '      edit:');
});
