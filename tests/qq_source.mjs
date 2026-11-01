// Offline QQ contract / fallback / queue regression tests: node --test tests/qq_source.mjs
// Keep the production source import before the player CommonJS bridge so capability startup is exercised first.
import test, { afterEach, beforeEach } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import * as qq from '../src/ui/sources/qq.mjs';
import { sourceOf, trackKey, withIdentity } from '../src/ui/sources/track.mjs';
import { artUrl } from '../src/ui/library.mjs';
const { AmPlayer, trackKey: playerKey, detectQqModes, useJocSettings, usePlaybackSettings } = createRequire(import.meta.url)('../src/ui/player.js');
const browserGlobals = Object.fromEntries(['window', 'document', 'MediaSource', 'fetch']
  .map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
const nativeFetch = globalThis.fetch;
beforeEach(() => {
  qq.clearMimeSupportCache();
  globalThis.window = {};
  globalThis.document = { createElement: () => ({ canPlayType: () => 'probably' }) };
  globalThis.MediaSource = { isTypeSupported: () => true };
});
afterEach(() => {
  qq.clearMimeSupportCache();
  for (const [key, descriptor] of Object.entries(browserGlobals)) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor); else delete globalThis[key];
  }
});

const dto = {
  id: '0039MnYb0qxYhV', name: '晴天', artists: ['周杰伦'], album: '叶惠美', albumMid: '003DFRzD192KKD',
  durationMs: 269000, vip: true, ttml: true,
  file_url: 'https://example.invalid/private.mp3',
  cover: 'https://y.qq.com/music/photo_new/T002R{w}x{h}M000003DFRzD192KKD_5.jpg',
};
test('QQ DTO maps names, timing, badges and direct QQ artwork', () => {
  const track = qq.toTrack(dto);
  assert.equal(track.key, 'qq:0039MnYb0qxYhV');
  assert.equal(track.source, 'qq');
  assert.equal(track.kind, 'song');
  assert.equal(track.name, '晴天');
  assert.equal(track.artist, '周杰伦');
  assert.deepEqual(track.artists, [{ name: '周杰伦', href: '' }]);
  assert.equal(track.duration, 269000);
  assert.deepEqual(track.qq, { mid: dto.id, albumMid: dto.albumMid, vip: true, ttml: true });
  assert.deepEqual(track.artwork, { url: qq.artworkUrl(dto.cover) });
  assert.equal(track.href, `/qq-song/${dto.id}`);
  assert.equal(track.albumHref, `/qq-album/${dto.albumMid}`);
  assert.equal(JSON.stringify(track).includes('example.invalid'), false);
  assert.equal(JSON.stringify(track).includes('file_url'), false);
  const entry = qq.entryOf(track);
  assert.equal(entry.track, dto.id);
  assert.equal(entry.key, track.key);
  assert.equal(entry.artwork, 'https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg');
  assert.equal(entry.artworkFetchUrl, `/qq/cover/${dto.albumMid}`);
  assert.equal(qq.qualityItem(track, 'SQ').artworkFetchUrl, entry.artworkFetchUrl);
  const opaque = { ...dto, cover: 'https://y.qq.com/cover.jpg?version=5' };
  assert.equal(qq.entryOf(qq.toTrack(opaque)).artwork, opaque.cover);
});

test('QQ DTO turns valid artist and album mids into local links', () => {
  const track = qq.toTrack({ ...dto, artists: [{ name: '周杰伦', mid: '0025NhlN2yWrP4' }] });
  assert.deepEqual(track.artists, [{ name: '周杰伦', href: '/qq-artist/0025NhlN2yWrP4' }]);
  assert.equal(track.albumHref, `/qq-album/${dto.albumMid}`);
});

test('QQ optional fields default safely; invalid IDs cannot form adapter paths', () => {
  const track = qq.toTrack({ id: '1', albumMid: '../bad', durationMs: -5 });
  assert.equal(track.duration, 0);
  assert.deepEqual(track.artwork, { url: '' });
  assert.equal(track.qq.vip, false);
  assert.equal(track.qq.ttml, false);
  assert.deepEqual(track.artists, []);
  for (const id of [undefined, null, '../secret', 'a?x=1', 'x'.repeat(33), '中']) {
    assert.throws(() => qq.toTrack({ id }));
    assert.throws(() => qq.playUrl(id));
    assert.throws(() => qq.downloadUrl(id));
    assert.throws(() => qq.songPath(id));
    assert.throws(() => qq.coverUrl(id));
  }
  assert.equal(qq.playUrl('1'), '/qq/song/1/play');
  assert.equal(qq.downloadUrl('1'), '/qq/song/1/download');
  assert.equal(qq.coverUrl('1'), '/qq/cover/1');
  assert.equal(qq.coverUrl(''), '');
});

test('QQ tracks without an album mid do not manufacture artwork URLs', () => {
  const track = qq.toTrack({ id: 'songonly', name: 'Song', artists: ['Artist'], cover: dto.cover });
  assert.equal(track.albumId, '');
  assert.deepEqual(track.artwork, { url: '' });
  const entry = qq.entryOf(track);
  assert.equal(entry.artwork, '');
  assert.equal(entry.artworkFetchUrl, '');
});

