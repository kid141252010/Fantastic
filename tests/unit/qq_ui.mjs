import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { runInNewContext } from 'node:vm';
import { document, ids } from '../support/ui-dom.mjs';
import { mount } from '../../web/views/qq-song.mjs';
import * as qq from '../../web/sources/qq.mjs';

const cover = 'https://y.qq.com/music/photo_new/T002R{w}x{h}M000003DFRzD192KKD_5.jpg';
const fixedCover = 'https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg';
const defaultQualities = [
  { quality: 'HQ', available: true, sizeBytes: 1024 },
  { quality: 'SQ', available: false, sizeBytes: 2048 },
  { quality: 'RS', available: true, sizeBytes: 0 },
  { quality: 'QAI', available: 'true', sizeBytes: 8192 },
  { quality: 'DA', available: true, sizeBytes: 4096 },
];

function detailPage(t, { qualities = defaultQualities, failQualities = false, nativeModes = ['native'],
  path = '/https://y.qq.com/n/ryqq/songDetail/test' } = {}) {
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
  const requests = [];
  globalThis.fetch = async (path) => { requests.push(path); return { ok: !failQualities || !path.endsWith('/qualities'), status: 503,
    json: async () => path.endsWith('/qualities') ? { qualities }
      : path.endsWith('/lyrics') ? { candidates: [], rawQrc: true, rawTrans: true, rawRoma: true }
        : { id: 'test', name: 'Test', albumMid: '003DFRzD192KKD', cover } }; };
  mount({ root, url: new URL('https://am.test' + path), signal: controller.signal,
    player, onLangChange: (fn) => { rerender = fn; }, toast() {} });
  return { root, played, queued, requests, rerender: () => rerender(),
    setPlaying: (item) => { player.current = item; paused = false; onPlayerChange(); } };
}

test('QQ routes accept both official versions for all five page types with unchanged IDs', () => {
  const source = fs.readFileSync(new URL('../../web/app.mjs', import.meta.url), 'utf8');
  const code = source.slice(source.indexOf('const CHARTS_PATH'), source.indexOf('/* ---------- 常驻'))
    .replace('export function route', 'function route');
  const route = runInNewContext(`${code}\nroute`, {
    location: new URL('https://am.test'), window: {}, regionlessResource: () => null, PAGES: [],
  });
  for (const [kind, view] of [['songDetail', 'song'], ['albumDetail', 'album'], ['singer', 'artist'], ['playlist', 'playlist'], ['mv/v', 'mv']]) {
    for (const version of ['ryqq', 'ryqq_v2']) {
      for (const scheme of ['https://', 'https:/']) {
        for (const suffix of ['', '/', '?foo=bar', '/?foo=bar']) {
          const url = new URL(`https://am.test/${scheme}y.qq.com/n/${version}/${kind}/003XqDlj393kQi${suffix}`);
          assert.equal(route(url), `qq-${view}`, url.href);
        }
      }
      assert.equal(route(new URL(`https://am.test/https://y.qq.com/n/${version}/${kind}/${'A'.repeat(32)}`)), `qq-${view}`);
      for (const id of ['', 'a-b', 'a_b', '中', 'a'.repeat(33), '123//', '123/play', '%31']) {
        const url = new URL(`https://am.test/https://y.qq.com/n/${version}/${kind}/${id}`);
        assert.equal(route(url), null, url.href);
      }
    }
  }
  assert.equal(route(new URL('https://am.test/https://y.qq.com/n/ryqq_v3/songDetail/123')), null);
  assert.equal(route(new URL('https://elsewhere.test/https://y.qq.com/n/ryqq_v2/songDetail/123')), null);
});

