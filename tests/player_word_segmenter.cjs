const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const { gunzipSync, gzipSync } = require('node:zlib');

const api = import('../src/ui/lyrics/word-segmenter.mjs');
const dictionary = api.then(({ parseDictionary }) => parseDictionary(gunzipSync(readFileSync(`${__dirname}/../src/ui/lyrics/cjdict.tsv.gz`)).toString()));
const jieba = api.then(({ parseJiebaDictionary }) => parseJiebaDictionary(gunzipSync(readFileSync(`${__dirname}/../src/ui/lyrics/jieba-words.txt.gz`)).toString()));
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
  assert.equal(calls, 2);
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
    assert.equal(calls, 2);
  }
});

test('Jieba DAG uses frequencies, total normalization, unknown defaults and longer ties without HMM', async () => {
  const { parseJiebaDictionary, cutJieba, createWordSegmenter } = await api;
  const dict = await parseJiebaDictionary('甲\t10\n乙\t10\n甲乙\t1\n');
  assert.equal(dict.total, 21);
  assert.deepEqual(cutJieba('甲乙', dict), ['甲', '乙']);
  assert.deepEqual(cutJieba('甲乙', { ...dict, total: 1000 }), ['甲乙']);
  assert.deepEqual(cutJieba('龘靐', dict), ['龘', '靐']);
  assert.deepEqual(cutJieba('', dict), []);
  const tie = await parseJiebaDictionary('甲\t1\n乙\t1\n甲乙\t1\n');
  assert.deepEqual(cutJieba('甲乙', { ...tie, total: 1 }), ['甲乙']);
  const noUnknownEdge = await parseJiebaDictionary('甲乙\t1\n乙丙\t999\n');
  assert.deepEqual(cutJieba('甲乙丙', noUnknownEdge), ['甲乙', '丙']);
  for (const text of ['', 'broken', '甲\tNaN', '甲\t-1', '甲\t1\textra', '甲\t0', '甲\t9007199254740992']) await assert.rejects(parseJiebaDictionary(text));
  const segmenter = createWordSegmenter(new Map(), null, dict);
  assert.deepEqual([...segmenter.segment('')], []);
  assert.equal(words(segmenter.segment('😀甲乙')), '😀|甲|乙');
});

test('real Jieba frequencies refine only singleton Han runs, with LEANS_BACK', async () => {
  const { createWordSegmenter } = await api;
  const frequencies = await jieba;
  assert.equal(frequencies.frequencies.size, 352825);
  assert.equal(frequencies.total, 60925619);
  assert.equal(frequencies.frequencies.get('杯壁'), 1);
  const before = createWordSegmenter(await dictionary, native);
  const after = createWordSegmenter(await dictionary, native, frequencies);
  const samples = [
    ['伤人的话', '伤人|的话'],
    ['烟雨朦胧', '烟雨|朦胧'],
    ['爱情难说', '爱情|难说'],
    ['情话飘散成千里之外', '情话|飘散|成|千里之外'],
    ['忘記思考生命其他的路徑', '忘記|思考|生命|其他|的|路徑'],
    ['忘记思考生命其他的路径', '忘记|思考|生命|其他|的|路径'],
    ['竟然發覺我的視線已經漸漸不清晰', '竟然|發覺|我的|視線|已經|漸漸|不|清晰'],
    ['私は東京に住んでいます', '私|は|東京|に|住|んで|い|ます'],
    ['冷凝水在杯壁画出了迷宫', '冷凝|水|在|杯|壁画|出了|迷宫'],
    ['蒸发成辞职信里两行鞠躬', '蒸发|成|辞职|信里|两行|鞠躬'],
    ['眼前的未来也并没有想通', '眼前|的|未来|也|并|没有|想通'],
    ['主动掉进陷阱还是往你脸上贴金', '主动|掉进|陷阱|还是|往|你|脸上|贴金'],
    ['也曾想过以后生活没有人牵挂', '也|曾|想过|以后|生活|没有|人|牵挂'],
    ['偷偷想着我们之间关系的远近', '偷偷|想着|我们之间|关系|的|远近'],
    ['在浪漫还有现实之间作斗争', '在|浪漫|还有|现实之间|作|斗争'],
    ['Love你一生一世', 'Love|你|一生一世'],
    ['晴天 Jay Chou，2003', '晴天| |Jay| |Chou|，|2003'],
    ['研究生命的起源', '研究|生命|的|起源'],
    ['我的心脏扑通到不再普通', '我的|心脏|扑通|到|不再|普通'],
    ['为你照亮角落的光该如何形容', '为|你|照亮|角落|的|光|该|如何|形容'],
  ];
  for (const [text, expected] of samples) {
    const items = [...after.segment(text)];
    assert.equal(words(items), expected, text);
    assert.equal(items.map(item => item.segment).join(''), text);
    for (const item of items) assert.equal(text.slice(item.index, item.index + item.segment.length), item.segment);
    console.log(`${text}: ${words(before.segment(text))} => ${words(items)}`);
  }
  for (const text of ['𠀀伤人的话！', '伤人的话かな', '伤人的话한글', '伤人的话ㄅ', '伤人的话ｶ', 'Hello world!', '']) {
    const items = [...after.segment(text)];
    assert.equal(items.map(item => item.segment).join(''), text);
    for (const item of items) assert.equal(text.slice(item.index, item.index + item.segment.length), item.segment);
    if (/[かな한글ｶ]/u.test(text) || !/\p{Script=Han}/u.test(text)) assert.deepEqual(items, [...before.segment(text)]);
  }
  for (const suffix of ['之外', '之内', '之中', '之间', '之后', '之前', '之上', '之下', '之一', '之余', '之际', '之初', '之时', '之久', '之多', '之类', '之处']) {
    const base = new Map([['千里', 1], [suffix, 1]]);
    assert.equal(words(createWordSegmenter(base, native, frequencies).segment(`千里${suffix}`)), `千里${suffix}`);
    assert.equal(words(createWordSegmenter(base, native, frequencies).segment(`千里 ${suffix}`)), `千里| |${suffix}`);
  }
});

