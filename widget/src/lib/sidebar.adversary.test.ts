import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parse } from 'vue/compiler-sfc';
import { SIDEBAR_ORDER } from './sidebar.ts';

/**
 * The unit's own order test reads the `data-section="…"` strings in source order, and its chip test
 * matches an `<input aria-label="your name">` anywhere after an `op-chip` class. Both stay green
 * when the proposal card is moved below the activity feed, or the rename input out of the chip,
 * as long as the attribute text keeps its order. These cases read the parsed template instead:
 * what is inside which element.
 */

interface Prop {
  type: number;
  name: string;
  value?: { content: string };
  arg?: { content: string };
  exp?: { content: string };
}

interface Node {
  type: number;
  tag?: string;
  props?: Prop[];
  children?: Node[];
}

const ELEMENT = 1;
const ATTRIBUTE = 6;
const DIRECTIVE = 7;

function template(component: string): Node {
  const sfc = readFileSync(new URL(`../components/${component}`, import.meta.url), 'utf8');
  const { descriptor, errors } = parse(sfc, { filename: component });
  assert.deepEqual(errors, []);
  return descriptor.template!.ast as unknown as Node;
}

function elements(n: Node): Node[] {
  return (n.children ?? []).flatMap((c) => (c.type === ELEMENT ? [c] : c.children ? elements(c) : []));
}

function attr(n: Node, name: string): string | undefined {
  return n.props?.find((p) => p.type === ATTRIBUTE && p.name === name)?.value?.content;
}

function bound(n: Node, name: string): string | undefined {
  return n.props?.find((p) => p.type === DIRECTIVE && p.name === 'bind' && p.arg?.content === name)?.exp?.content;
}

function hasClass(n: Node, cls: string): boolean {
  return (attr(n, 'class') ?? '').split(/\s+/).includes(cls);
}

function all(n: Node, pred: (e: Node) => boolean): Node[] {
  return elements(n).flatMap((e) => [...(pred(e) ? [e] : []), ...all(e, pred)]);
}

const tag = (t: string) => (e: Node) => e.tag === t;
const cls = (c: string) => (e: Node) => hasClass(e, c);

function sections(): Map<string, Node> {
  const [root] = elements(template('SidebarPanel.vue'));
  assert.ok(hasClass(root, 'side'), 'the sidebar root is .side');
  return new Map(elements(root).map((e) => [attr(e, 'data-section') ?? `(${e.tag})`, e]));
}

test('every direct child of the sidebar root is a section, in the story order', () => {
  const [root] = elements(template('SidebarPanel.vue'));
  assert.deepEqual(
    elements(root).map((e) => attr(e, 'data-section') ?? `(${e.tag} without data-section)`),
    [...SIDEBAR_ORDER],
  );
});

test('each sidebar section holds its own content, and the proposal card is only in the card section', () => {
  const s = sections();
  const inside = (section: string, pred: (e: Node) => boolean, what: string) =>
    assert.ok(all(s.get(section)!, pred).length > 0, `${what} is inside the ${section} section`);
  inside('header', tag('PresenceStrip'), 'the operator chips');
  inside('header', cls('conn'), 'the connection dot');
  inside('header', cls('badge'), 'the findings badge');
  inside('header', cls('help-button'), 'the help button');
  inside('card', tag('ProposalCard'), 'the proposal card');
  inside('card', cls('undo-row'), 'undo');
  inside('input', cls('mic'), 'the microphone');
  inside('input', cls('say'), 'the text input');
  inside('tree', tag('TreeNode'), 'the tree');
  inside('activity', tag('ActivityFeed'), 'the activity feed');

  const cards = [...s].filter(([, e]) => all(e, tag('ProposalCard')).length > 0).map(([name]) => name);
  assert.deepEqual(cards, ['card']);
});

test('the file path is the tooltip of the document title in the tree section, and has no row of its own', () => {
  const s = sections();
  const titles = all(s.get('tree')!, cls('doc-title'));
  assert.equal(titles.length, 1);
  assert.equal(bound(titles[0], 'title'), 'doc.file');
  const [root] = elements(template('SidebarPanel.vue'));
  assert.deepEqual(all(root, cls('file')), []);
});

test('the rename input and the rename button are inside the operator chip', () => {
  const chips = all(template('PresenceStrip.vue'), cls('op-chip'));
  assert.equal(chips.length, 1);
  const inChip = all(chips[0], () => true);
  assert.ok(
    inChip.some((e) => e.tag === 'input' && attr(e, 'aria-label') === 'your name'),
    'the name input is a descendant of .op-chip',
  );
  assert.ok(
    inChip.some((e) => e.tag === 'button' && attr(e, 'aria-label') === 'rename yourself'),
    'the rename button is a descendant of .op-chip',
  );
});