test('QQ quality rows list all tiers, retain formats and sizes, and omit unavailable actions', () => {
  const rows = qq.qualityRows({ qualities: [
    { quality: 'SQ', available: true, sizeBytes: 55397039, format: 'FLAC' },
    { quality: 'HQ', available: true, sizeBytes: 10792943, format: 'MP3 320 kbps' },
    { quality: 'WEB', available: false, sizeBytes: 0, format: 'MP3 128 kbps' },
    { quality: 'NAC', available: true, sizeBytes: 0, format: 'AAC' },
    { quality: 'unknown', available: true, sizeBytes: 42, format: 'ignored' },
  ] }, dto.id);
  assert.deepEqual(rows.map((row) => row.quality), ['MP3128', 'AAC48', 'WEB', 'AAC192', 'OGG96', 'OGG192', 'HQ', 'NAC',
    'SQ', 'RS', 'QAI', 'Q360V1', 'Q360V2', 'RA360', 'DTS', 'DTSX', 'DA', 'DAEAC3', 'DA448', 'DA1500']);
  const byQuality = Object.fromEntries(rows.map((row) => [row.quality, row]));
  assert.equal(byQuality.HQ.format, 'MP3 320 kbps');
  assert.equal(byQuality.HQ.sizeBytes, 10792943);
  assert.equal(byQuality.SQ.downloadUrl, `/qq/song/${dto.id}/download?quality=SQ`);
  assert.equal(byQuality.SQ.playUrl, `/qq/song/${dto.id}/play?quality=SQ`);
  for (const row of rows.filter((row) => !['HQ', 'SQ'].includes(row.quality))) {
    assert.equal(row.available, false);
    assert.equal(Object.hasOwn(row, 'playUrl'), false);
    assert.equal(Object.hasOwn(row, 'downloadUrl'), false);
  }
  assert.equal(byQuality.RS.sizeBytes, null);
  assert.equal(qq.qualityRows(null, dto.id).length, 20);
  for (const quality of ['', 'hq', '../SQ', 'HQ&x=1', null, 1]) {
    assert.throws(() => qq.playUrl(dto.id, quality));
    assert.throws(() => qq.downloadUrl(dto.id, quality));
    assert.throws(() => qq.qualityEntry(qq.toTrack(dto), quality));
  }
});

test('QQ consumes the complete capabilities table at module startup and keeps fallback on malformed data', async () => {
  const saved = { window: Object.getOwnPropertyDescriptor(globalThis, 'window'), fetch: Object.getOwnPropertyDescriptor(globalThis, 'fetch') };
  const base = qq.capabilities();
  const fixture = JSON.parse(fs.readFileSync(new URL('./fixtures/qq_capabilities.json', import.meta.url), 'utf8'));
  const withoutDisplayName = (rows) => rows.map(({ displayName, ...entry }) => entry);
  assert.deepEqual(withoutDisplayName(base), withoutDisplayName(fixture.qualities));
  const custom = base.map((entry) => entry.quality === 'HQ'
    ? { ...entry, defaultCodec: 'vorbis', contentType: 'audio/ogg; codecs="vorbis"' } : entry);
  let requested = 0;
  Object.defineProperty(globalThis, 'window', { value: {}, configurable: true });
  Object.defineProperty(globalThis, 'fetch', { value: async (path) => {
    assert.equal(path, '/qq/capabilities'); requested++;
    return { ok: true, json: async () => ({ enabled: true, qualities: custom }) };
  }, configurable: true });
  try {
    const loaded = await import(`../src/ui/sources/qq.mjs?capabilities-${Date.now()}`);
    await loaded.capabilitiesReady;
    assert.equal(requested, 1);
    assert.equal(globalThis.AmHookQq?.supportsMime, undefined);
    assert.equal(globalThis.AmHook?.supportsMime, undefined);
    assert.equal(loaded.playbackInfo('HQ').codec, 'vorbis');
    assert.equal(loaded.playbackInfo('HQ').contentType, 'audio/ogg; codecs="vorbis"');
    assert.equal(loaded.capabilities().length, 20);
    let malformedId = 0;
    const malformed = [
      { ok: true, json: async () => ({ enabled: true, qualities: [] }) },
      { ok: true, json: async () => ({ enabled: true, qualities: base.slice(0, -1).concat(base[0]) }) },
      { ok: true, json: async () => ({ enabled: true, qualities: base.map((row) => row.quality === 'HQ' ? { ...row, playbackClass: 'broken' } : row) }) },
      { ok: true, json: async () => ({ enabled: false, qualities: base }) },
      { ok: false, status: 503 },
    ];
    for (const response of malformed) {
      globalThis.fetch = async () => response;
      const fallback = await import(`../src/ui/sources/qq.mjs?capabilities-malformed-${Date.now()}-${malformedId++}`);
      await fallback.capabilitiesReady;
      assert.deepEqual(fallback.capabilities(), base);
    }

    let release;
    let qualityCalls = 0;
    const capabilityResponse = new Promise((resolve) => { release = resolve; });
    globalThis.fetch = async (path) => {
      if (path === '/qq/capabilities') return capabilityResponse;
      qualityCalls++;
      return { ok: true, json: async () => ({ qualities: [] }) };
    };
    const waiting = await import(`../src/ui/sources/qq.mjs?capabilities-wait-${Date.now()}`);
    const pending = waiting.qualities('song');
    await Promise.resolve();
    assert.equal(qualityCalls, 0);
    release({ ok: true, json: async () => ({ enabled: true, qualities: base }) });
    await pending;
    assert.equal(qualityCalls, 1);
  } finally {
    if (saved.window) Object.defineProperty(globalThis, 'window', saved.window); else delete globalThis.window;
    if (saved.fetch) Object.defineProperty(globalThis, 'fetch', saved.fetch); else delete globalThis.fetch;
  }
});

