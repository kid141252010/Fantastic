// 歌词解析（对唱侧、本地化选择与时间轴推断）：离线，只需 Node：
//   node --test tests/agent_duet.mjs
// 按 parseTTML 的输出手工构造歌曲（只给 duetSides 用到的 agents 与每行的 agent），检查 src/ui/lyrics/ttml.mjs 的 duetSides
// 照 AMLL（packages/ttml 的 amll-converter）逐行决定的显示侧：第一位主唱在主侧（type 为 other 时在对唱侧），同一位主唱不换侧，
// 换人就换侧；合唱（type 为 group，如 v1000）不在对唱侧，也不打断交替。和声跟随所属主歌词行由 tests/lyrics_ui.cjs 在页面里检查。
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { duetSides, parseTTML } from '../src/ui/lyrics/ttml.mjs';
import { Node as UiNode } from './helpers/ui-dom.mjs';

const i18nSource = readFileSync(new URL('../src/ui/i18n.js', import.meta.url), 'utf8');
function loadI18n(language, saved) {
  const writes = [];
  const events = {};
  const context = {
    writes,
    events,
    navigator: { language },
    localStorage: { getItem: () => saved, setItem: (...args) => writes.push(args) },
    document: { documentElement: {}, addEventListener() {}, querySelectorAll: () => [] },
    addEventListener(type, handler) { events[type] = handler; },
  };
  runInNewContext(i18nSource, context);
  return context;
}
for (const language of ['zh-Hant', 'zh-TW', 'zh-HK', 'zh-MO', 'ZH_tw']) {
  test(`Traditional Chinese browser locale: ${language}`, () => {
    const { AmI18n, document } = loadI18n(language);
    assert.equal(AmI18n.lang, 'zh-Hant');
    assert.equal(AmI18n.locale, 'zh-TW');
    AmI18n.apply();
    assert.equal(document.documentElement.lang, 'zh-TW');
    assert.equal(AmI18n.t('lang.button'), '繁體中文');
  });
}
test('direct language selection preserves saved Simplified Chinese preferences', () => {
  const { AmI18n, writes } = loadI18n('zh-TW', 'zh');
  assert.equal(AmI18n.lang, 'zh-Hans');
  assert.deepEqual(writes, [['am-hook:lang', 'zh-Hans']]);
  assert.equal(AmI18n.locale, 'zh-CN');
  for (const language of ['zh-Hant', 'en', 'zh-Hans']) {
    AmI18n.setLang(language);
    assert.equal(AmI18n.lang, language);
  }
});
test('empty browser language and invalid saved language fall back to English', () => {
  const { AmI18n } = loadI18n('', 'invalid');
  assert.equal(AmI18n.lang, 'en');
  assert.equal(AmI18n.t('unknown.key'), 'unknown.key');
  AmI18n.setLang('invalid');
  assert.equal(AmI18n.lang, 'en');
});
test('unavailable localStorage does not prevent language detection', () => {
  const context = loadI18n('zh-CN');
  context.localStorage.getItem = () => { throw new Error('blocked'); };
  runInNewContext(i18nSource, context);
  assert.equal(context.AmI18n.lang, 'zh-Hans');
});
test('legacy storage events migrate to the Simplified Chinese key', () => {
  const { AmI18n, events, writes } = loadI18n('en');
  events.storage({ key: 'am-hook:lang', newValue: 'zh' });
  assert.equal(AmI18n.lang, 'zh-Hans');
  assert.equal(writes.at(-1)[1], 'zh-Hans');
});
test('language picker selects each language without waiting for catalog data', async () => {
  class PickerNode extends UiNode {
    focus() {}
    scrollIntoView() {}
    closest() { return this.dataset.picker ? this : null; }
    getBoundingClientRect() { return { width: 300, left: 10, top: 100, bottom: 140 }; }
    querySelectorAll(selector) {
      const [base, attribute, value] = selector.match(/^([^[]*)\[([^=]+)="([^"]+)"\]$/)?.slice(1) || [];
      if (!attribute) return super.querySelectorAll(selector);
      const nodes = super.querySelectorAll(base || 'span');
      return nodes.filter(node => node.getAttribute(attribute) === value);
    }
  }
  const context = loadI18n('zh-CN');
  const root = new PickerNode();
  const picker = new PickerNode();
  for (const name of ['title', 'hint', 'body', 'close']) {
    const node = new PickerNode();
    node.className = `picker-${name}`;
    picker.append(node);
  }
  const anchor = new PickerNode('button');
  anchor.dataset.picker = 'uiLang';
  anchor.isConnected = true;
  const value = new PickerNode('span');
  value.setAttribute('data-setting-value', 'uiLang');
  root.append(picker, value);
  const clicks = {};
  context.document = {
    documentElement: {}, createElement: tag => new PickerNode(tag),
    querySelectorAll: selector => root.querySelectorAll(selector),
    addEventListener: (type, fn) => { clicks[type] = fn; },
  };
  Object.assign(context, {
    window: context, Element: PickerNode, innerWidth: 1200, innerHeight: 800,
    matchMedia: () => ({ matches: false }),
    AmWrapper: { settings: { url: 'http://localhost' } },
  });
  context.AmI18n.storefronts = () => Promise.resolve(null);
  context.AmI18n.catalogLang = () => Promise.resolve(undefined);
  runInNewContext(readFileSync(new URL('../src/ui/settings.mjs', import.meta.url), 'utf8')
    .replace('export function mountSettings', 'function mountSettings'), context);
  context.mountSettings({ picker, scrim: new PickerNode(), checkStatus() {} });
  for (const [code, label] of [['zh-Hant', '繁體中文'], ['en', 'English'], ['zh-Hans', '简体中文']]) {
    clicks.click({ target: anchor });
    const options = picker.querySelectorAll('.picker-option');
    assert.deepEqual(options.map(node => node.textContent), ['简体中文', '繁體中文', 'English']);
    assert.equal(options.filter(node => node.getAttribute('aria-checked') === 'true').length, 1);
    await options.find(node => node.textContent === label).click();
    assert.equal(context.AmI18n.lang, code);
    assert.equal(value.textContent, label);
    assert.equal(picker.hidden, true);
    assert.equal(anchor.getAttribute('aria-expanded'), 'false');
  }
});

/** 按 parseTTML 的输出构造歌曲：singers 为每行的 ttm:agent；不给 agents 时整个省略 */
const song = (singers, agents) => ({ ...(agents ? { agents } : {}), lines: singers.map((agent) => ({ agent })) });

/** parseTTML 的 agents：xml:id → { id, type } */
const declare = (types) => Object.fromEntries(Object.entries(types).map(([id, type]) => [id, { id, type }]));
const PEOPLE = declare({ v1: 'person', v2: 'person' });
const CHORUS = declare({ v1: 'person', v2: 'person', v1000: 'group' });

test('只有一位主唱时全部在主侧', () => {
  assert.deepEqual(duetSides(song(['v1', 'v1', 'v1'], declare({ v1: 'person' }))), [false, false, false]);
});

test('两位主唱轮流唱时左右交替', () => {
  assert.deepEqual(duetSides(song(['v1', 'v2', 'v1', 'v2'], PEOPLE)), [false, true, false, true]);
});

test('第一位主唱在主侧，即使它是 v2', () => {
  assert.deepEqual(duetSides(song(['v2', 'v1'], PEOPLE)), [false, true]);
});

test('第一位主唱的 type 为 other 时从对唱侧开始，之后照常交替', () => {
  assert.deepEqual(duetSides(song(['v1', 'v1', 'v2', 'v1'], declare({ v1: 'other', v2: 'person' }))), [true, true, false, true]);
  // other 只决定第一位主唱的显示侧，后来才出现的 other 照常换侧
  assert.deepEqual(duetSides(song(['v2', 'v1', 'v3'], declare({ v1: 'person', v2: 'person', v3: 'other' }))), [false, true, false]);
});

test('合唱（type 为 group）不在对唱侧，也不打断主唱的交替', () => {
  assert.deepEqual(duetSides(song(['v1', 'v1000', 'v2', 'v1000', 'v1'], CHORUS)), [false, false, true, false, false]);
});

test('以合唱开头时，之后的第一位主唱仍在主侧', () => {
  assert.deepEqual(duetSides(song(['v1000', 'v2', 'v1'], CHORUS)), [false, false, true]);
});

test('agents 里没有声明的歌手中途出现时，和普通主唱一样换侧', () => {
  assert.deepEqual(duetSides(song(['v1', 'v2', 'v3', 'v1'], PEOPLE)), [false, true, false, true]);
});

test('同一位主唱连续几行不换侧，中间隔着合唱也一样', () => {
  assert.deepEqual(duetSides(song(['v2', 'v2', 'v1', 'v1', 'v1000', 'v1', 'v2'], CHORUS)), [false, false, true, true, false, true, false]);
});

test('歌曲没有 agents 时不报错，仍按 ttm:agent 的 id 交替', () => {
  assert.deepEqual(duetSides(song(['v2', 'v1', 'v1', 'v2'])), [false, true, true, false]);
});

// 离线 DOM：Node 没有内置 DOMParser，用最小的 TTML 树覆盖 parseTTML。
const XML = 'http://www.w3.org/XML/1998/namespace';
const APPLE = 'http://music.apple.com/lyric-ttml-internal';
const META = 'http://www.w3.org/ns/ttml#metadata';
class TextNode {
  constructor(text) { this.nodeType = 3; this.textContent = text; }
}
class Element {
  constructor(localName, attributes = {}, children = []) {
    this.localName = localName;
    this.namespaceURI = '';
    this.attributes = attributes;
    this.children = children;
    this.childNodes = children;
    for (let i = 0; i < children.length; i++) children[i].nextSibling = children[i + 1];
  }
  getAttribute(name) { return this.attributes[name] || ''; }
  getAttributeNS(namespace, name) {
    const prefix = namespace === XML ? 'xml' : namespace === APPLE ? 'itunes' : namespace === META ? 'ttm' : '';
    return prefix ? this.attributes[`${prefix}:${name}`] || '' : '';
  }
  get textContent() { return this.childNodes.map((node) => node.textContent).join(''); }
}
const text = (value) => new TextNode(value);
const element = (name, attributes, children) => new Element(name, attributes, children);
function fixture(translationLanguages, pronunciationLanguages) {
  const translations = translationLanguages.map((language) => element('translation', {
    'xml:lang': language, type: 'subtitle', automaticallyCreated: 'false',
  }, [element('text', { for: 'L1' }, [text(`translation ${language}`)])]));
  const transliterations = pronunciationLanguages.map((language) => element('transliteration', {
    'xml:lang': language,
  }, [element('text', { for: 'L1' }, [text(`pronunciation ${language}`)])]));
  const metadata = element('iTunesMetadata', {}, [
    element('translations', {}, translations), element('transliterations', {}, transliterations),
  ]);
  const head = element('head', {}, [element('metadata', {}, [metadata])]);
  const line = element('p', { 'itunes:key': 'L1' }, [
    element('span', { begin: '00:01.000', end: '00:02.000' }, [text('source')]),
  ]);
  const body = element('body', { dur: '00:02.000' }, [element('div', {
    begin: '00:01.000', end: '00:02.000',
  }, [line])]);
  return { documentElement: element('tt', { 'xml:lang': 'en' }, [head, body]), querySelector: () => null };
}
class FixtureParser {
  parseFromString(xml) {
    const { translation, pronunciation } = JSON.parse(xml);
    return fixture(translation, pronunciation);
  }
}
const panelSource = readFileSync(new URL('../src/ui/lyrics/panel.mjs', import.meta.url), 'utf8');
function languageChangePanel({ visible = true, empty = false } = {}) {
  const raw = JSON.stringify({ translation: ['zh-Hans', 'zh-Hant', 'en'], pronunciation: [] });
  const calls = [];
  const context = {
    AmI18n: { lang: 'zh-Hans' },
    song: empty ? null : parseTTML(raw, FixtureParser, 'zh-Hans'), lyricsText: empty ? '' : raw,
    has: {}, lyricsVisible: visible, open: true, menu: { hidden: true },
    parseTTML: (xml, _, prefLang) => parseTTML(xml, FixtureParser, prefLang),
    fetch() { throw new Error('language changes must not request lyrics'); },
    currentTime: () => 4250,
    view: {
      getElement: () => ({}), setCurrentTime: (...args) => calls.push(['time', ...args]),
      resetScroll: () => calls.push(['reset']), setLyricLines: lines => calls.push(['lines', lines.length]),
    },
    setLines: () => calls.push(['translation', context.song.lines[0].translation]),
    syncOptions() {}, renderCredits() {}, renderMenu() {}, renderHeader() {}, syncToggle() {},
    onLangChange: handler => { context.change = handler; },
  };
  const parseFunction = panelSource.match(/  const parseWithUiLanguage = \(text\) => \{[\s\S]*?\n  \};/)[0];
  const renderFunction = panelSource.match(/  function renderSong\(\) \{[\s\S]*?\n  \}/)[0];
  const handler = panelSource.match(/  onLangChange\(\(\) => \{[\s\S]*?\n  \}\);/)[0];
  runInNewContext(`${parseFunction}\n${renderFunction}\n${handler}`, context);
  return { context, calls };
}
test('changing UI language re-parses cached lyrics and aligns the visible view to playback', () => {
  const { context, calls } = languageChangePanel();
  for (const language of ['zh-Hant', 'en', 'zh-Hans']) {
    context.AmI18n.lang = language;
    context.change();
    assert.equal(context.song.translation.language, language);
    assert.deepEqual(calls.splice(0), [['translation', `translation ${language}`], ['time', 4250, true], ['reset']]);
    assert.equal(context.has.translation, true);
  }
});
test('changing UI language clears hidden lyric lines for layout when reopened', () => {
  const { context, calls } = languageChangePanel({ visible: false });
  context.AmI18n.lang = 'zh-Hant';
  context.change();
  assert.equal(context.song.translation.language, 'zh-Hant');
  assert.deepEqual(calls, [['lines', 0]]);
});
test('changing UI language before lyrics are loaded does not parse or request lyrics', () => {
  const { context, calls } = languageChangePanel({ empty: true });
  context.change();
  assert.equal(context.song, null);
  assert.deepEqual(calls, []);
});
test('a failed cached lyric parse does not replace the previous song', () => {
  const { context } = languageChangePanel();
  const previous = context.song;
  context.lyricsText = 'invalid';
  assert.throws(() => context.change(), /Unexpected token/);
  assert.equal(context.song, previous);
});
const parse = (translation, pronunciation = translation, prefLang) => parseTTML(
  JSON.stringify({ translation, pronunciation }), FixtureParser, prefLang,
);
const localizationCases = [
  ['zh-Hant', ['zh-Hans', 'zh-Hant'], 'zh-Hant'],
  ['zh-Hant', ['zh-Hans-CN', 'zh-Hant-HK'], 'zh-Hant-HK'],
  ['zh-Hant', ['zh-CN', 'zh-TW'], 'zh-TW'],
  ['zh-Hans', ['zh-Hant', 'zh-Hans'], 'zh-Hans'],
  ['zh', ['zh-Hant', 'zh-Hans'], 'zh-Hans'],
  ['zh', ['en', 'zh-Hans-CN'], 'zh-Hans-CN'],
  ['zh', ['zh-CN', 'zh-Hant'], 'zh-CN'],
  ['zh', ['zh-TW', 'zh-CN'], 'zh-CN'],
  ['zh', ['zh-CN', 'zh-Hans'], 'zh-Hans'],
  ['zh', ['en-US'], 'en-US'],
  ['en', ['zh-Hans', 'en-US'], 'en-US'],
  ['en', ['en-GB', 'en-US'], 'en-US'],
  ['en', ['en-GB', 'en'], 'en'],
];
for (const [prefLang, languages, expected] of localizationCases) {
  test(`pref=${prefLang} selects ${expected}`, () => {
    const parsed = parse(languages, languages, prefLang);
    assert.equal(parsed.translation.language, expected);
    assert.equal(parsed.pronunciation.language, expected);
    assert.equal(parsed.lines[0].translation, `translation ${expected}`);
    assert.equal(parsed.lines[0].pronunciation, `pronunciation ${expected}`);
  });
}
test('language selection keeps document order when no preferred language matches', () => {
  const parsed = parse(['en-US', 'ja-JP'], ['en-US', 'ja-JP'], 'zh');
  assert.equal(parsed.translation.language, 'en-US');
  assert.equal(parsed.pronunciation.language, 'en-US');
});
test('omitting prefLang keeps the first localization', () => {
  const parsed = parse(['zh-Hant', 'zh-Hans'], ['zh-Hant', 'zh-Hans']);
  assert.equal(parsed.translation.language, 'zh-Hant');
  assert.equal(parsed.pronunciation.language, 'zh-Hant');
  assert.equal(parsed.lines[0].translation, 'translation zh-Hant');
  assert.equal(parsed.lines[0].pronunciation, 'pronunciation zh-Hant');
});
test('language matching ignores case and treats underscores as hyphens', () => {
  const parsed = parse(['zh_hant', 'ZH_hans_CN'], ['zh_hant', 'ZH_hans_CN'], 'ZH');
  assert.equal(parsed.translation.language, 'ZH_hans_CN');
  assert.equal(parsed.pronunciation.language, 'ZH_hans_CN');
});

const parseDocument = (document, prefLang) => parseTTML('', class {
  parseFromString() { return document; }
}, prefLang);
for (const prefLang of ['zh-Hant', 'zh-Hans', 'en', undefined]) {
  test(`replacement respects Chinese script preference: ${prefLang}`, () => {
    const document = fixture(['zh-Hans-CN'], []);
    document.documentElement.children[1].children[0].children[0].children[0].nodeType = 1;
    const translation = document.documentElement.children[0].children[0].children[0].children[0].children[0];
    translation.attributes.type = 'replacement';
    const parsed = parseDocument(document, prefLang);
    assert.equal(parsed.lines[0].text, prefLang === 'zh-Hant' ? 'source' : 'translation zh-Hans-CN');
    assert.equal(parsed.lines[0].translation, '');
  });
}
test('replacement decision uses the selected localization', () => {
  const document = fixture(['zh-Hans-CN', 'zh-Hant-TW'], []);
  const translations = document.documentElement.children[0].children[0].children[0].children[0];
  for (const translation of translations.children) translation.attributes.type = 'replacement';
  assert.equal(parseDocument(document, 'zh-Hant').lines[0].text, 'translation zh-Hant-TW');
});
for (const [prefLang, text, translation] of [
  ['zh-Hant', 'translation zh-Hant-TW', 'translation zh-Hant'], ['zh-Hans', 'source', 'translation zh-Hans'],
]) {
  test(`replacement and subtitle translations are selected separately: ${prefLang}`, () => {
    const document = fixture(['zh-Hant-TW', 'zh-Hans', 'zh-Hant'], []);
    document.documentElement.children[1].children[0].children[0].children[0].nodeType = 1;
    document.documentElement.children[0].children[0].children[0].children[0].children[0].attributes.type = 'replacement';
    const parsed = parseDocument(document, prefLang);
    assert.equal(parsed.lines[0].text, text);
    assert.equal(parsed.lines[0].translation, translation);
  });
}
for (const [language, prefLang, skip] of [
  ['zh-CN', 'zh-Hant', true], ['zh-SG', 'zh-Hant', true],
  ['zh-TW', 'zh-Hans', true], ['zh-HK', 'zh-Hans', true], ['zh-MO', 'zh-Hans', true],
  ['zh-CN', 'zh-Hans', false], ['zh-TW', 'zh-Hant', false], ['zh-CN', 'en', false],
  ['zh-Hant-CN', 'zh-Hant', false], ['en-CN', 'zh-Hant', false],
]) {
  test(`replacement infers Chinese script from region: ${language}, ${prefLang}`, () => {
    const document = fixture([language], []);
    document.documentElement.children[1].children[0].children[0].children[0].nodeType = 1;
    const translation = document.documentElement.children[0].children[0].children[0].children[0].children[0];
    translation.attributes.type = 'replacement';
    const parsed = parseDocument(document, prefLang);
    assert.equal(parsed.lines[0].text, skip ? 'source' : `translation ${language}`);
    assert.equal(parsed.lines[0].translation, '');
  });
}
test('empty localization collections keep the original lyrics', () => {
  const parsed = parse([], [], 'zh-Hant');
  assert.equal(parsed.lines[0].tokens[0].text, 'source');
  assert.equal(parsed.lines[0].translation, '');
});
test('invalid TTML still fails for a Traditional Chinese preference', () => {
  assert.throws(() => parseDocument({ querySelector: () => true }, 'zh-Hant'), /无法解析 TTML/);
});
function parseLines(lines, { prefLang, timing } = {}) {
  return parseDocument({
    documentElement: element('tt', timing === undefined ? {} : { 'itunes:timing': timing }, [
      element('body', {}, [element('div', {}, lines)]),
    ]),
    querySelector: () => null,
  }, prefLang);
}
const inlineSpans = (role, languages) => languages.map((language, index) => element('span', {
  'ttm:role': role, ...(language ? { 'xml:lang': language } : {}),
}, [text(`${role} ${index}`)]));

const inlineCases = [
  ['pref=zh prefers CN', 'zh', ['de-DE', 'zh-TW', 'zh-CN'], 2],
  ['pref=en prefers US', 'en', ['de-DE', 'en-GB', 'en-US'], 2],
  ['exact language wins', 'zh', ['zh-CN', 'zh-Hans', 'zh'], 2],
  ['script outranks region', 'zh', ['zh-CN', 'zh-Hans'], 1],
  ['same primary language keeps document order', 'en', ['de-DE', 'en-GB', 'en-AU'], 1],
  ['no matching language keeps the first span', 'en', ['de-DE', 'zh-CN'], 0],
  ['no preference keeps the first span', undefined, ['de-DE', 'zh-TW', 'zh-CN'], 0],
  ['unlabelled spans keep the first span', 'zh', [undefined, undefined], 0],
  ['case and underscores are normalized', 'ZH', ['zh-TW', 'ZH_cn'], 1],
];
for (const [name, prefLang, languages, expected] of inlineCases) {
  test(`inline translation and pronunciation: ${name}`, () => {
    const voice = () => [text('source'), ...inlineSpans('x-translation', languages), ...inlineSpans('x-roman', languages)];
    const parsed = parseLines([element('p', {}, [
      ...voice(), element('span', { 'ttm:role': 'x-bg' }, voice()),
    ])], { prefLang });
    for (const part of [parsed.lines[0], parsed.lines[0].background]) {
      assert.equal(part.translation, `x-translation ${expected}`);
      assert.equal(part.pronunciation, `x-roman ${expected}`);
    }
  });
}

test('metadata localization takes precedence over inline fallback', () => {
  const document = fixture(['zh-CN'], ['zh-CN']);
  const line = document.documentElement.children[1].children[0].children[0];
  line.children.push(...inlineSpans('x-translation', ['zh']), ...inlineSpans('x-roman', ['zh']));
  const parsed = parseDocument(document, 'zh');
  assert.equal(parsed.lines[0].translation, 'translation zh-CN');
  assert.equal(parsed.lines[0].pronunciation, 'pronunciation zh-CN');
});

const timedWord = () => element('span', { begin: '00:14.760', end: '00:15.530' }, [text('word')]);
test('missing timing infers Word when a later line has word tokens', () => {
  const parsed = parseLines([
    element('p', {}, [text('untimed')]),
    element('p', { begin: '00:14.760', end: '00:15.530' }, [timedWord()]),
  ]);
  assert.equal(parsed.timing, 'Word');
  assert.equal(parsed.lines[1].tokens[0].begin, 14760);
  assert.equal(parsed.lines[1].tokens[0].end, 15530);
});
test('missing timing infers Word from background word tokens', () => {
  const parsed = parseLines([element('p', {}, [
    text('source'), element('span', { 'ttm:role': 'x-bg' }, [timedWord()]),
  ])]);
  assert.equal(parsed.timing, 'Word');
});
for (const attributes of [
  { begin: '00:14.760', end: '00:15.530' },
  { begin: '00:14.760', end: '0' },
  { begin: '0', end: '00:15.530' },
]) {
  test(`missing timing infers Line from nonzero line times ${JSON.stringify(attributes)}`, () => {
    const parsed = parseLines([element('p', {}, [text('untimed')]), element('p', attributes, [text('source')])]);
    assert.equal(parsed.timing, 'Line');
  });
}
test('missing timing infers None when all line times are zero', () => {
  const parsed = parseLines([
    element('p', {}, [text('no time attributes')]),
    element('p', { begin: '0', end: '00:00.000' }, [text('zero times')]),
  ]);
  assert.equal(parsed.timing, 'None');
});
for (const timing of ['Line', 'None', 'Word']) {
  test(`explicit timing=${timing} takes precedence over word tokens`, () => {
    const parsed = parseLines([element('p', {}, [timedWord()])], { timing });
    assert.equal(parsed.timing, timing);
  });
}
