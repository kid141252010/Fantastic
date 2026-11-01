// Offline panel, lazy variants and transport regressions: node --test tests/player_quality.mjs
import test, { beforeEach, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import * as qq from '../src/ui/sources/qq.mjs';
const { AmPlayer, qualityBadge, variantLabel, useJocSettings } = createRequire(import.meta.url)('../src/ui/player.js');

const original = Object.fromEntries(['fetch', 'AmWrapper', 'AmDecrypt', 'MediaSource', 'Worker', 'AudioContext', 'window', 'document', 'AmI18n']
  .map((key) => [key, globalThis[key]]));
beforeEach(() => {
  qq.clearMimeSupportCache();
  globalThis.AmWrapper = { songMaster() { assert.fail('QQ must not request an Apple master'); } };
  globalThis.AmDecrypt = undefined;
  globalThis.window = {};
  globalThis.document = { createElement: () => ({ canPlayType: () => 'probably' }) };
  globalThis.MediaSource = { isTypeSupported: () => true };
  globalThis.fetch = async () => { throw new Error('offline'); };
});
afterEach(() => { Object.assign(globalThis, original); qq.clearMimeSupportCache(); useJocSettings(null); });

// Only DOM primitives used by the production quality panel; all rendering and events are real.
class Element {
  constructor(tag) { this.tag = tag; this.children = []; this.attributes = {}; this.events = {}; }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = children; }
  set textContent(value) { this.children = [value]; }
  get textContent() { return this.children.map((child) => typeof child === 'string' ? child : child.textContent).join(''); }
  setAttribute(key, value) { this.attributes[key] = value; }
  getAttribute(key) { return this.attributes[key] ?? null; }
  removeAttribute(key) { delete this.attributes[key]; }
  addEventListener(type, fn) { this.events[type] = fn; }
  insertAdjacentHTML(_position, html) { this.check = html; }
  all(className) {
    return this.children.filter((child) => typeof child !== 'string').flatMap((child) => [
      ...(child.className === className ? [child] : []), ...child.all(className),
    ]);
  }
}

const track = (id = 'song') => qq.toTrack({ id, name: id, artists: ['Artist'] });
function playerHarness() {
  const player = Object.create(AmPlayer.prototype);
  const badge = new Element('button');
  Object.assign(player, {
    source: 'qq', current: null, queue: null, playToken: 0, queueSerial: 0, nextSerial: 0,
    shuffle: false, variantCache: new Map(), unsupportedListeners: new Set(),
    root: { ownerDocument: { createElement: (tag) => new Element(tag) } },
    qualityPanel: Object.assign(new Element('div'), { hidden: true }),
    $: (selector) => { assert.equal(selector, '.player-mode'); return badge; },
    audio: { paused: true, currentTime: 0, duration: 240, canPlayType: () => 'probably',
      async play() { this.paused = false; }, pause() { this.paused = true; } },
  });
  player.clock = player.audio;
  for (const name of ['layout', 'emit', 'renderQueue', 'setLoading', 'renderTrackText', 'renderArtwork',
    'updateMediaSession', 'renderProgress', 'prepareNext', 'extendQueue', 'show', 'placeQualityPanel']) player[name] = () => {};
  player.showError = (error) => { player.error = error; };
  player.teardown = () => player.audio.pause();
  return player;
}
const payload = { qualities: [
  { quality: 'HQ', available: true, sizeBytes: 100, format: 'MP3 320 kbps' },
  { quality: 'SQ', available: true, sizeBytes: 200, format: 'FLAC 16-bit / 44.1 kHz' },
  { quality: 'Q360V1', available: true, sizeBytes: 300, format: 'Ogg Vorbis', codec: 'vorbis', contentType: 'audio/ogg' },
  { quality: 'DAEAC3', available: true, sizeBytes: 400, format: 'E-AC-3 JOC' },
  { quality: 'RS', available: false, sizeBytes: 0, format: 'unavailable' },
  { quality: 'DA', available: true, sizeBytes: 500, format: 'AC-4' },
  { quality: 'NAC', available: true, sizeBytes: 600, format: 'NAC' },
] };
const variants = () => qq.qualityVariants(qq.qualityRows(payload, 'song'));
const settle = () => new Promise(setImmediate);

test('QQ variants retain available native/JOC formats and actual payload metadata only', () => {
  const list = variants();
  assert.deepEqual(list.map((v) => v.quality), ['HQ', 'SQ', 'Q360V1', 'DA', 'DAEAC3']);
  assert.ok(list.every((v) => v.group_id === v.quality && v.source === 'qq'));
  assert.deepEqual(list.map((v) => v.playback), ['native', 'native', 'native', 'native', 'joc']);
  assert.equal(list[1].format, 'FLAC 16-bit / 44.1 kHz');
  assert.equal(list[2].codec, 'vorbis');
  assert.equal(list[2].contentType, 'audio/ogg');
  assert.equal(qualityBadge(list[1]), 'lossless');
  assert.equal(qualityBadge(list[3]), 'atmos');
  assert.deepEqual(qq.qualityVariants([{ quality: 'HQ', available: true, playback: 'download' }]), []);
});

