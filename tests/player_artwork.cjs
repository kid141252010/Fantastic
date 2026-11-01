const assert = require('node:assert/strict');
const { test } = require('node:test');
const { readFileSync } = require('node:fs');
const vm = require('node:vm');
const { AmPlayer } = require('../src/ui/player.js');

for (const source of ['apple', 'qq']) test(`Media Session fetches ${source} artwork and reuses its blob across qualities`, async () => {
  const original = { navigator: Object.getOwnPropertyDescriptor(global, 'navigator'), MediaMetadata: global.MediaMetadata, fetch: global.fetch };
  const session = { metadata: null };
  const requests = [];
  Object.defineProperty(global, 'navigator', { configurable: true, value: { mediaSession: session } });
  global.MediaMetadata = class MediaMetadata {
    constructor(value) { Object.assign(this, value); }
  };
  global.fetch = async (url) => {
    requests.push(url);
    return { ok: true, blob: async () => new Blob(['jpeg'], { type: 'image/jpeg' }) };
  };

  const player = Object.create(AmPlayer.prototype);
  player.playToken = 1;
  const item = { source, title: 'Test', artwork: source === 'qq'
    ? 'https://y.qq.com/music/photo_new/T002R500x500M000album_5.jpg'
    : 'https://is1-ssl.mzstatic.com/image/600x600bb.jpg', artworkFetchUrl: '/qq/cover/album' };
  player.current = { ...item };
  const expected = source === 'qq' ? item.artworkFetchUrl : item.artwork;
  try {
    player.updateMediaSession();
    await new Promise(setImmediate);
    assert.deepEqual(requests, [expected]);
    assert.equal(player.current.artwork, item.artwork);
    assert.match(session.metadata.artwork[0].src, /^blob:/);
    assert.equal(session.metadata.title, 'Test');
    const artworkUrl = session.metadata.artwork[0].src;
    const metadata = session.metadata;

    player.playToken = 2;
    player.current = { ...item };
    player.updateMediaSession();
    assert.deepEqual(requests, [expected]);
    assert.equal(session.metadata.artwork[0].src, artworkUrl);
    assert.equal(session.metadata, metadata);
  } finally {
    if (player.mediaArtController) player.mediaArtController.abort();
    if (player.mediaArtUrl) URL.revokeObjectURL(player.mediaArtUrl);
    if (original.navigator) Object.defineProperty(global, 'navigator', original.navigator);
    else delete global.navigator;
    global.MediaMetadata = original.MediaMetadata;
    global.fetch = original.fetch;
  }
});

test('lyrics backdrop fetches QQ through the adapter, keeps the img direct, and switches back to Apple', async () => {
  // Exercise the existing panel closure functions without its unrelated layout/WebGL setup.
  const source = readFileSync(require.resolve('../src/ui/lyrics/panel.mjs'), 'utf8');
  const section = (begin, end) => source.slice(source.indexOf(begin), source.indexOf(end, source.indexOf(begin)));
  const requests = [], images = [];
  let meta = { artwork: 'https://y.qq.com/music/photo_new/T002R500x500M000album_5.jpg', artworkFetchUrl: '/qq/cover/album' };
  const art = { getAttribute: () => art.src, removeAttribute: () => { delete art.src; } };
  const canvas = { hidden: true };
  const classes = new Set();
  const backdrop = { setImage: async (blob) => { images.push(blob); return true; }, resume() {}, pause() {} };
  const context = vm.createContext({
    getMeta: () => meta, art, canvas, backdrop, heads: [], AbortController, console,
    root: { classList: { add: (name) => classes.add(name), remove: (name) => classes.delete(name) } },
    fetch: async (url, options) => {
      requests.push({ url, options });
      if (url.startsWith('https://y.qq.com/')) throw new Error('QQ CDN has no CORS');
      return { ok: true, blob: async () => new Blob(['jpeg'], { type: 'image/jpeg' }) };
    },
  });
  vm.runInContext(`let artworkSource = '', artworkController = null, covered = false, open = true;\n${
    section('  function renderHeader() {', '\n  function renderCredits()')
  }\n${section('  function loadArtwork() {', '\n  function setLines()')}\nthis.renderHeader = renderHeader; this.loadArtwork = loadArtwork;`, context);
  context.renderHeader();
  context.loadArtwork();
  await new Promise(setImmediate);
  assert.equal(requests[0].url, '/qq/cover/album');
  assert.equal(requests[0].options.cache, 'force-cache');
  assert.equal(art.src, meta.artwork);
  assert.equal(images.length, 1);
  assert.equal(images[0].type, 'image/jpeg');
  assert.equal(canvas.hidden, false);
  assert.ok(classes.has('has-backdrop'));
  context.loadArtwork();
  assert.equal(requests.length, 1, 'same album reuses the backdrop');
  meta = { artwork: 'https://is1-ssl.mzstatic.com/image/600x600bb.jpg' };
  context.renderHeader();
  context.loadArtwork();
  await new Promise(setImmediate);
  assert.equal(requests[0].options.signal.aborted, true);
  assert.equal(requests[1].url, meta.artwork);
  assert.equal(art.src, meta.artwork);
  assert.equal(images.length, 2);
});
