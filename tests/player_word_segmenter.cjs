const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { gunzipSync, gzipSync } = require('node:zlib');

const api = import('../src/ui/lyrics/word-segmenter.mjs');
const dictionary = api.then(({ parseDictionary }) => parseDictionary(gunzipSync(readFileSync(`${__dirname}/../src/ui/lyrics/cjdict.tsv.gz`)).toString()));
const native = new Intl.Segmenter('zh', { granularity: 'word' });
const words = items => [...items].map(item => item.segment).join('|');
class StubSegmenter {
  *segment(text) {
    let index = 0;
    for (const segment of text) { yield { segment, index, isWordLike: true }; index += segment.length; }
  }
}

test('ICU dictionary costs reproduce Chinese, Japanese and mixed lyric boundaries', async () => {
  const { createWordSegmenter } = await api;
  const segmenter = createWordSegmenter(await dictionary, native);
  for (const text of ['忘記思考生命其他的路徑', '忘记思考生命其他的路径', '私は東京に住んでいます', 'Love你一生一世', 'カタカナのテスト', 'ｶﾞｯﾂﾎﾟｰｽﾞ', '𠀀生命，Hello world!']) {
    const items = [...segmenter.segment(text)];
    assert.equal(words(items), words(native.segment(text)), text);
    assert.equal(items.map(item => item.segment).join(''), text);
    for (const item of items) assert.equal(text.slice(item.index, item.index + item.segment.length), item.segment);
    console.log(`${text} => ${words(items)}`);
  }
  assert.deepEqual([...segmenter.segment('')], []);
  assert.deepEqual([...createWordSegmenter(new Map()).segment('Hello 42!')], [
    { segment: 'Hello', index: 0, isWordLike: true }, { segment: ' ', index: 5, isWordLike: false },
    { segment: '42', index: 6, isWordLike: true }, { segment: '!', index: 8, isWordLike: false },
  ]);
  assert.equal(words(createWordSegmenter(new Map()).segment('龘靐')), '龘|靐');
});

test('deterministic dictionary-adjacent lyric sample compared with desktop ICU', async () => {
  const { createWordSegmenter } = await api;
  const dict = await dictionary;
  assert.equal(dict.size, 315964);
  const segmenter = createWordSegmenter(dict, native);
  const entries = [...dict.keys()];
  const lyrics = ['忘記', '思考', '生命', '其他', '的路徑', '我爱你', 'Love', '一生一世', '私は', '東京', 'に住んでいます', '，', ' '];
  let seed = 0x141252;
  const random = n => { seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0; return seed % n; };
  let matched = 0;
  const mismatches = [];
  for (let i = 0; i < 500; i++) {
    const text = lyrics[random(lyrics.length)] + entries[random(entries.length)] + lyrics[random(lyrics.length)];
    const actual = words(segmenter.segment(text));
    const expected = words(native.segment(text));
    if (actual === expected) matched++;
    else if (mismatches.length < 5) mismatches.push({ text, actual, expected });
  }
  console.log(`ICU comparison: ${matched}/500 (${matched / 5}%), Node ${process.version}, ICU ${process.versions.icu}`);
  console.log('Sample mismatches:', mismatches);
  assert(matched >= 490, 'at least 98% of dictionary-adjacent samples agree with desktop ICU');
});

test('capability detection, one background fetch, install and relayout', async () => {
  const { needsDictionary, createDictionaryLoader } = await api;
  assert.equal(needsDictionary(Intl.Segmenter), false);
  assert.equal(needsDictionary(StubSegmenter), true);
  assert.equal(needsDictionary(null), true);
  let calls = 0;
  let ready = 0;
  const installed = [];
  const loader = createDictionaryLoader({ Segmenter: StubSegmenter, fetcher: async () => {
    calls++;
    return new Response(gzipSync('生命\t1\n'));
  } });
  const install = value => installed.push(value);
  assert.deepEqual(await Promise.all([loader(install, () => ready++), loader(install, () => ready++)]), [true, true]);
  assert.equal(calls, 1);
  assert.equal(ready, 2);
  assert.equal(installed[0], installed[1]);
  assert.equal(words(installed[0].segment('生命')), '生命');
  const healthy = createDictionaryLoader({ Segmenter: Intl.Segmenter, fetcher: () => { throw new Error('must not fetch'); } });
  assert.equal(await healthy(install, () => ready++), false);
  assert.equal(ready, 2);
  const missing = createDictionaryLoader({ Segmenter: null, fetcher: async () => new Response(gzipSync('生命\t1\n')) });
  assert.equal(await missing(install, () => ready++), true);
  assert.equal(words(installed.at(-1).segment('Love生命')), 'Love|生命');
});

test('HTTP, network, decompression and malformed dictionaries silently retain native behavior', async () => {
  const { createDictionaryLoader, parseDictionary } = await api;
  for (const text of ['', 'broken', '生命\t999', '生命\tNaN']) await assert.rejects(parseDictionary(text));
  for (const fetcher of [
    async () => { throw new Error('offline'); },
    async () => new Response('', { status: 404 }),
    async () => new Response('not gzip'),
    async () => new Response(gzipSync('broken')),
  ]) {
    let calls = 0;
    const loader = createDictionaryLoader({ Segmenter: StubSegmenter, fetcher: (...args) => { calls++; return fetcher(...args); } });
    const unexpected = () => assert.fail('must retain native behavior');
    assert.equal(await loader(unexpected, unexpected), false);
    assert.equal(await loader(unexpected, unexpected), false);
    assert.equal(calls, 1);
  }
});
