import test from 'node:test';
import assert from 'node:assert/strict';
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
  const saved = Object.fromEntries(['document', 'window', 'MediaSource', 'matchMedia', 'fetch', 'localStorage']
    .map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const controller = new AbortController();
  t.after(() => {
    controller.abort();
    for (const [key, descriptor] of Object.entries(saved)) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  });
  const root = ids('favorite', 'title', 'subtitle', 'meta', 'badges', 'art', 'radiosity', 'enqueue', 'play-best',
    'variant-list', 'quality-status', 'lyrics', 'lyrics-source', 'lyrics-actions', 'lyrics-preview',
    'lyrics-qrc', 'lyrics-trans', 'lyrics-roma', 'lyrics-lines', 'alert');
  const data = new Map();
  const local = { getItem: (key) => data.get(key) ?? null, setItem: (key, value) => data.set(key, value) };
  const played = [], queued = [];
  const player = { current: null, onChange() {}, play: (item) => played.push(item), appendQueue: (entry) => queued.push(entry) };
  let rerender;
  globalThis.document = document;
  globalThis.localStorage = local;
  globalThis.matchMedia = () => ({ matches: false, addEventListener() {} });
  globalThis.window = { AmI18n: { t: (key) => key }, AmHook: { formatTime: String, detectQqModes: () => nativeModes } };
  globalThis.MediaSource = { isTypeSupported: () => true };
  globalThis.fetch = async (path) => ({ ok: !failQualities || !path.endsWith('/qualities'), status: 503,
    json: async () => path.endsWith('/qualities') ? { qualities }
      : path.endsWith('/lyrics') ? { candidates: [], rawQrc: true, rawTrans: true, rawRoma: true }
        : { id: 'test', name: 'Test', albumMid: '003DFRzD192KKD', cover } });
  mount({ root, url: new URL('https://am.test/qq-song/test'), signal: controller.signal,
    player, onLangChange: (fn) => { rerender = fn; }, toast() {} });
  return { root, local, played, queued, rerender: () => rerender() };
}

test('QQ detail omits unavailable tiers while retaining runtime-playable and download rows after rerender', async (t) => {
  const { root, rerender } = detailPage(t);
  assert.equal(root.querySelector('#variant-list').children.length, 0, 'loading has no placeholder tiers');
  assert.equal(root.querySelector('#quality-status').textContent, 'qq.qualitiesLoading');
  await new Promise(setImmediate);
  for (let i = 0; i < 2; i++) {
    const rows = root.querySelector('#variant-list').children;
    assert.deepEqual(rows.map((row) => row.dataset.quality), ['HQ', 'DA']);
    assert.ok(rows[0].querySelector('button'));
    assert.equal(rows[0].querySelector('a').href, '/qq/song/test/download?quality=HQ');
    assert.ok(rows[1].querySelector('button'));
    assert.equal(rows[1].querySelector('a').href, '/qq/song/test/download?quality=DA');
    assert.equal(root.querySelector('#quality-status').textContent, '');
    rerender();
  }
});

test('QQ detail header, radiosity, playback, favorites and queue use the fixed cover URL', async (t) => {
  const { root, local, played, queued } = detailPage(t);
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
  await root.querySelector('#variant-list').children[0].querySelector('button').click();
  await root.querySelector('#favorite').click();
  assert.equal(queued[0].artwork, fixedCover);
  assert.deepEqual(played.map((item) => item.artwork), [fixedCover, fixedCover]);
  assert.deepEqual(JSON.parse(local.getItem('am-hook:qq-favorites'))[0].artwork, { url: fixedCover });
});

test('QQ detail retains available downloads when the browser cannot play any tier', async (t) => {
  const { root } = detailPage(t, { nativeModes: [] });
  await new Promise(setImmediate);
  assert.deepEqual(root.querySelector('#variant-list').children.map((row) => row.dataset.quality), ['HQ', 'DA']);
  assert.equal(root.querySelector('#variant-list').querySelectorAll('button').length, 0);
  assert.equal(root.querySelector('#variant-list').querySelectorAll('a').length, 2);
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