for (const version of ['ryqq', 'ryqq_v2']) {
  test(`QQ song view extracts the mid from ${version}`, async (t) => {
    const { root, requests } = detailPage(t, { path: `/https:/y.qq.com/n/${version}/songDetail/test/?foo=bar` });
    await new Promise(setImmediate);
    assert.equal(root.querySelector('#title').textContent, 'Test');
    assert.ok(requests.includes('/qq/song/test'));
    assert.ok(requests.includes('/qq/song/test/qualities'));
    assert.ok(requests.includes('/qq/song/test/lyrics'));
    assert.equal(root.querySelector('#alert').hidden, true);
  });

  for (const [kind, page] of [['album', 'albumDetail'], ['artist', 'singer'], ['playlist', 'playlist'], ['mv', 'mv/v']]) {
    test(`QQ ${kind} view extracts the mid from ${version} and retains canonical navigation`, async (t) => {
      const originalFetch = globalThis.fetch;
      const controller = new AbortController();
      t.after(() => { controller.abort(); globalThis.fetch = originalFetch; });
      const root = ids('hero', 'title', 'artist', 'art', 'radiosity', 'sub', 'notes', 'notes-text', 'notes-more', 'play-all', 'shuffle',
        'alert', 'tracks', 'footer', 'creator', 'more', 'name', 'hero-art', 'portrait', 'portrait-glow', 'counts', 'bio', 'born',
        'bio-text', 'bio-more', 'shelves', 'mv-shelf', 'video', 'open', 'actions', 'meta', 'desc');
      for (const node of root.children) node.classList.remove = () => {};
      const buttons = document.createElement('div'); buttons.className = 'detail-actions'; root.querySelector('#hero').append(buttons);
      Object.assign(root.querySelector('#video'), { pause() {}, load() {} });
      const song = { id: 'song123', name: 'Song', album: 'Album', albumMid: 'album123', artists: [{ name: 'Artist', mid: 'artist123' }] };
      const album = { mid: 'album123', name: 'Album', artists: song.artists };
      const requests = [], played = [];
      globalThis.fetch = async (path) => {
        requests.push(path);
        return { ok: true, json: async () => kind === 'album' ? { album, songs: [song] }
          : kind === 'artist' ? { artist: { mid: 'test123', name: 'Artist' }, songs: [song], albums: [album] }
            : kind === 'playlist' ? { playlist: { id: 'test123', name: 'Playlist' }, songs: [song] }
              : { vid: 'test123', name: 'MV', artists: song.artists } };
      };
      // Keep the real view bodies and source adapter; inject browser-only controls, as in ncm_ui.mjs.
      const source = fs.readFileSync(new URL(`../../web/views/qq-${kind}.mjs`, import.meta.url), 'utf8')
        .replace(/^import .*;\n/gm, '').replace(/^export /gm, '');
      const mountView = runInNewContext(`${source}\nmount`, {
        window: { AmI18n: { t: (key) => key } }, document, qq, trackKey: qq.trackKey, artUrl: (value) => value?.url || value || '',
        createActions: () => ({ headerMenu: () => document.createElement('button'), moreButton: () => document.createElement('button') }),
        createDetailHeader: () => () => {}, syncTrackRows() {},
        trackRow: (track) => { const row = document.createElement('a'); row.href = track.href; return { row, track }; },
      });
      mountView({ root, signal: controller.signal, url: new URL(`https://am.test/https:/y.qq.com/n/${version}/${page}/test123/?foo=bar`),
        player: { current: null, onChange() {}, pause() {}, playQueue: (entries) => played.push(entries) },
        navigate() {}, onLangChange() {}, toast() {} });
      await new Promise(setImmediate);
      assert.equal(requests[0], `/qq/${kind}/test123${kind === 'playlist' ? '?offset=0' : ''}`);
      assert.equal(root.querySelector('#alert').hidden, true);
      assert.equal(root.querySelector(kind === 'artist' ? '#name' : '#title').textContent,
        ({ album: 'Album', artist: 'Artist', playlist: 'Playlist', mv: 'MV' })[kind]);
      if (kind !== 'mv') {
        await root.querySelector('#play-all').click();
        assert.equal(played[0][0].key, 'qq:song123');
        assert.equal(played[0][0].href, '/https://y.qq.com/n/ryqq/songDetail/song123');
      } else assert.equal(root.querySelector('#video').src, '/qq/mv/test123/play');
      if (kind === 'album' || kind === 'mv') assert.equal(root.querySelector('#artist').querySelector('a').href, '/https://y.qq.com/n/ryqq/singer/artist123');
      if (kind === 'artist') assert.equal(root.querySelector('#shelves').querySelector('a').href, '/https://y.qq.com/n/ryqq/albumDetail/album123');
    });
  }
}

