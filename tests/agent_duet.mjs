// 歌词解析（对唱侧与本地化选择）：离线，只需 Node：
//   node --test tests/agent_duet.mjs
// 按 parseTTML 的输出手工构造歌曲（只给 duetSides 用到的 agents 与每行的 agent），检查 src/ui/lyrics/ttml.mjs 的 duetSides
// 照 AMLL（packages/ttml 的 amll-converter）逐行决定的显示侧：第一位主唱在主侧（type 为 other 时在对唱侧），同一位主唱不换侧，
// 换人就换侧；合唱（type 为 group，如 v1000）不在对唱侧，也不打断交替。和声跟随所属主歌词行由 tests/lyrics_ui.cjs 在页面里检查。
import test from 'node:test';
import assert from 'node:assert/strict';
import { duetSides, parseTTML } from '../src/ui/lyrics/ttml.mjs';

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

// localization() 的离线 DOM：Node 没有内置 DOMParser，用最小的 TTML 树覆盖 parseTTML 的语言选择。
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
  ['zh', ['zh-Hant', 'zh-Hans'], 'zh-Hans'],
  ['zh', ['en', 'zh-Hans-CN'], 'zh-Hans-CN'],
  ['zh', ['zh-CN', 'zh-Hant'], 'zh-CN'],
  ['zh', ['en-US'], 'en-US'],
  ['en', ['zh-Hans', 'en-US'], 'en-US'],
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