test('QQ capabilities keep the bundled table when a non-2xx response has valid but different JSON', async () => {
  const saved = { window: Object.getOwnPropertyDescriptor(globalThis, 'window'), fetch: Object.getOwnPropertyDescriptor(globalThis, 'fetch') };
  const base = qq.capabilities();
  const altered = base.map((entry) => entry.quality === 'HQ'
    ? { ...entry, defaultCodec: 'vorbis', contentType: 'audio/ogg; codecs="vorbis"', displayName: 'MUTATED HQ' } : entry);
  Object.defineProperty(globalThis, 'window', { value: {}, configurable: true });
  Object.defineProperty(globalThis, 'fetch', { value: async () => ({
    ok: false,
    status: 500,
    json: async () => ({ enabled: true, qualities: altered }),
  }), configurable: true });
  try {
    const fallback = await import(`../src/ui/sources/qq.mjs?capabilities-http-error-${Date.now()}`);
    await fallback.capabilitiesReady;
    assert.deepEqual(fallback.capabilities(), base);
    assert.equal(fallback.playbackInfo('HQ').codec, 'mp3');
    assert.equal(fallback.playbackInfo('HQ').contentType, 'audio/mpeg');
  } finally {
    if (saved.window) Object.defineProperty(globalThis, 'window', saved.window); else delete globalThis.window;
    if (saved.fetch) Object.defineProperty(globalThis, 'fetch', saved.fetch); else delete globalThis.fetch;
  }
});

test('QQ capabilities keep the bundled table when the capabilities request rejects', async () => {
  const saved = { window: Object.getOwnPropertyDescriptor(globalThis, 'window'), fetch: Object.getOwnPropertyDescriptor(globalThis, 'fetch') };
  const base = qq.capabilities();
  Object.defineProperty(globalThis, 'window', { value: {}, configurable: true });
  Object.defineProperty(globalThis, 'fetch', { value: async (path) => {
    if (path === '/qq/capabilities') throw new Error('network offline');
    assert.equal(path, '/qq/song/song/qualities');
    return { ok: true, json: async () => ({ qualities: [{ quality: 'HQ', available: true, sizeBytes: 1 }] }) };
  }, configurable: true });
  try {
    const fallback = await import(`../src/ui/sources/qq.mjs?capabilities-rejected-${Date.now()}`);
    await assert.doesNotReject(fallback.capabilitiesReady);
    assert.deepEqual(fallback.capabilities(), base);
    assert.equal(fallback.playbackInfo('HQ').codec, 'mp3');
    const rows = await fallback.qualities('song');
    assert.equal(rows.find((row) => row.quality === 'HQ').codec, 'mp3');
  } finally {
    if (saved.window) Object.defineProperty(globalThis, 'window', saved.window); else delete globalThis.window;
    if (saved.fetch) Object.defineProperty(globalThis, 'fetch', saved.fetch); else delete globalThis.fetch;
  }
});

test('detail lyric lines use the best parsed source and retain translations as text', () => {
  const result = qq.selectLyrics([candidate('ttml', 'bad'), candidate('lrc', 'valid')], (text) => {
    if (text === 'bad') throw new Error('bad XML');
    return { lines: [{ text: '晴天 <script>', translation: 'Sunny day' }, { text: '  ' }] };
  });
  assert.equal(result.source, 'lrc');
  assert.deepEqual(qq.lyricLines(result), [{ text: '晴天 <script>', translation: 'Sunny day' }]);
  assert.deepEqual(qq.lyricLines({ song: null }), []);
});

test('source namespaces isolate equal numeric IDs and upgrade old Apple snapshots', () => {
  for (const input of [{ id: '123' }, { track: '123' }, { id: 'variant', track: '123' }, { id: '123', key: 'qq:forged' }]) {
    assert.equal(trackKey(input), 'apple:123');
    assert.equal(playerKey(input), trackKey(input));
    assert.equal(withIdentity(input).source, 'apple');
  }
  const apple = withIdentity({ id: '123' });
  const other = withIdentity({ source: 'qq', id: '123' });
  assert.notEqual(apple.key, other.key);
  assert.equal(playerKey(other), other.key);
  assert.equal(new Set([apple.key, other.key]).size, 2);
  assert.equal(sourceOf({ source: 'unknown' }), 'apple');
});

const candidate = (source, text = source) => ({ source, text });
const parse = (text) => {
  if (text === 'malformed') throw new Error('invalid XML');
  return { lines: text === 'empty' ? [] : [{ text }] };
};
test('QQ lyrics prefer AMLL, upstream TTML, QRC, then LRC regardless of response order', () => {
  const sources = ['lrc', 'qrc', 'ttml', 'amll'];
  for (const expected of ['amll', 'ttml', 'qrc', 'lrc']) {
    assert.equal(qq.selectLyrics(sources.map((s) => candidate(s)), parse).source, expected);
    sources.splice(sources.indexOf(expected), 1);
  }
});
test('missing, blank, malformed and untimed lyrics fall through without blocking LRC', () => {
  assert.equal(qq.selectLyrics([candidate('amll', 'malformed'), candidate('ttml', 'empty'), candidate('qrc', ' '), candidate('lrc')], parse).source, 'lrc');
  assert.equal(qq.selectLyrics([candidate('ttml', null), candidate('qrc', {}), candidate('lrc')], parse).source, 'lrc');
  assert.deepEqual(qq.selectLyrics(undefined, parse), { song: null });
  assert.deepEqual(qq.selectLyrics([candidate('apple'), candidate('unknown')], parse), { song: null });
});

test('home search source changes do not switch or tear down the playing source', () => {
  const home = fs.readFileSync(new URL('../src/ui/views/home.mjs', import.meta.url), 'utf8');
  assert.doesNotMatch(home, /player\.switchSource\(source\)/);
  const player = playerHarness();
  player.current = { source: 'apple', track: '123' };
  let tornDown = 0;
  player.teardown = () => { tornDown++; };
  player.switchSource('qq');
  assert.equal(tornDown, 1, 'the old switchSource path tears down playback; home must not call it');
});