test('optional Jieba failure still installs cjdict, while both fetches start in parallel', async () => {
  const { createDictionaryLoader } = await api;
  for (const optional of [
    async () => { throw new Error('offline'); },
    async () => new Response('', { status: 404 }),
    async () => new Response('not gzip'),
    async () => new Response(gzipSync('broken')),
    async () => new Response(gzipSync('')),
  ]) {
    const calls = [];
    let release;
    const blocked = new Promise(resolve => { release = resolve; });
    const loader = createDictionaryLoader({ Segmenter: StubSegmenter, fetcher: async url => {
      calls.push(url.pathname);
      if (url.pathname.endsWith('jieba-words.txt.gz')) { await blocked; return optional(); }
      return new Response(gzipSync('生命\t1\n'));
    } });
    let installed;
    let ready = 0;
    const loading = loader(value => { installed = value; }, () => ready++);
    assert.equal(calls.length, 2);
    release();
    assert.equal(await loading, true);
    assert.equal(words(installed.segment('生命')), '生命');
    assert.equal(ready, 1);
  }
  const loader = createDictionaryLoader({ Segmenter: StubSegmenter, fetcher: async url => new Response(gzipSync(url.pathname.endsWith('jieba-words.txt.gz') ? '伤人\t1\n' : '的\t1\n')) });
  let installed;
  assert.equal(await loader(value => { installed = value; }, () => {}), true);
  assert.equal(words(installed.segment('伤人')), '伤人');
});

test('vendored AMLL balances the reported song at phone-like widths', async () => {
  const { createWordSegmenter } = await api;
  const source = readFileSync(`${__dirname}/../src/ui/lyrics/amll-core.mjs`, 'utf8');
  const constants = source.match(/(\w+=1e3,\w+=\.15,\w+=\.35,\w+=\.5,\w+=\.4,\w+=\.6,\w+=\/.*?\/);function (\w+)\(/u);
  assert(constants, 'locate the actual vendored balancer, including its penalty constants');
  const start = source.indexOf(`function ${constants[2]}(`);
  let end = source.indexOf('{', start);
  let depth = 0;
  do {
    if (source[end] === '{') depth++;
    if (source[end] === '}') depth--;
    end++;
  } while (depth);
  const cjk = source.match(/\bst=(.*?\.test\(s\))/u);
  assert(cjk, 'locate the vendored isCJK predicate');
  const calcBalancedBreaks = require('node:vm').runInNewContext(`const st=${cjk[1]}; const ${constants[1]}; ${source.slice(start, end)}; ${constants[2]}`);
  const before = createWordSegmenter(await dictionary, native);
  const after = createWordSegmenter(await dictionary, native, await jieba);
  const cases = [
    ['我的心脏扑通到不再普通', ['我的心脏扑 / 通到不再普通', '我的心脏扑 / 通到不再普通', '我的心脏扑 / 通到不再普通'], ['我的心脏扑通 / 到不再普通', '我的心脏 / 扑通到不再普通', '我的心脏 / 扑通到不再普通']],
    ['为你照亮角落的光该如何形容', ['为你照亮 / 角落的光 / 该如何形容', '为你照亮角落 / 的光该如何形容', '为你照亮角落 / 的光该如何形容'], ['为你照亮 / 角落的光 / 该如何形容', '为你照亮角落 / 的光该如何形容', '为你照亮角落 / 的光该如何形容']],
  ];
  for (const [text, oldRows, newRows] of cases) {
    const children = [...text].map(text => ({ text, width: 1, isSpace: false }));
    for (const width of [6, 7, 8]) {
      const rows = segmenter => {
        const breaks = [0, ...calcBalancedBreaks(children, width, text, segmenter), children.length];
        return breaks.slice(1).map((end, i) => children.slice(breaks[i], end).map(item => item.text).join('')).join(' / ');
      };
      assert.equal(rows(before), oldRows[width - 6]);
      assert.equal(rows(after), newRows[width - 6]);
      console.log(`${text}, width=${width}: ${rows(before)} => ${rows(after)}`);
    }
  }
});
