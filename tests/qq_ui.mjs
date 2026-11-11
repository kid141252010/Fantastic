import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { document, ids } from './helpers/ui-dom.mjs';
import { mount } from '../src/ui/views/qq-song.mjs';

const cover = 'https://y.qq.com/music/photo_new/T002R{w}x{h}M000003DFRzD192KKD_5.jpg';
const fixedCover = 'https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg';
const defaultQualities = [
  { quality: 'HQ', available: true, sizeBytes: 1024 },
  { quality: 'SQ', available: false, sizeBytes: 2048 },
  { quality: 'RS', available: true, sizeBytes: 0 },
  { quality: 'QAI', available: 'true', sizeBytes: 8192 },
  { quality: 'DA', available: true, sizeBytes: 4096 },
];

function detailPage(t, { qualities = defaultQualities, failQualities = false, nativeModes = ['native'] } = {}) {
  const saved = Object.fromEntries(['document', 'window', 'MediaSource', 'matchMedia', 'fetch']
    .map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const controller = new AbortController();
  t.after(() => {
    controller.abort();
    for (const [key, descriptor] of Object.entries(saved)) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  });
  const root = ids('title', 'subtitle', 'meta', 'badges', 'art', 'radiosity', 'enqueue', 'play-best',
    'variant-list', 'quality-status', 'lyrics', 'lyrics-source', 'lyrics-actions', 'lyrics-preview',
    'lyrics-qrc', 'lyrics-trans', 'lyrics-roma', 'lyrics-lines', 'alert', 'mv-link', 'info', 'facts', 'intro');
  const played = [], queued = [];
  let paused = false, onPlayerChange;
  const player = { current: null, onChange: (fn) => { onPlayerChange = fn; },
    transport: () => ({ paused }), toggle: () => { paused = !paused; onPlayerChange(); },
    play: (item) => played.push(item), appendQueue: (entry) => queued.push(entry) };
  let rerender;
  globalThis.document = document;
  globalThis.matchMedia = () => ({ matches: false, addEventListener() {} });
  globalThis.window = { AmI18n: { t: (key) => key }, AmHook: { formatTime: String, detectQqModes: () => nativeModes } };
  globalThis.MediaSource = { isTypeSupported: () => true };
  globalThis.fetch = async (path) => ({ ok: !failQualities || !path.endsWith('/qualities'), status: 503,
    json: async () => path.endsWith('/qualities') ? { qualities }
      : path.endsWith('/lyrics') ? { candidates: [], rawQrc: true, rawTrans: true, rawRoma: true }
        : { id: 'test', name: 'Test', albumMid: '003DFRzD192KKD', cover } });
  mount({ root, url: new URL('https://am.test/qq-song/test'), signal: controller.signal,
    player, onLangChange: (fn) => { rerender = fn; }, toast() {} });
  return { root, played, queued, rerender: () => rerender(),
    setPlaying: (item) => { player.current = item; paused = false; onPlayerChange(); } };
}

test('QQ detail omits unavailable tiers while retaining runtime-playable and download rows after rerender', async (t) => {
  const { root, rerender } = detailPage(t);
  assert.equal(root.querySelector('#variant-list').children.length, 0, 'loading has no placeholder tiers');
  assert.equal(root.querySelector('#quality-status').textContent, 'qq.qualitiesLoading');
  await new Promise(setImmediate);
  for (let i = 0; i < 2; i++) {
    const list = root.querySelector('#variant-list');
    assert.deepEqual(list.children.map((group) => group.querySelector('.quality-group-title').textContent), ['q.atmos', 'MP3']);
    const rows = list.querySelectorAll('.variant');
    assert.deepEqual(rows.map((row) => row.dataset.quality), ['DA', 'HQ']);
    assert.ok(rows[0].querySelector('.variant-play'));
    assert.ok(rows[0].querySelector('.variant-dl'));
    assert.ok(rows[0].querySelector('.variant-more'));
    assert.equal(rows[0].querySelector('.variant-dl').href, '/qq/song/test/download?quality=DA');
    assert.ok(rows[1].querySelector('.variant-play'));
    assert.equal(rows[1].querySelector('.variant-dl').href, '/qq/song/test/download?quality=HQ');
    assert.match(rows[1].querySelector('.variant-spec').textContent, / · HQ$/);
    assert.equal(rows[1].querySelector('.qq-quality-tip'), null);
    assert.equal(root.querySelector('#quality-status').textContent, '');
    rerender();
  }
});

test('QQ detail renders the 20 WET tiers in AM-style groups and specified tier order', async (t) => {
  const fixture = JSON.parse(fs.readFileSync(new URL('./fixtures/qq_capabilities.json', import.meta.url), 'utf8'));
  const qualities = fixture.qualities.map(({ quality, displayName }) => ({ quality, format: displayName, available: true, sizeBytes: 1024 }));
  const { root, rerender } = detailPage(t, { qualities: qualities.toReversed() });
  await new Promise(setImmediate);
  for (let i = 0; i < 2; i++) {
    const groups = root.querySelector('#variant-list').children;
    assert.ok(groups.every((group) => group.className === 'quality-group' && group.children[0].tagName === 'h3'));
    assert.deepEqual(groups.map((group) => group.querySelector('.quality-group-title').textContent),
      ['q.lossless', 'q.atmos', 'qq.groupSpatial', 'DTS', 'AAC', 'OGG', 'MP3']);
    assert.deepEqual(groups.map((group) => group.querySelectorAll('.variant').map((row) => row.dataset.quality)), [
      ['RS', 'SQ'], ['DA', 'DA1500', 'DA448', 'DAEAC3'], ['QAI', 'Q360V1', 'Q360V2', 'RA360'],
      ['DTSX', 'DTS'], ['AAC192', 'WEB', 'AAC48'], ['OGG192', 'OGG96'], ['HQ', 'MP3128', 'NAC'],
    ]);
    assert.equal(root.querySelector('#variant-list').querySelectorAll('.variant').length, 20);
    rerender();
  }
});

test('QQ detail preserves unknown tiers in final groups titled by literal format or quality', async (t) => {
  const { root } = detailPage(t, { qualities: [
    { quality: 'NEXT', available: true, sizeBytes: 42, format: '<Future Format>' },
    { quality: 'FUTURE', available: true, sizeBytes: 84 },
    { quality: 'HQ', available: true, sizeBytes: 1024 },
  ] });
  await new Promise(setImmediate);
  const list = root.querySelector('#variant-list');
  assert.deepEqual(list.children.map((group) => group.querySelector('.quality-group-title').textContent), ['MP3', '<Future Format>', 'FUTURE']);
  assert.deepEqual(list.querySelectorAll('.variant').map((row) => row.dataset.quality), ['HQ', 'NEXT', 'FUTURE']);
  assert.equal(list.children[1].querySelector('.variant-dl').href, '/qq/song/test/download?quality=NEXT');
  assert.ok(list.children[1].querySelector('.variant-play').disabled);
});

test('QQ detail synchronizes playing and paused rows inside quality groups', async (t) => {
  const { root, played, setPlaying, rerender } = detailPage(t);
  await new Promise(setImmediate);
  const list = root.querySelector('#variant-list');
  const rowFor = (quality) => list.querySelectorAll('.variant').find((row) => row.dataset.quality === quality);
  await rowFor('HQ').querySelector('.variant-play').click();
  setPlaying(played[0]);
  for (let i = 0; i < 2; i++) {
    assert.match(rowFor('HQ').className, /\bplaying\b/);
    assert.doesNotMatch(rowFor('HQ').className, /\bpaused\b/);
    assert.match(rowFor('HQ').querySelector('.variant-play').innerHTML, /<rect/);
    assert.doesNotMatch(rowFor('DA').className, /\bplaying\b/);
    assert.match(rowFor('DA').querySelector('.variant-play').innerHTML, /<path/);
    assert.ok(list.children.every((group) => group.className === 'quality-group'));
    rerender();
  }
  await rowFor('HQ').querySelector('.variant-play').click();
  assert.equal(played.length, 1);
  assert.match(rowFor('HQ').className, /\bpaused\b/);
  assert.match(rowFor('HQ').querySelector('.variant-play').innerHTML, /<path/);
  assert.equal(root.querySelector('#play-best').textContent, 'song.play');
});

test('QQ quality rows play on double click outside their controls', async (t) => {
  const { root, played } = detailPage(t);
  await new Promise(setImmediate);
  const row = root.querySelector('#variant-list').querySelector('.variant');
  row.events.dblclick[0]({ target: row });
  assert.equal(played[0].quality, 'DA');
});

test('QQ detail header, radiosity, playback and queue use the fixed cover URL', async (t) => {
  const { root, played, queued } = detailPage(t);
  assert.equal(root.querySelector('#lyrics-preview').getAttribute('aria-controls'), 'lyrics-lines');
  await new Promise(setImmediate);
  assert.equal(root.querySelector('#lyrics-qrc').href, '/qq/song/test/lyrics/raw?kind=qrc');
  assert.equal(root.querySelector('#lyrics-trans').href, '/qq/song/test/lyrics/raw?kind=qrctrans');
  assert.equal(root.querySelector('#lyrics-roma').href, '/qq/song/test/lyrics/raw?kind=qrcroma');
  assert.equal(root.querySelector('#lyrics-qrc').hidden, false);
  assert.equal(root.querySelector('#lyrics-trans').hidden, false);
  assert.equal(root.querySelector('#lyrics-roma').hidden, false);
  assert.equal(root.querySelector('#art').querySelector('img').src, fixedCover);
  assert.equal(root.querySelector('#radiosity').querySelector('img').src, fixedCover);
  await root.querySelector('#enqueue').click();
  await root.querySelector('#play-best').click();
  await root.querySelector('#variant-list').querySelector('.variant').querySelector('.variant-play').click();
  assert.equal(queued[0].artwork, fixedCover);
  assert.deepEqual(played.map((item) => item.artwork), [fixedCover, fixedCover]);
});

test('QQ detail retains available downloads when the browser cannot play any tier', async (t) => {
  const { root } = detailPage(t, { nativeModes: [] });
  await new Promise(setImmediate);
  assert.deepEqual(root.querySelector('#variant-list').querySelectorAll('.variant').map((row) => row.dataset.quality), ['DA', 'HQ']);
  assert.equal(root.querySelector('#variant-list').querySelectorAll('.variant-play').length, 2);
  assert.ok(root.querySelector('#variant-list').querySelectorAll('.variant-play').every((button) => button.disabled));
  assert.equal(root.querySelector('#variant-list').querySelectorAll('.variant-dl').length, 2);
  assert.equal(root.querySelector('#play-best').disabled, true);
});

for (const failQualities of [false, true]) {
  test(`QQ detail has no quality rows when ${failQualities ? 'loading fails' : 'all tiers are unavailable'}`, async (t) => {
    const { root } = detailPage(t, { qualities: [{ quality: 'HQ', available: false, sizeBytes: 1024 }], failQualities });
    await new Promise(setImmediate);
    assert.equal(root.querySelector('#variant-list').children.length, 0);
    assert.equal(root.querySelector('#quality-status').textContent, failQualities ? 'qq.qualitiesFailed' : '');
    assert.equal(root.querySelector('#play-best').disabled, true);
    assert.equal(root.querySelector('#title').textContent, 'Test');
  });
}