test('QQ covers resolve for rows and player entries; Apple templates retain their shape', () => {
  const track = qq.toTrack(dto);
  const fixed = dto.cover.replace('{w}', '500').replace('{h}', '500');
  assert.equal(track.artwork.url, fixed);
  assert.doesNotMatch(track.artwork.url, /\{[wh]\}/);
  for (const size of [68, 80, 96, 160, 600, 632, 1000]) assert.equal(artUrl(track.artwork, size), fixed);
  for (const artwork of [dto.cover, { url: dto.cover }]) {
    assert.equal(qq.entryOf({ ...track, artwork }).artwork, fixed);
    assert.equal(qq.qualityItem({ ...track, artwork }, 'SQ').artwork, fixed);
  }
  assert.equal(qq.entryOf(track).artwork, fixed);
  const apple = 'https://example.com/{w}x{h}{c}.{f}';
  assert.equal(artUrl(apple, 80, 60), 'https://example.com/80x60bb.jpg');
  assert.equal(artUrl({ url: apple }, 80, 60), 'https://example.com/80x60bb.jpg');
});

test('QQ clients only request local adapter paths and pass cancellation signals', async () => {
  const original = globalThis.fetch;
  const calls = [];
  const controller = new AbortController();
  globalThis.fetch = async (path, options) => {
    calls.push({ path, options });
    if (path.includes('/lyrics')) return { ok: true, json: async () => ({ candidates: [candidate('lrc')], rawQrc: true, rawTrans: false, rawRoma: true }) };
    if (path.includes('/qualities')) return { ok: true, json: async () => ({ qualities: [{ quality: 'HQ', available: true, sizeBytes: 10792943, format: 'MP3 320 kbps' }] }) };
    return { ok: true, json: async () => path.includes('/search') ? { songs: [dto], total: 99 } : dto };
  };
  try {
    const result = await qq.searchSongs('晴天 & 风', { limit: 10, offset: 20, signal: controller.signal });
    assert.equal(result.total, 99);
    assert.equal(result.songs[0].key, `qq:${dto.id}`);
    assert.equal(result.songs[0].artwork.url, 'https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg');
    assert.equal(new URL(calls[0].path, 'https://example.invalid').searchParams.get('q'), '晴天 & 风');
    assert.equal(calls[0].options.signal, controller.signal);
    assert.equal((await qq.song(dto.id)).key, `qq:${dto.id}`);
    const lyric = await qq.lyrics(dto.id, parse);
    assert.equal(lyric.source, 'lrc');
    assert.deepEqual({ rawQrc: lyric.rawQrc, rawTrans: lyric.rawTrans, rawRoma: lyric.rawRoma }, { rawQrc: true, rawTrans: false, rawRoma: true });
    assert.equal((await qq.qualities(dto.id, controller.signal)).find((row) => row.quality === 'HQ').available, true);
    await qq.qualities(dto.id, controller.signal, { basic: true });
    assert.equal(calls.at(-1).path, `/qq/song/${dto.id}/qualities?basic=1`);
    assert.equal(calls.at(-1).options.signal, controller.signal);
    assert.ok(calls.every(({ path }) => path.startsWith('/qq/')));
    globalThis.fetch = async () => ({ ok: false, status: 503, json() { throw new Error('must not read upstream body'); } });
    await assert.rejects(qq.searchSongs('x'), /QQ HTTP 503/);
  } finally { globalThis.fetch = original; }
});

// Small transport harness, using the real queue/source/start/tryMode methods.
function playerHarness() {
  if (globalThis.fetch === nativeFetch) globalThis.fetch = async () => { throw new Error('offline'); };
  const player = Object.create(AmPlayer.prototype);
  Object.assign(player, {
    source: 'apple', current: null, queue: null, sourceContexts: new Map(), playToken: 0, queueSerial: 0, nextSerial: 0,
    shuffle: false, repeat: 'off', root: { hidden: true }, unsupportedListeners: new Set(),
    audio: { paused: true, currentTime: 0, duration: 269, async play() { this.paused = false; }, pause() { this.paused = true; }, load() {}, addEventListener() {}, getAttribute() { return this.src || null; } },
  });
  player.clock = player.audio;
  for (const name of ['closeQualityPanel', 'layout', 'emit', 'renderQueue', 'renderModes', 'setLoading', 'renderToggle', 'renderTrackText', 'renderArtwork', 'renderMode', 'updateMediaSession', 'renderProgress', 'prepareNext', 'extendQueue', 'show']) player[name] = () => {};
  player.showError = (error) => { player.error = error; };
  player.teardown = () => { player.audio.pause(); };
  return player;
}
test('QQ plays the local 302 route through native audio without asking Apple for a master', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(qq.toTrack(dto))]);
  assert.equal(player.current.source, 'qq');
  assert.equal(player.current.mode, 'native');
  assert.equal(player.current.key, `qq:${dto.id}`);
  assert.equal(player.current.artwork, 'https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg');
  assert.equal(player.queue.entries[0].artwork, player.current.artwork);
  assert.equal(player.audio.src, `/qq/song/${dto.id}/play?quality=HQ`);
  assert.equal(player.audio.paused, false);
});

test('QQ native play rejection advances through the planned mode list', { timeout: 10000 }, async () => {
  const player = playerHarness();
  const attempted = [];
  player.tryMode = async (mode) => {
    attempted.push(mode);
    if (mode === 'native') throw new Error('unsupported native source');
  };
  await player.start(qq.qualityEntry(qq.toTrack(dto), 'DAEAC3'), { modes: ['native', 'joc2'] });
  assert.deepEqual(attempted, ['native', 'joc2']);
  assert.equal(player.current.mode, 'joc2');
});