test('QQ player labels reuse localized quality names and fall back to the code', () => {
  globalThis.AmI18n = { t: (key) => key === 'qq.quality.SQ.label' ? 'Localized lossless' : key };
  assert.equal(variantLabel({ source: 'qq', quality: 'SQ' }), 'QQ · Localized lossless');
  assert.equal(variantLabel({ source: 'qq', quality: 'DAEAC3' }), 'QQ · DAEAC3');
  globalThis.AmI18n = { t: () => '' };
  assert.equal(variantLabel({ source: 'qq', quality: 'HQ' }), 'QQ · HQ');
});

test('QQ badge opens the shared panel, fetches once and marks the current quality with its format', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(track())]);
  const calls = [];
  globalThis.fetch = async (path) => { calls.push(path); return { ok: true, json: async () => payload }; };
  const badge = player.$('.player-mode');
  assert.equal(badge.disabled, false);
  assert.equal(badge.textContent, 'QQ · HQ');
  player.openQualityPanel();
  player.renderMode();
  assert.equal(player.qualityPanel.hidden, false);
  assert.equal(badge.getAttribute('aria-expanded'), 'true');
  assert.match(player.qualityPanel.textContent, /player.qualitiesLoading/);
  await settle();
  assert.deepEqual(calls, ['/qq/song/song/qualities']);
  const options = player.qualityPanel.all('quality-option');
  assert.equal(options.length, 5);
  assert.equal(options[0].getAttribute('aria-current'), 'true');
  assert.ok(options[0].check);
  assert.match(options[1].textContent, /QQ · SQFLAC 16-bit \/ 44.1 kHz/);
  assert.equal(options[0].disabled, false); // No AmDecrypt / MSE needed for native QQ.
  assert.equal(options[4].disabled, false); // Native EC-3 passthrough does not require the JOC renderer.
  assert.equal(player.current.variants, player.variantCache.get('qq:song').variants);
  player.closeQualityPanel();
  player.openQualityPanel();
  assert.deepEqual(calls, ['/qq/song/song/qualities']);
});

test('QQ panel retries a failed qualities request when reopened and caches empty results', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(track())]);
  let requests = 0;
  globalThis.fetch = async () => ++requests === 1 ? { ok: false, status: 503 }
    : { ok: true, json: async () => ({ qualities: [] }) };
  player.openQualityPanel();
  await settle();
  assert.match(player.qualityPanel.textContent, /player.qualitiesFailed/);
  assert.equal(player.variantCache.get('qq:song').error, 'QQ HTTP 503');
  player.renderMode();
  assert.equal(requests, 1);
  player.closeQualityPanel();
  player.openQualityPanel();
  await settle();
  assert.deepEqual(player.current.variants, []);
  assert.equal(player.qualityPanel.all('quality-option').length, 0);
  player.closeQualityPanel();
  player.openQualityPanel();
  assert.equal(requests, 2);
});

test('QQ default quality surfaces a 429 immediately without retrying or starting HQ', async () => {
  const player = playerHarness();
  let requests = 0;
  globalThis.fetch = async () => {
    requests++;
    return { ok: false, status: 429 };
  };
  await player.playQueue([qq.entryOf(track())]);
  assert.equal(requests, 1);
  assert.equal(player.current, null);
  assert.equal(player.audio.src, undefined);
  assert.equal(typeof player.error, 'function');
});

test('panel selection resumes native QQ at the same time and retains queue order, repeat payload and selection', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(track()), qq.entryOf(track('next'))]);
  player.current.variants = variants();
  player.queue.ordered = player.queue.entries.slice();
  const queue = player.queue;
  const ordered = queue.ordered;
  player.audio.currentTime = 68.5;
  player.openQualityPanel();
  await player.qualityPanel.all('quality-option')[1].events.click();
  assert.equal(player.queue, queue);
  assert.equal(queue.ordered, ordered);
  assert.equal(queue.pos, 0);
  assert.equal(queue.entries[1].track, 'next');
  assert.equal(player.current.id, 'qq:song:SQ');
  assert.equal(player.audio.src, '/qq/song/song/play?quality=SQ');
  assert.equal(player.audio.currentTime, 68.5);
  assert.equal(queue.entries[0].quality, 'SQ');
  assert.equal(player.$('.player-mode').textContent, 'QQ · SQ');
  assert.equal(player.qualityPanel.all('quality-option')[1].getAttribute('aria-current'), 'true');
  await player.switchVariant(player.current.variant);
  assert.equal(player.audio.paused, false); // Selecting the checked row must not toggle playback.
  await player.playAt(1);
  assert.equal(player.current.quality, 'HQ');
  await player.playAt(0);
  assert.equal(player.current.quality, 'SQ');
  assert.equal(player.current.variant.format, 'FLAC 16-bit / 44.1 kHz');
});