test('QQ collection actions extract album and playlist IDs from both official versions', async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => { globalThis.fetch = originalFetch; });
  const source = fs.readFileSync(new URL('../../web/views/actions.mjs', import.meta.url), 'utf8');
  const start = source.indexOf('async function qqCollectionEntries');
  const code = source.slice(start, source.indexOf('  async function play(', start));
  const entriesFor = runInNewContext(`${code}\nqqCollectionEntries`, { qq });
  for (const version of ['ryqq', 'ryqq_v2']) {
    for (const [kind, page] of [['album', 'albumDetail'], ['playlist', 'playlist']]) {
      const requests = [];
      globalThis.fetch = async (path) => { requests.push(path); return { ok: true, json: async () => ({ songs: [{ id: 'song123', name: 'Song' }] }) }; };
      const entries = await entriesFor({ kind, href: `/https://y.qq.com/n/${version}/${page}/test123` });
      assert.equal(requests[0], `/qq/${kind}/test123${kind === 'playlist' ? '?offset=0' : ''}`);
      assert.equal(entries[0].key, 'qq:song123');
    }
  }
});

test('QQ homepage recognizes pasted official links, labels their kinds and navigates without search', () => {
  const source = fs.readFileSync(new URL('../../web/views/home.mjs', import.meta.url), 'utf8')
    .replace(/^import .*;\n/gm, '').replace(/^export /gm, '');
  const browserDocument = { ...document, addEventListener() {} };
  const window = { FANTASTIC_HOST: { qqEnabled: true }, AmI18n: { t: (key) => key, onSettingsChange: () => () => {} } };
  const mountHome = runInNewContext(`${source}\nmount`, {
    window, document: browserDocument, qq: { ...qq, enabled: () => true }, ncm: { enabled: () => false }, URL,
    location: new URL('https://am.test/?source=qq'), localStorage: { getItem: () => null, setItem() {}, removeItem() {} },
    createActions: () => ({}), createResults: () => ({ close() {}, run() { assert.fail('pasted QQ links must navigate directly'); } }),
    addEventListener() {}, ResizeObserver: class { observe() {} disconnect() {} },
    setTimeout() { assert.fail('recognized QQ links must not schedule suggestions'); }, clearTimeout() {},
  });
  const root = ids('input', 'error', 'detect', 'submit-label', 'suggest', 'landing', 'source-select', 'source-switch', 'storefront',
    'categories', 'results', 'results-title', 'results-tabs', 'results-body', 'form', 'close-results', 'clear-recent', 'recent', 'recent-list',
    'brick-grid', 'categories-toggle', 'categories-retry', 'footer-version');
  root.querySelector('#input').value = '';
  root.querySelector('#source-select').options = [];
  const navigated = [];
  const controller = new AbortController();
  try {
    mountHome({ root, signal: controller.signal, player: {}, navigate: (path) => navigated.push(path), onLangChange() {},
      loadStatus() { assert.fail('QQ links do not need Apple status'); } });
    for (const version of ['ryqq', 'ryqq_v2']) {
      for (const [kind, label] of [['songDetail', 'Song'], ['albumDetail', 'Album'], ['singer', 'Artist'], ['playlist', 'Playlist'], ['mv/v', 'Mv']]) {
        const link = `https://y.qq.com/n/${version}/${kind}/003XqDlj393kQi`;
        root.querySelector('#input').value = link;
        for (const input of root.querySelector('#input').events.input) input();
        assert.equal(root.querySelector('#detect').hidden, false);
        assert.equal(root.querySelector('#detect').textContent, `home.detect${label}`);
        assert.equal(root.querySelector('#submit-label').textContent, 'home.submit');
        for (const submit of root.querySelector('#form').events.submit) submit({ preventDefault() {} });
        assert.equal(navigated.at(-1), '/' + link);
      }
    }
    assert.equal(navigated.length, 10);
  } finally { controller.abort(); }
});

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
  const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/qq_capabilities.json', import.meta.url), 'utf8'));
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