test('QQ native media error hands off with the live paused state and preserves the queue', { timeout: 10000 }, async () => {
  const player = playerHarness();
  const attempted = [];
  const item = qq.qualityEntry(qq.toTrack(dto), 'DAEAC3');
  const next = qq.entryOf(qq.toTrack({ id: 'next' }));
  player.tryMode = async (mode, _item, resumeAt, _token, paused) => {
    attempted.push({ mode, resumeAt, paused });
    if (mode === 'native') player.audio.paused = false;
  };
  await player.start(item, { modes: ['native', 'joc2'] });
  player.queue = { entries: [item, next], pos: 0 };
  player.audio.currentTime = 18;
  player.audio.paused = true; // changed after native startup: handoff must read this value
  player.audio.error = { code: 4 }; // MEDIA_ERR_SRC_NOT_SUPPORTED
  player.handleAudioError();
  await new Promise(setImmediate);
  assert.deepEqual(attempted, [{ mode: 'native', resumeAt: 0, paused: false }, { mode: 'joc2', resumeAt: 18, paused: true }]);
  assert.equal(player.current.mode, 'joc2');
  assert.equal(player.current.quality, 'DAEAC3');
  assert.deepEqual(player.queue.entries.map((entry) => entry.track), [dto.id, 'next']);
});

test('QQ native fallback does not react to a network error after native startup', { timeout: 10000 }, async () => {
  const player = playerHarness();
  const attempted = [];
  player.tryMode = async (mode) => { attempted.push(mode); };
  await player.start(qq.qualityEntry(qq.toTrack(dto), 'DAEAC3'), { modes: ['native', 'joc2'] });
  assert.deepEqual(attempted, ['native']);
  player.audio.error = { code: 2 }; // MEDIA_ERR_NETWORK must not restart another mode
  player.handleAudioError();
  await new Promise(setImmediate);
  assert.deepEqual(attempted, ['native']);
});

test('QQ native startup hands off on a media error while play is pending', { timeout: 10000 }, async () => {
  const player = playerHarness();
  const item = qq.qualityEntry(qq.toTrack(dto), 'DAEAC3');
  const realTryMode = player.tryMode.bind(player);
  player.audio.play = () => new Promise(() => {});
  player.tryMode = (mode, ...args) => mode === 'native' ? realTryMode(mode, ...args) : Promise.resolve();
  const pending = player.start(item, { modes: ['native', 'joc2'] });
  await new Promise(setImmediate);
  assert.equal(player.current.mode, 'native');
  assert.equal(qq.playbackInfo('DAEAC3', { codec: 'ec-3' }).contentType, 'audio/mp4; codecs="ec-3"');
  player.audio.error = { code: 4 };
  player.handleAudioError();
  await pending;
  assert.equal(player.current.mode, 'joc2');
});

test('QQ native handoff probes a missing JOC content type from its EC-3 codec', () => {
  const calls = [];
  const previous = { media: globalThis.MediaSource, managed: globalThis.ManagedMediaSource };
  globalThis.ManagedMediaSource = undefined;
  globalThis.MediaSource = { isTypeSupported: (mime) => { calls.push(mime); return true; } };
  try {
    const item = { ...qq.qualityItem(qq.toTrack(dto), 'DAEAC3'), contentType: undefined };
    assert.equal(item.codec, 'ec-3');
    assert.equal(item.contentType, undefined);
    assert.deepEqual(detectQqModes(item), ['native']);
    assert.deepEqual(calls, ['audio/mp4; codecs="ec-3"']);
  } finally {
    globalThis.MediaSource = previous.media;
    if (previous.managed) globalThis.ManagedMediaSource = previous.managed; else delete globalThis.ManagedMediaSource;
  }
});

test('QQ lazily resolves defaults per track; changing the setting preserves current and manual quality', async () => {
  const original = globalThis.fetch;
  let preference = 'hires';
  const calls = [];
  usePlaybackSettings({ get: () => ({ quality: preference }) });
  globalThis.fetch = async (path) => {
    calls.push(path);
    return { ok: true, json: async () => ({ qualities: ['RS', 'SQ', 'HQ', 'WEB'].map((quality) => ({ quality, available: true, sizeBytes: 100 })) }) };
  };
  try {
    const player = playerHarness();
    const track = qq.toTrack(dto);
    await player.playQueue([qq.entryOf(track), qq.entryOf(qq.toTrack({ id: 'next' }))]);
    assert.equal(player.current.quality, 'RS');
    preference = 'high';
    assert.equal(player.current.quality, 'RS');
    await player.play(qq.qualityItem(track, 'SQ'));
    await player.playAt(0);
    assert.equal(player.current.quality, 'SQ');
    assert.equal(calls.length, 1);
    await player.playAt(1);
    assert.equal(player.current.quality, 'HQ');
    assert.equal(calls.length, 2);
  } finally { globalThis.fetch = original; usePlaybackSettings(null); }
});

test('QQ lookup failure retains HQ, but a successful empty qualities response does not invent a playable tier', async () => {
  const original = globalThis.fetch;
  try {
    globalThis.fetch = async () => { throw new Error('offline'); };
    const player = playerHarness();
    await player.playQueue([qq.entryOf(qq.toTrack(dto))]);
    assert.equal(player.current.quality, 'HQ');
    assert.equal(player.current.variant.quality, 'HQ');
    assert.equal(player.audio.src, qq.playUrl(dto.id, 'HQ'));
    assert.equal(qq.qualityItem(qq.toTrack(dto)).quality, 'HQ');
    assert.equal(qq.qualityEntry(qq.toTrack(dto)).quality, 'HQ');
    globalThis.fetch = async () => ({ ok: true, json: async () => ({ qualities: [] }) });
    const empty = playerHarness();
    await empty.playQueue([qq.entryOf(qq.toTrack(dto))]);
    assert.equal(empty.current, null);
    assert.equal(typeof empty.error, 'function');
  } finally { globalThis.fetch = original; }
});