test('switching a QQ variant uses its API codec override and rejects unsupported choices', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(track())]);
  await player.switchVariant(variants().find((variant) => variant.quality === 'Q360V1'));
  assert.equal(player.current.codec, 'vorbis');
  assert.equal(player.current.contentType, 'audio/ogg');
  assert.equal(player.audio.src, '/qq/song/song/play?quality=Q360V1');
  const current = player.current;
  await player.switchVariant(qq.qualityVariant('NAC'));
  assert.equal(player.current, current);
});

test('QQ and Apple lazy caches stay separate for the same ID, and a late QQ result cannot replace Apple panel data', async () => {
  const player = playerHarness();
  await player.playQueue([qq.entryOf(track())]);
  let finish;
  globalThis.fetch = async () => ({ ok: true, json: () => new Promise((resolve) => { finish = resolve; }) });
  player.openQualityPanel();
  await settle();
  const apple = { track: 'song', id: 'song:audio-stereo-256', codecs: 'mp4a.40.2', mode: 'mse' };
  const appleVariants = [{ group_id: 'audio-stereo-256', codecs: 'mp4a.40.2', bandwidth: 256000 }];
  globalThis.AmWrapper = { songMaster: async () => ({ variants: appleVariants }) };
  globalThis.MediaSource = { isTypeSupported: () => true };
  globalThis.AmDecrypt = { supported: () => true };
  player.current = apple;
  player.renderQualityPanel();
  await settle();
  finish(payload);
  await settle();
  assert.equal(player.variantCache.size, 2);
  assert.equal(player.variantsOf(apple).variants, appleVariants);
  assert.equal(apple.variants, undefined);
  assert.equal(player.qualityPanel.all('quality-option').length, 1);
  assert.match(player.qualityPanel.textContent, /AAC · 256 kbps/);
  assert.equal(player.qualityPanel.all('quality-option')[0].disabled, false);
});

test('Apple inline variants keep their badge, panel and MSE switching behavior without changing QQ selection', async () => {
  const player = playerHarness();
  globalThis.MediaSource = { isTypeSupported: () => true };
  globalThis.AmDecrypt = { supported: () => true };
  const appleVariants = [
    { group_id: 'audio-stereo-256', codecs: 'mp4a.40.2', bandwidth: 256000, url: 'aac.m3u8' },
    { group_id: 'audio-alac-stereo-48000-24', codecs: 'alac', sample_rate: 48000, bit_depth: 24, url: 'alac.m3u8' },
    { group_id: 'audio-unsupported', codecs: 'unsupported', url: 'unsupported.m3u8' },
  ];
  globalThis.MediaSource.isTypeSupported = (mime) => !mime.includes('unsupported');
  const requests = [];
  player.mse = { destroy() {}, async load(url, codecs) { requests.push({ url, codecs }); this.playlist = { duration: 240 }; } };
  player.source = 'apple';
  player.current = { source: 'apple', track: 'song', id: 'song:audio-stereo-256', mode: 'mse',
    codecs: 'mp4a.40.2', variant: appleVariants[0], variants: appleVariants, badge: 'lossy' };
  player.queue = { entries: [{ track: 'song' }, { track: 'next' }], pos: 0 };
  const queue = player.queue;
  player.audio.currentTime = 32.5;
  player.renderMode();
  assert.match(player.$('.player-mode').getAttribute('aria-label'), /AAC/);
  player.openQualityPanel();
  assert.equal(player.variantCache.size, 0);
  const options = player.qualityPanel.all('quality-option');
  assert.equal(options.length, 3);
  assert.equal(options.find((option) => option.textContent.includes('unsupported')).disabled, true);
  await options.find((option) => option.textContent.includes('ALAC')).events.click();
  assert.deepEqual(requests, [{ url: 'alac.m3u8', codecs: 'alac' }]);
  assert.equal(player.current.id, 'song:audio-alac-stereo-48000-24');
  assert.equal(player.current.badge, 'lossless');
  assert.equal(player.current.mode, 'mse');
  assert.equal(player.audio.currentTime, 32.5);
  assert.equal(player.queue, queue);
  assert.equal(player.current.variants, appleVariants);
});
