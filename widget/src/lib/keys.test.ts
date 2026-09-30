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

const withAttrs = (tagName: string, attrs: Record<string, string>) => ({
  tagName,
  isContentEditable: false,
  getAttribute: (name: string) => attrs[name] ?? null,
});

test('a summary keeps Enter: it opens and closes its details', () => {
  assert.equal(enterAccepts(el('SUMMARY')), false);
});

test('an element out of the tab order (tabindex -1) leaves Enter to the proposal', () => {
  assert.equal(enterAccepts(withAttrs('BUTTON', { tabindex: '-1' })), true);
  assert.equal(enterAccepts(withAttrs('A', { tabindex: '-1' })), true);
  assert.equal(enterAccepts(withAttrs('BUTTON', { tabindex: '0' })), false);
});

test('an element focused by a mouse click leaves Enter to the proposal', () => {
  assert.equal(enterAccepts(el('BUTTON'), true), true);
  assert.equal(enterAccepts(el('A'), true), true);
  assert.equal(enterAccepts(el('INPUT'), true), false, 'a field keeps Enter however it was focused');
  assert.equal(enterAccepts(el('BUTTON'), false), false);
});
