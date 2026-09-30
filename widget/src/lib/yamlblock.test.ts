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

test('collection items in the list form the server writes, and in the old map form', () => {
  const doc = [
    'pages:',
    '  members:',
    '    sections:',
    '      list:',
    '        component: collection',
    '        item:',
    '        - name: badge',
    '          primitive: badge',
    '        - name: card',
    '          component: member_card',
    '      old:',
    '        component: collection',
    '        item:',
    '          count:',
    '            component: metric',
    '',
  ].join('\n');
  const at = (p: string) => {
    const r = yamlBlock(doc, p);
    return r && [r.start, r.end];
  };
  assert.deepEqual(at('page:members/section:list/item:card'), [8, 10]);
  assert.deepEqual(at('page:members/section:list/item:badge'), [6, 8]);
  assert.deepEqual(at('page:members/section:old/item:count'), [13, 15]);
  assert.equal(at('page:members/section:list/item:nope'), null);
});

test('widgets under the root and their body nodes', () => {
  const doc = ['widgets:', '  member_card:', '    summary: s', '    body:', '    - name: title', '      primitive: text', '    - name: tone', '      primitive: badge', ''].join('\n');
  const r = yamlBlock(doc, 'component:member_card')!;
  assert.deepEqual([r.start, r.end], [1, 8]);
  const n = yamlBlock(doc, 'component:member_card/node:tone')!;
  assert.deepEqual([n.start, n.end], [6, 8]);
});

test('a section, an item in a list and a widget body node, in the layout the server writes', () => {
  const doc = [
    'widgets:', //                         0
    '  loan_card:', //                     1
    '    arrange: column', //              2
    '    body:', //                        3
    '    - name: title', //                4
    '      primitive: text', //            5
    '      style: heading', //             6
    '    - name: state', //                7
    '      primitive: badge', //           8
    '      tone: neutral', //              9
    'pages:', //                           10
    '  showcase:', //                      11
    '    kind: dashboard_page', //         12
    '    sections:', //                    13
    '      loans:', //                     14
    '        component: collection', //    15
    '        reads:', //                   16
    '          view: loans.All', //        17
    '        columns:', //                 18
    '        - field: title', //           19
    '        item:', //                    20
    '        - name: card', //             21
    '          component: loan_card', //   22
    '          args:', //                  23
    '            loan: row', //            24
    '        - name: due', //              25
    '          primitive: text', //        26
    '      highlights:', //                27
    '        component: board', //         28
    '        widgets:', //                 29
    '          overdue:', //               30
    '            component: metric', //    31
    '            from: overdue', //        32
    '    overlays:', //                    33
    '      edit:', //                      34
    '        kind: drawer', //             35
    '',
  ].join('\n');
  const at = (p: string) => {
    const r = yamlBlock(doc, p);
    return r && [r.start, r.end];
  };
  assert.deepEqual(at('page:showcase/section:loans'), [14, 27], 'a section holds its lists and ends at its sibling');
  assert.deepEqual(at('page:showcase/section:highlights'), [27, 33], 'the last section ends at the overlays');
  assert.deepEqual(at('page:showcase/section:loans/item:card'), [21, 25], 'an item holds its nested keys');
  assert.deepEqual(at('page:showcase/section:loans/item:due'), [25, 27], 'the last item ends at the next section');
  assert.deepEqual(at('page:showcase/section:highlights/widget:overdue'), [30, 33]);
  assert.deepEqual(at('component:loan_card/node:title'), [4, 7]);
  assert.deepEqual(at('component:loan_card/node:state'), [7, 10], 'the last body node ends at the next top-level key');
});
