import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';

const panel = readFileSync(new URL('../src/ui/lyrics/panel.mjs', import.meta.url), 'utf8');
const prefsCode = panel.slice(panel.indexOf('const DEFAULT_PREFS'), panel.indexOf('/** AMLL TTML DB'));
function preferences(raw, unavailable = false) {
  const storage = {
    getItem() { if (unavailable) throw new Error('storage unavailable'); return raw; },
    setItem(key, value) { if (unavailable) throw new Error('storage unavailable'); raw = value; },
  };
  const context = vm.createContext({ localStorage: storage, PREFS_KEY: 'lyrics', FONT_SCALES: [1], FONT_WEIGHTS: [600], BACKDROPS: ['amll'] });
  vm.runInContext(prefsCode, context);
  return context;
}

test('autospace defaults off for empty, invalid and unavailable preferences', () => {
  for (const raw of [null, '{}', 'null', 'invalid', '{"textAutospace":"true"}']) {
    assert.equal(vm.runInContext('loadPrefs().textAutospace', preferences(raw)), false);
  }
  assert.equal(vm.runInContext('loadPrefs().textAutospace', preferences(null, true)), false);
});

test('autospace restores and saves both boolean values', () => {
  for (const value of [true, false]) {
    const context = preferences(JSON.stringify({ textAutospace: value }));
    assert.equal(vm.runInContext('loadPrefs().textAutospace', context), value);
    vm.runInContext('const prefs = loadPrefs(); prefs.textAutospace = !prefs.textAutospace; savePrefs(prefs)', context);
    assert.equal(vm.runInContext('loadPrefs().textAutospace', context), !value);
  }
});

test('autospace action applies immediately and survives unavailable storage', () => {
  const action = panel.slice(panel.indexOf("} else if (action === 'textAutospace') {") + "} else if (action === 'textAutospace') {".length, panel.indexOf("} else if (action === 'pulse')"));
  for (const unavailable of [false, true]) {
    const context = preferences('{}', unavailable);
    const calls = [];
    Object.assign(context, { prefs: { textAutospace: false }, closeMenu() {}, view: { setEnableTextAutospace(value) { calls.push(value); } } });
    vm.runInContext(action, context);
    vm.runInContext(action, context);
    assert.deepEqual(calls, [true, false]);
  }
  assert.match(panel, /const view = new DomLyricPlayer\(\);\s+view\.setEnableTextAutospace\(prefs\.textAutospace\)/);
});

test('hover patch accepts changing CSS module hashes but rejects mismatched selectors', () => {
  const build = readFileSync(new URL('../scripts/build-amll.cjs', import.meta.url), 'utf8');
  const expression = build.match(/const hoverUnblur = (.+);/)[1];
  const regex = vm.runInNewContext(expression);
  const rule = (a, b = a) => `  &:hover .${a}_lyricLine, &:hover .${b}_lyricLineWrapper {\n    filter: unset !important;\n  }\n`;
  for (const prefix of ['_63lo6G', 'kjAh-W', 'future_hash-123']) assert.equal(rule(prefix).match(regex)?.[1], prefix);
  assert.equal(regex.test(rule('one', 'two')), false);
  assert.equal(regex.test(''), false);
});
