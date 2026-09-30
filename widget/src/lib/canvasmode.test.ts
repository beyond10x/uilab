import { test } from 'node:test';
import assert from 'node:assert/strict';
import { accountName, CANVAS_MODE_KEY, createCanvasMode, MODE_HELP, togglesMode, type ModeStorage } from './canvasmode.ts';

/** A localStorage stand-in that keeps what it is given. */
function memory(initial: Record<string, string> = {}): ModeStorage & { data: Record<string, string> } {
  const data = { ...initial };
  return {
    data,
    getItem: (k) => (k in data ? data[k] : null),
    setItem: (k, v) => {
      data[k] = v;
    },
  };
}

const throwing: ModeStorage = {
  getItem: () => {
    throw new Error('storage disabled');
  },
  setItem: () => {
    throw new Error('quota exceeded');
  },
};

test('the canvas starts in structure mode when nothing is stored', () => {
  assert.equal(createCanvasMode(() => memory()).mode.value, 'structure');
  assert.equal(createCanvasMode(() => null).mode.value, 'structure');
  assert.equal(createCanvasMode(() => throwing).mode.value, 'structure');
  assert.equal(createCanvasMode(() => memory({ [CANVAS_MODE_KEY]: 'bogus' })).mode.value, 'structure');
});

test('toggle flips structure and preview and stores each mode', () => {
  const store = memory();
  const m = createCanvasMode(() => store);
  assert.equal(m.toggle(), 'preview');
  assert.equal(m.mode.value, 'preview');
  assert.equal(store.data[CANVAS_MODE_KEY], 'preview');
  assert.equal(m.toggle(), 'structure');
  assert.equal(m.mode.value, 'structure');
  assert.equal(store.data[CANVAS_MODE_KEY], 'structure');
});

test('a reload restores the stored mode', () => {
  const store = memory();
  createCanvasMode(() => store).toggle();
  assert.equal(createCanvasMode(() => store).mode.value, 'preview');
});

test('a storage that refuses writes still toggles the mode on screen', () => {
  const m = createCanvasMode(() => throwing);
  assert.equal(m.toggle(), 'preview');
  assert.equal(m.mode.value, 'preview');
});

const key = (k: string, mods: { ctrlKey?: boolean; metaKey?: boolean; altKey?: boolean } = {}) => ({ key: k, ...mods });

test('a plain p toggles the mode; with a modifier, or another key, it does not', () => {
  assert.equal(togglesMode(key('p')), true);
  assert.equal(togglesMode(key('P')), true, 'caps lock or shift');
  for (const mod of ['ctrlKey', 'metaKey', 'altKey'] as const) assert.equal(togglesMode(key('p', { [mod]: true })), false, mod);
  for (const other of ['Enter', ' ', 'Escape', '?', '1', 'o']) assert.equal(togglesMode(key(other)), false, other);
});

test('help lists the mode key the toggle answers to', () => {
  const [keys, what] = MODE_HELP;
  assert.deepEqual(keys, ['p']);
  assert.ok(togglesMode(key(keys[0])));
  assert.match(what, /preview/);
  assert.match(what, /structure/);
});

test('the account menu reads the staff member name from the first row', () => {
  assert.equal(accountName([{ id: 's-1', name: 'Example Librarian', email: 'librarian@example.com' }]), 'Example Librarian');
  assert.equal(accountName([{ id: 's-1', display_name: 'Ex Lib' }]), 'Ex Lib');
  assert.equal(accountName([{ id: 's-1', email: 'librarian@example.com' }]), 'librarian@example.com');
  assert.equal(accountName([{ id: 's-1', name: '  ' }]), null);
  assert.equal(accountName([]), null);
  assert.equal(accountName(undefined), null);
  assert.equal(accountName(['not a row', 3]), null);
});