test('a manual QQ selection during the qualities lookup cannot be replaced or paused by the pending default', async () => {
  const original = globalThis.fetch;
  let finish;
  let requested;
  const requestStarted = new Promise((resolve) => { requested = resolve; });
  globalThis.fetch = () => new Promise((resolve) => { finish = resolve; requested(); });
  try {
    const player = playerHarness();
    const track = qq.toTrack(dto);
    const pending = player.playQueue([qq.entryOf(track)]);
    await requestStarted;
    await player.play(qq.qualityItem(track, 'HQ'));
    finish({ ok: true, json: async () => ({ qualities: [{ quality: 'RS', available: true, sizeBytes: 100 }] }) });
    await pending;
    assert.equal(player.current.quality, 'HQ');
    assert.equal(player.audio.paused, false);
    assert.equal(player.pendingKey, null);
    assert.equal(player.queue.entries[0].quality, 'HQ');
  } finally { globalThis.fetch = original; }
});

test('Apple defaults use existing capability filtering, respect preference with JOC enabled, and leave manual playback intact', async () => {
  const saved = { wrapper: globalThis.AmWrapper, decrypt: globalThis.AmDecrypt, media: globalThis.MediaSource, worker: globalThis.Worker };
  const variants = [
    { group_id: 'audio-alac-stereo-96000-24', codecs: 'alac', sample_rate: 96000, bandwidth: 4000000 },
    { group_id: 'audio-alac-stereo-48000-24', codecs: 'alac', sample_rate: 48000, bandwidth: 2000000 },
    { group_id: 'audio-atmos-2768', codecs: 'ec-3', bandwidth: 768000 },
    { group_id: 'audio-stereo-256', codecs: 'mp4a.40.2', bandwidth: 256000 },
  ];
  let preference = 'hires';
  globalThis.AmWrapper = { songMaster: async () => ({ variants }) };
  globalThis.AmDecrypt = { supported: () => true };
  globalThis.MediaSource = { isTypeSupported: () => true };
  usePlaybackSettings({ get: () => ({ quality: preference }) });
  useJocSettings({ get: () => ({ mode: 'speaker' }) });
  try {
    const player = playerHarness();
    player.start = async (item) => { player.current = item; };
    await player.playQueue([{ track: '123', name: 'Apple song' }]);
    assert.equal(player.current.variant, variants[0]);
    preference = 'dolby';
    assert.equal(player.current.variant, variants[0]);
    await player.playQueue([{ track: '123', name: 'Apple song' }]);
    assert.equal(player.current.variant, variants[2]);
    // EC-3 has neither MSE nor PCM; the filtered candidates fall back to AAC.
    globalThis.MediaSource = { isTypeSupported: (mime) => mime.includes('mp4a') };
    globalThis.Worker = undefined;
    await player.playQueue([{ track: '123', name: 'Apple song' }]);
    assert.equal(player.current.variant, variants[3]);
    const previous = player.current;
    globalThis.AmDecrypt = { supported: () => false };
    await player.playQueue([{ track: '123', name: 'Apple song' }]);
    assert.equal(player.current, previous);
    assert.equal(typeof player.error, 'function');
    const manual = { track: '123', id: '123:manual', variant: variants[0], codecs: 'alac' };
    await player.play(manual);
    assert.equal(player.current.variant, variants[0]);
  } finally {
    globalThis.AmWrapper = saved.wrapper;
    globalThis.AmDecrypt = saved.decrypt;
    globalThis.MediaSource = saved.media;
    globalThis.Worker = saved.worker;
    usePlaybackSettings(null);
    useJocSettings(null);
  }
});

test('every native QQ quality reaches the play URL and matching playback label', async () => {
  const track = qq.toTrack(dto);
  for (const quality of qq.QUALITY_IDS.filter((quality) => qq.playbackInfo(quality).playback === 'native')) {
    const player = playerHarness();
    await player.playQueue([qq.qualityEntry(track, quality)]);
    assert.equal(player.current.quality, quality);
    assert.equal(player.current.id, `qq:${dto.id}:${quality}`);
    assert.equal(player.audio.src, qq.playUrl(dto.id, quality));
    assert.equal(player.modeText(), `QQ · ${quality}`);
    assert.equal(player.current.href, qq.songPath(dto.id));
    assert.equal(player.current.artwork, track.artwork.url);
    assert.equal(player.queue.entries[0].artwork, track.artwork.url);
    assert.equal(player.queue.entries[0].quality, quality);
  }
});

test('changing QQ quality on its detail page preserves the remaining queue and repeats that payload', async () => {
  const player = playerHarness();
  const track = qq.toTrack(dto);
  const next = qq.entryOf(qq.toTrack({ id: 'next', name: 'Next' }));
  await player.playQueue([qq.entryOf(track), next]);
  const queue = player.queue;
  await player.play(qq.qualityItem(track, 'Q360V1', { codec: 'vorbis', contentType: 'audio/ogg' }));
  assert.equal(player.audio.src, qq.playUrl(dto.id, 'Q360V1'));
  assert.equal(player.queue, queue);
  assert.equal(queue.entries[1].track, 'next');
  assert.equal(queue.entries[0].quality, 'Q360V1');
  await player.playAt(0);
  assert.equal(player.current.quality, 'Q360V1');
  assert.equal(player.current.codec, 'vorbis');
  assert.equal(player.current.contentType, 'audio/ogg');
  assert.equal(player.current.playback, 'native');
});

