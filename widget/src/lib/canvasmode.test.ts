import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  accountChrome,
  accountName,
  CANVAS_MODE_KEY,
  createCanvasMode,
  emptyLine,
  MODE_HELP,
  modeButton,
  togglesMode,
  togglesModeOn,
  type ModeStorage,
} from './canvasmode.ts';

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

test('p switches the mode on the UI tab only', () => {
  assert.equal(togglesModeOn(key('p'), 'ui'), true);
  for (const view of ['yaml', 'docs', 'components']) assert.equal(togglesModeOn(key('p'), view), false, view);
  assert.equal(togglesModeOn(key('p', { ctrlKey: true }), 'ui'), false, 'a modifier still does not toggle');
  assert.equal(togglesModeOn(key('o'), 'ui'), false);
});

test('the mode button keeps one label and carries the mode in aria-pressed', () => {
  const structure = modeButton('structure');
  const preview = modeButton('preview');
  assert.equal(structure.label, 'Preview');
  assert.equal(preview.label, structure.label, 'the label does not follow the mode');
  assert.equal(structure.pressed, false);
  assert.equal(preview.pressed, true);
});

test('structure names the view in the empty line; preview never does', () => {
  const cases = [
    { view: undefined, loaded: false, count: 0, connOpen: true },
    { view: 'members.All', loaded: false, count: 0, connOpen: true },
    { view: 'members.All', loaded: false, count: 0, connOpen: false },
    { view: 'members.All', loaded: true, count: 0, connOpen: true },
  ];
  assert.deepEqual(
    cases.map((c) => emptyLine({ ...c, preview: false })),
    ['no data yet (no view)', 'loading members.All…', 'no data yet (members.All)', 'no data yet (members.All)'],
  );
  assert.deepEqual(
    cases.map((c) => emptyLine({ ...c, preview: true })),
    ['no data yet', 'loading…', 'no data yet', 'no data yet'],
  );
  for (const preview of [false, true]) assert.equal(emptyLine({ view: 'members.All', loaded: true, count: 2, connOpen: true, preview }), null);
});

test('the account chrome shows the name from its rows, marks a draft read as sample data, and falls back', () => {
  const rows = [{ id: 's-1', name: 'Example Librarian' }];
  assert.deepEqual(accountChrome('staff.Me', rows, 'Account'), { name: 'Example Librarian', initial: 'E', sample: false });
  assert.deepEqual(accountChrome('draft.Me', [{ id: 's-1', name: 'sample person' }], 'Account'), { name: 'sample person', initial: 'S', sample: true });
  assert.deepEqual(accountChrome('draft.Me', undefined, 'Account'), { name: 'Account', initial: 'A', sample: false }, 'no rows: nothing made up is shown');
  assert.deepEqual(accountChrome(undefined, undefined, 'Signed in'), { name: 'Signed in', initial: 'S', sample: false });
});
