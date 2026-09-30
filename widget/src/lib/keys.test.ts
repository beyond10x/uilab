import { test } from 'node:test';
import assert from 'node:assert/strict';
import { enterAccepts } from './keys.ts';

const el = (tagName: string, opts: { editable?: boolean; role?: string } = {}) => ({
  tagName,
  isContentEditable: !!opts.editable,
  getAttribute: (name: string) => (name === 'role' ? opts.role ?? null : null),
});

test('Enter on a focused button, link, field or editable region stays with that element', () => {
  for (const [what, target] of [
    ['button', el('BUTTON')],
    ['link', el('A')],
    ['input', el('INPUT')],
    ['textarea', el('TEXTAREA')],
    ['select', el('SELECT')],
    ['contenteditable', el('DIV', { editable: true })],
    ['role=button', el('SPAN', { role: 'button' })],
    ['role=link', el('SPAN', { role: 'link' })],
  ] as const) {
    assert.equal(enterAccepts(target), false, what);
  }
});

test('Enter anywhere else accepts the waiting proposal', () => {
  for (const [what, target] of [
    ['body', el('BODY')],
    ['a plain element', el('DIV')],
    ['a status region', el('SPAN', { role: 'status' })],
    ['no target', null],
    ['the window', {}],
  ] as const) {
    assert.equal(enterAccepts(target), true, what);
  }
});