test('a first queued QQ song is loaded paused and a saved pending queue is accessible on switching source', async () => {
  const player = playerHarness();
  player.source = 'qq';
  player.appendQueue(qq.entryOf(qq.toTrack(dto)));
  await new Promise(setImmediate);
  assert.equal(player.current.track, dto.id);
  assert.equal(player.audio.paused, true);
  assert.equal(player.queue.pos, 0);
  assert.equal(player.queue.entries.length, 1);

  const other = playerHarness();
  other.appendQueue(qq.qualityEntry(qq.toTrack(dto), 'AAC48'));
  assert.equal(other.current, null);
  other.switchSource('qq');
  await new Promise(setImmediate);
  assert.equal(other.current.quality, 'AAC48');
  assert.equal(other.audio.paused, true);
  assert.equal(other.audio.src, qq.playUrl(dto.id, 'AAC48'));
});

test('adding QQ to its saved queue preserves Apple playback and source restoration retains quality', async () => {
  const player = playerHarness();
  const appleQueue = { entries: [{ track: '123' }], pos: 0 };
  player.current = { track: '123', id: '123:stereo', source: 'apple' };
  player.queue = appleQueue;
  player.audio.paused = false;
  const entry = qq.qualityEntry(qq.toTrack(dto), 'SQ');
  player.appendQueue(entry);
  assert.equal(player.source, 'apple');
  assert.equal(player.queue, appleQueue);
  assert.equal(player.current.track, '123');
  assert.equal(player.audio.paused, false);
  assert.equal(player.sourceContexts.get('qq').queue.entries[0].quality, 'SQ');
  await player.playQueue([entry]);
  player.audio.currentTime = 23;
  player.switchSource('apple');
  player.switchSource('qq');
  await new Promise(setImmediate);
  assert.equal(player.current.quality, 'SQ');
  assert.equal(player.current.label, 'QQ · SQ');
  assert.equal(player.audio.src, qq.playUrl(dto.id, 'SQ'));
  assert.equal(player.audio.paused, true);
});
test('source switching saves each queue, position and progress; mixed input is filtered', async () => {
  const player = playerHarness();
  const appleQueue = { entries: [{ track: '123' }], pos: 0 };
  player.queue = appleQueue;
  player.current = { track: '123', id: '123:stereo', source: 'apple' };
  player.audio.currentTime = 42;
  await player.playQueue([{ track: '123' }, qq.entryOf(qq.toTrack({ id: '123' }))], 1);
  assert.deepEqual(player.queue.entries.map((entry) => entry.key), ['qq:123']);
  const qqQueue = player.queue;
  player.audio.currentTime = 18;
  const starts = [];
  player.start = (item, options) => { starts.push({ item, options }); player.current = item; };
  player.switchSource('apple');
  assert.equal(player.queue, appleQueue);
  assert.equal(starts[0].options.resumeAt, 42);
  assert.equal(starts[0].options.paused, true);
  player.switchSource('qq');
  assert.equal(player.queue, qqQueue);
  assert.equal(starts[1].options.resumeAt, 18);
  assert.equal(starts[1].item.key, 'qq:123');
});

test('restoring a native QQ context loads metadata without autoplay', async () => {
  const player = playerHarness();
  let plays = 0;
  player.audio.play = async () => { plays++; };
  await player.start({ ...qq.entryOf(qq.toTrack(dto)), id: 'qq:test:HQ', streamUrl: qq.playUrl(dto.id) }, { paused: true, resumeAt: 12 });
  assert.equal(plays, 0);
  assert.equal(player.current.source, 'qq');
  assert.equal(player.audio.src, qq.playUrl(dto.id));
});

test('starting old QQ entries rebuilds the blob artwork URL while retaining the display URL', async () => {
  for (const album of [{ qq: { albumMid: dto.albumMid } }, { albumId: dto.albumMid }, {}]) {
    const player = playerHarness();
    const artwork = qq.artworkUrl(dto.cover);
    await player.start({ source: 'qq', track: dto.id, id: `qq:${dto.id}:HQ`, ...album,
      artwork, streamUrl: qq.playUrl(dto.id), quality: 'HQ', ...qq.playbackInfo('HQ') }, { paused: true });
    assert.equal(player.current.artwork, artwork);
    assert.equal(player.current.artworkFetchUrl, album.qq?.albumMid ? `/qq/cover/${album.qq.albumMid}` : '');
  }
});

test('switching source invalidates a pending Apple resolution before it can replace QQ context', async () => {
  const player = playerHarness();
  const original = globalThis.AmWrapper;
  let finish;
  globalThis.AmWrapper = { songMaster: () => new Promise((resolve) => { finish = resolve; }) };
  try {
    const pending = player.playQueue([{ track: '123', name: 'Apple song' }]);
    assert.equal(player.pendingKey, 'apple:123');
    player.switchSource('qq');
    finish({ variants: [] });
    await pending;
    assert.equal(player.source, 'qq');
    assert.equal(player.current, null);
    assert.equal(player.pendingKey, null);
    assert.equal(player.queue, null);
  } finally { globalThis.AmWrapper = original; }
});


