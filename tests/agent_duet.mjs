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

const i18nSource = readFileSync(new URL('../src/ui/i18n.js', import.meta.url), 'utf8');
function loadI18n(language, saved) {
  const context = {
    navigator: { language },
    localStorage: { getItem: () => saved, setItem() {} },
    document: { documentElement: {}, addEventListener() {}, querySelectorAll: () => [] },
    addEventListener() {},
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
test('language cycling preserves saved Simplified Chinese preferences', () => {
  const { AmI18n } = loadI18n('zh-TW', 'zh');
  assert.equal(AmI18n.lang, 'zh');
  assert.equal(AmI18n.locale, 'zh-CN');
  for (const language of ['zh-Hant', 'en', 'zh']) {
    AmI18n.toggle();
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
  assert.equal(context.AmI18n.lang, 'zh');
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
