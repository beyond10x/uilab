import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { parse } from 'vue/compiler-sfc';
import { enterAccepts, type KeyTarget } from './keys.ts';
import { presenceChips } from './sidebar.ts';

/**
 * Adversary pass 2. keys.test.ts checks enterAccepts against element shapes the unit chose; these
 * cases read the elements the widget's own templates render, so an element the list forgot, or one
 * it takes that has no Enter of its own, shows up. The chip case evaluates the template's own
 * `:key` on the chips presenceChips returns before and after presence lists the local operator.
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

function template(file: string): Node {
  const sfc = readFileSync(new URL(`../${file}`, import.meta.url), 'utf8');
  const { descriptor, errors } = parse(sfc, { filename: file });
  assert.deepEqual(errors, []);
  return descriptor.template!.ast as unknown as Node;
}

function all(n: Node): Node[] {
  return (n.children ?? []).flatMap((c) => (c.type === ELEMENT ? [c, ...all(c)] : c.children ? all(c) : []));
}

function attr(n: Node, name: string): string | undefined {
  return n.props?.find((p) => p.type === ATTRIBUTE && p.name === name)?.value?.content;
}

function directive(n: Node, name: string, arg?: string): string | undefined {
  return n.props?.find((p) => p.type === DIRECTIVE && p.name === name && (arg === undefined || p.arg?.content === arg))?.exp?.content;
}

const FILES = ['App.vue', ...readdirSync(new URL('../components/', import.meta.url)).filter((f) => f.endsWith('.vue')).map((f) => `components/${f}`)];

/** A focused element as the key event's target sees it. */
function targetOf(n: Node): KeyTarget {
  return {
    tagName: n.tag!.toUpperCase(),
    isContentEditable: false,
    getAttribute: (name: string) => attr(n, name) ?? null,
  };
}

/** Elements Tab reaches whose Enter does something of their own: activate, follow, submit, toggle. */
function ownsEnter(n: Node): boolean {
  if (attr(n, 'tabindex') === '-1') return false;
  if (['button', 'select', 'textarea', 'input', 'summary'].includes(n.tag!)) return true;
  if (n.tag === 'a' && (attr(n, 'href') !== undefined || directive(n, 'bind', 'href') !== undefined)) return true;
  return ['button', 'link', 'checkbox', 'switch', 'tab', 'menuitem', 'option', 'treeitem'].includes(attr(n, 'role') ?? '');
}

test('Enter on any element the templates put in the tab order that has an Enter of its own stays with it', () => {
  const taken: string[] = [];
  for (const file of FILES) {
    for (const n of all(template(file))) {
      if (!/^[a-z]+$/.test(n.tag ?? '') || !ownsEnter(n)) continue;
      if (enterAccepts(targetOf(n))) taken.push(`${file} <${n.tag}${attr(n, 'class') ? ` class="${attr(n, 'class')}"` : ''}>`);
    }
  }
  assert.deepEqual(taken, [], 'a focused element whose own Enter the waiting proposal takes instead');
});

test('an inert canvas preview control, focused by the click that selected it, leaves Enter to the waiting proposal', () => {
  const inert: string[] = [];
  const kept: string[] = [];
  for (const file of FILES) {
    for (const n of all(template(file))) {
      if (attr(n, 'tabindex') !== '-1' || !['button', 'a'].includes(n.tag ?? '')) continue;
      inert.push(`${file} <${n.tag}>`);
      if (!enterAccepts(targetOf(n))) kept.push(`${file} <${n.tag}>`);
    }
  }
  assert.ok(inert.length >= 2, `the canvas renders inert preview buttons and links: ${inert.join(', ')}`);
  assert.deepEqual(kept, [], 'a clicked inert preview control that keeps Enter, so Enter does not accept');
});

test('the local chip keeps its v-for key when presence starts listing it, so a rename in progress is not remounted', () => {
  const chip = all(template('components/PresenceStrip.vue')).find((n) => (attr(n, 'class') ?? '').split(/\s+/).includes('op-chip'))!;
  const vFor = directive(chip, 'for');
  const key = directive(chip, 'bind', 'key');
  if (!vFor || !key) return;
  const alias = vFor.split(/\s+(?:in|of)\s+/)[0].trim();
  const keyOf = new Function(alias, `return (${key});`) as (o: unknown) => unknown;
  const listed = { id: 'h-2', name: 'Ada', kind: 'human' as const, colour: '#000', local: true };
  for (const conn of ['connecting', 'closed', 'open'] as const) {
    const [before] = presenceChips([], 'Ada', conn);
    const [after] = presenceChips([listed], 'Ada', 'open');
    assert.ok(before.local && after.local, 'the first chip is the local one both times');
    assert.equal(keyOf(after), keyOf(before), `the local chip's key from ${conn} to listed`);
  }
  const [shared] = presenceChips([listed], 'Ada', 'open');
  const [ambiguous] = presenceChips([{ ...listed, local: false }, { ...listed, id: 'h-3', local: false }], 'Ada', 'open');
  assert.equal(keyOf(ambiguous), keyOf(shared), 'the local chip key when a second tab of the same name joins');
});