test('known MIME codecs use runtime capability detection; only DAEAC3 among Dolby tiers selects JOC', () => {
  for (const quality of ['AAC48', 'WEB', 'AAC192', 'OGG96', 'OGG192', 'QAI', 'Q360V1', 'Q360V2']) {
    assert.equal(qq.playbackInfo(quality).playback, 'native', quality);
  }
  for (const [quality, playback, codec] of [
    ['DA', 'native', 'ac-4'], ['DA448', 'native', 'ac-4'], ['DA1500', 'native', 'ac-4'], ['DAEAC3', 'joc', 'ec-3'],
  ]) {
    for (const row of [{}, { playback: 'joc', codec: 'ec-3' }, { playback: 'native', codec: 'flac' }, { codec: 'ac-4' }]) {
      const info = qq.playbackInfo(quality, row);
      assert.equal(info.playback, playback, quality);
      assert.equal(info.codec, codec, quality);
      assert.equal(info.contentType, `audio/mp4; codecs="${codec}"`);
    }
  }
  for (const quality of ['NAC', 'DTS', 'DTSX', 'RA360']) {
    assert.equal(qq.playbackInfo(quality, { codec: 'flac' }).playback, 'download', quality);
  }
  assert.equal(qq.playbackInfo('Q360V1', { codec: 'ac-4' }).playback, 'native');
  assert.equal(qq.playbackInfo('QAI', { codec: 'ec-3' }).playback, 'joc');
});

test('AC-4 playbackInfo follows injected browser capability, never JOC, and is download-only without a browser', () => {
  const env = (supported) => ({ window: {}, MediaSource: { isTypeSupported: () => supported },
    document: { createElement: () => ({ canPlayType: () => '' }) } });
  for (const quality of ['DA', 'DA448', 'DA1500']) {
    assert.equal(qq.playbackInfo(quality, {}, env(true)).contentType, 'audio/mp4; codecs="ac-4"');
    assert.equal(qq.playbackInfo(quality, {}, env(true)).playback, 'native');
    assert.equal(qq.playbackInfo(quality, {}, env(false)).playback, 'download');
    assert.equal(qq.playbackInfo(quality, {}, {}).playback, 'download');
  }
});

test('shared QQ MIME detection prefers ManagedMediaSource with the same player probe', () => {
  let managedCalls = 0;
  const previous = { managed: globalThis.ManagedMediaSource, media: globalThis.MediaSource };
  const environment = { window: {}, ManagedMediaSource: { isTypeSupported: () => { managedCalls++; return true; } },
    MediaSource: { isTypeSupported: () => false }, document: { createElement: () => ({ canPlayType: () => '' }) } };
  try {
    assert.equal(qq.playbackInfo('DA', {}, environment).playback, 'native');
    assert.equal(managedCalls, 1);
    globalThis.ManagedMediaSource = environment.ManagedMediaSource;
    globalThis.MediaSource = environment.MediaSource;
    qq.clearMimeSupportCache();
    assert.deepEqual(detectQqModes(qq.playbackInfo('DA'), { canPlayType: () => '' }), ['native']);
  } finally {
    if (previous.managed) globalThis.ManagedMediaSource = previous.managed; else delete globalThis.ManagedMediaSource;
    globalThis.MediaSource = previous.media;
  }
});

test('QQ rows keep downloads when codecs or browser capabilities prevent playback', () => {
  globalThis.MediaSource = { isTypeSupported: () => false };
  const data = { qualities: ['NAC', 'DTS', 'DTSX', 'RA360', 'DA', 'DA448', 'DA1500', 'OGG96', 'AAC48', 'QAI'].map((quality) => ({
    quality, available: true, sizeBytes: 1024,
  })) };
  const rows = qq.qualityRows(data, dto.id, (info) => detectQqModes(info, { canPlayType: (mime) => mime.startsWith('audio/mp4') ? 'maybe' : '' }));
  for (const row of rows.filter((row) => row.available)) {
    assert.ok(row.downloadUrl);
    assert.equal(row.playable, ['AAC48', 'DA', 'DA448', 'DA1500'].includes(row.quality), row.quality);
    assert.equal(Object.hasOwn(row, 'playUrl'), row.playable);
  }
});

test('QQ JOC mode planning follows the JOC setting, then native passthrough, then PCM fallback', () => {
  const saved = { Worker: globalThis.Worker, AudioContext: globalThis.AudioContext, AmDecrypt: globalThis.AmDecrypt,
    MediaSource: globalThis.MediaSource };
  globalThis.Worker = function Worker() {};
  globalThis.AudioContext = function AudioContext() {};
  globalThis.AmDecrypt = undefined;
  try {
    for (const [supported, expected] of [[true, {
      binaural: ['jocbin', 'joc2', 'native'], speaker: ['joc2', 'native'], off: ['native', 'joc2'],
    }], [false, {
      binaural: ['jocbin', 'joc2'], speaker: ['joc2'], off: ['joc2'],
    }]]) {
      globalThis.MediaSource = { isTypeSupported: () => supported };
      for (const mode of ['binaural', 'speaker', 'off']) {
        useJocSettings({ get: () => ({ mode }) });
        assert.deepEqual(detectQqModes(qq.playbackInfo('DAEAC3')), expected[mode], `${supported ? 'native' : 'PCM'} ${mode}`);
      }
    }
    for (const quality of ['DA', 'DA448', 'DA1500']) assert.deepEqual(detectQqModes(qq.playbackInfo(quality)), ['native']);
    globalThis.MediaSource = { isTypeSupported: () => true };
    globalThis.Worker = undefined;
    useJocSettings({ get: () => ({ mode: 'speaker' }) });
    assert.deepEqual(detectQqModes(qq.playbackInfo('DAEAC3')), ['native']);
  } finally { Object.assign(globalThis, saved); useJocSettings(null); }
});

test('download-only QQ entries cannot start audio', async () => {
  for (const quality of ['NAC', 'DTS', 'DTSX', 'RA360']) {
    const player = playerHarness();
    await player.playQueue([qq.qualityEntry(qq.toTrack(dto), quality)]);
    assert.equal(player.current, null);
    assert.equal(player.audio.src, undefined);
    assert.ok(player.error);
  }
});
