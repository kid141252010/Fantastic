// QQ containers, the real JOC Worker's packet contract, and player input injection; offline.
import test, { afterEach, beforeEach } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { qqJocTrack, openQqJocTrack } from '../../web/qq-joc-input.mjs';
import * as qq from '../../web/sources/qq.mjs';
const { AmPlayer, JocEngine, JocLiveEngine } = createRequire(import.meta.url)('../../web/player/player.js');
const fixture = new Uint8Array(fs.readFileSync(new URL('../../crates/joc-core/tests/fixtures/joc_fixture.eac3', import.meta.url)));
const worker = fs.readFileSync(new URL('../../web/joc/decode-worker.js', import.meta.url), 'utf8');
// Run the production packet extractor, so changes to the Worker contract are covered.
const samplesFromFragment = vm.runInNewContext(`${worker.slice(worker.indexOf('function boxes('), worker.indexOf('async function loadEc3('))}\nsamplesFromFragment`, { Uint8Array, DataView });
const originalFetch = Object.getOwnPropertyDescriptor(globalThis, 'fetch');
beforeEach(() => {});
afterEach(() => {
  if (originalFetch) Object.defineProperty(globalThis, 'fetch', originalFetch); else delete globalThis.fetch;
});
const concat = (...parts) => { const out = new Uint8Array(parts.reduce((n, part) => n + part.length, 0)); let at = 0; for (const part of parts) { out.set(part, at); at += part.length; } return out; };
const words = (...values) => { const out = new Uint8Array(values.length * 4); const view = new DataView(out.buffer); values.forEach((value, i) => view.setUint32(i * 4, value)); return out; };
const box = (name, ...parts) => concat(words(8 + parts.reduce((n, part) => n + part.length, 0)), Uint8Array.from(name, (c) => c.charCodeAt(0)), ...parts);
const packets = [];
for (let at = 0; at < fixture.length;) { const size = ((((fixture[at + 2] & 7) << 8) | fixture[at + 3]) + 1) * 2; packets.push(fixture.subarray(at, at + size)); at += size; }
function ac4FrameBytes(fsIndex = 1, frIndex = 13, payloadLen = 8, { crc = false, extended = false } = {}) {
  const payload = new Uint8Array(Math.max(3, payloadLen));
  payload[1] = 0x10 | ((fsIndex & 1) << 2) | ((frIndex >> 2) & 3);
  payload[2] = (frIndex & 3) << 6;
  const head = extended ? 7 : 4;
  const out = new Uint8Array(head + payload.length + (crc ? 2 : 0));
  out[0] = 0xac;
  out[1] = crc ? 0x41 : 0x40;
  if (extended) {
    out[2] = 0xff;
    out[3] = 0xff;
    out[4] = (payload.length >> 16) & 0xff;
    out[5] = (payload.length >> 8) & 0xff;
    out[6] = payload.length & 0xff;
  } else {
    out[2] = (payload.length >> 8) & 0xff;
    out[3] = payload.length & 0xff;
  }
  out.set(payload, head);
  return { payload, syncframe: out };
}
function mp4({ fragmented = false, codec = 'ec-3', badOffset = false, sampleList = packets, sampleDuration = 1536 } = {}) {
  const stsd = box('stsd', words(0, 1), box(codec, new Uint8Array(28)));
  const track = (tables) => box('trak', box('tkhd', words(0, 0, 0, 1)), box('mdia', box('mdhd', words(0, 0, 0, 48000)), box('minf', box('stbl', stsd, ...tables))));
  const payload = concat(...sampleList);
  if (fragmented) {
    const moov = box('moov', track([]), box('mvex', box('trex', words(0, 1, 1, sampleDuration, 0, 0))));
    const moof = (offset) => box('moof', box('traf', box('tfhd', words(0x020000, 1)), box('trun', words(0x201, sampleList.length, offset, ...sampleList.map((packet) => packet.length)))));
    return concat(moov, moof(moof(0).length + 8), box('mdat', payload));
  }
  const moov = (offset) => box('moov', track([
    box('stts', words(0, 1, sampleList.length, sampleDuration)), box('stsc', words(0, 1, 1, sampleList.length, 1)),
    box('stsz', words(0, 0, sampleList.length, ...sampleList.map((packet) => packet.length))), box('stco', words(0, 1, offset)),
  ]));
  return concat(moov(badOffset ? 1 : moov(0).length + 8), box('mdat', payload));
}

test('raw E-AC-3, standard MP4 and fragmented MP4 reach the actual Worker packet reader unchanged', async () => {
  for (const bytes of [fixture, mp4(), mp4({ fragmented: true })]) {
    const track = qqJocTrack(bytes);
    assert.equal(track.codec, 'ec-3');
    assert.ok(Math.abs(track.duration - packets.length * 1536 / 48000) < 1e-10);
    const fragment = new Uint8Array(await track.load(track.segments[0]));
    const decoded = samplesFromFragment(fragment);
    assert.equal(decoded.length, packets.length);
    decoded.forEach((packet, index) => assert.deepEqual(packet, packets[index]));
    // Transfer detaches only the generated buffer; seek can produce the segment again.
    structuredClone(fragment.buffer, { transfer: [fragment.buffer] });
    const again = samplesFromFragment(new Uint8Array(await track.load(track.segments[0])));
    assert.deepEqual(concat(...again), fixture);
  }
});

test('raw AC-4 syncframes and AC-4 MP4 containers parse with codec=ac-4 and reach the Worker unchanged', async () => {
  const f1 = ac4FrameBytes(1, 13, 12, { crc: false });
  const f2 = ac4FrameBytes(1, 13, 16, { crc: true });
  const f3 = ac4FrameBytes(1, 3, 20, { crc: false, extended: true });
  const rawAc4 = concat(f1.syncframe, f2.syncframe, f3.syncframe);
  const rawTrack = qqJocTrack(rawAc4);
  assert.equal(rawTrack.codec, 'ac-4');
  const expectedRawDuration = (2 * 2048 / 48000) + (1001 / 30000);
  assert.ok(Math.abs(rawTrack.duration - expectedRawDuration) < 1e-10);
  const rawDecoded = samplesFromFragment(new Uint8Array(await rawTrack.load(rawTrack.segments[0])));
  assert.equal(rawDecoded.length, 3);
  [f1.syncframe, f2.syncframe, f3.syncframe].forEach((expected, i) => assert.deepEqual(rawDecoded[i], expected));

  const ac4Samples = [f1.payload, f2.payload];
  for (const fragmented of [false, true]) {
    const track = qqJocTrack(mp4({ fragmented, codec: 'ac-4', sampleList: ac4Samples, sampleDuration: 2048 }));
    assert.equal(track.codec, 'ac-4');
    assert.ok(Math.abs(track.duration - (2 * 2048 / 48000)) < 1e-10);
    const decoded = samplesFromFragment(new Uint8Array(await track.load(track.segments[0])));
    assert.equal(decoded.length, ac4Samples.length);
    ac4Samples.forEach((expected, i) => assert.deepEqual(decoded[i], expected));
  }
});

test('JOC parser rejects non-JOC codecs, invalid sample ranges, truncated frames and unsupported E-AC-3 sample rates', () => {
  assert.throws(() => qqJocTrack(mp4({ codec: 'mp4a' })), /no unencrypted ec-3 or ac-4/);
  assert.throws(() => qqJocTrack(mp4({ badOffset: true })), /sample range/);
  assert.throws(() => qqJocTrack(fixture.subarray(0, fixture.length - 1)), /truncated/);
  const rawAc4 = ac4FrameBytes(1, 13, 12).syncframe;
  assert.throws(() => qqJocTrack(rawAc4.subarray(0, rawAc4.length - 1)), /truncated AC-4/);
  const wrongRate = fixture.slice(); wrongRate[4] |= 0x40;
  assert.throws(() => qqJocTrack(wrongRate), /48 kHz/);
  assert.throws(() => qqJocTrack(new Uint8Array([0, 0, 0, 1])), /truncated/);
});

test('QQ opener accepts Dolby JOC download routes, forwards cancellation and rejects external URLs before fetching', async () => {
  const original = globalThis.fetch;
  const controller = new AbortController();
  const calls = [];
  globalThis.fetch = async (url, options) => {
    calls.push({ url, options });
    const ac4 = url.includes('DAEAC3') ? false : true;
    const data = ac4
      ? mp4({ codec: 'ac-4', sampleList: [ac4FrameBytes(1, 13, 10).payload], sampleDuration: 2048 })
      : mp4();
    return { ok: true, arrayBuffer: async () => data.buffer };
  };
  try {
    for (const [quality, expectedCodec] of [['DAEAC3', 'ec-3'], ['DA', 'ac-4'], ['DA448', 'ac-4'], ['DA1500', 'ac-4']]) {
      const url = `/qq/song/WET/download?quality=${quality}`;
      const track = await openQqJocTrack(url, controller.signal);
      assert.ok(track.duration > 0);
      assert.equal(track.codec, expectedCodec);
      assert.equal(calls.at(-1).url, url);
      assert.equal(calls.at(-1).options.signal, controller.signal);
      assert.equal(calls.at(-1).options.credentials, 'same-origin');
      assert.equal(calls.at(-1).options.redirect, undefined);
    }
    const count = calls.length;
    for (const url of ['https://qq.com/file.mp4', '//qq.com/file.mp4', '/qq/song/WET/play?quality=DA', '/qq/song/WET/download?quality=HQ', '/qq/song/WET/download?quality=da', '/qq/song/WET/download?quality=DA&url=https://qq.com']) {
      await assert.rejects(openQqJocTrack(url), /local QQ download/);
    }
    assert.equal(calls.length, count);
    controller.abort();
    await assert.rejects(openQqJocTrack('/qq/song/WET/download?quality=DAEAC3', controller.signal), { name: 'AbortError' });
    assert.equal(calls.length, count);
  } finally { globalThis.fetch = original; }
});

test('QQ opener reads redirected CDN audio responses for E-AC-3 and AC-4', async () => {
  const ac4Samples = [ac4FrameBytes(1, 13, 10).payload];
  for (const [quality, codec, samples] of [['DAEAC3', 'ec-3', packets], ['DA', 'ac-4', ac4Samples], ['DA448', 'ac-4', ac4Samples], ['DA1500', 'ac-4', ac4Samples]]) {
    const data = mp4({ codec, sampleList: samples, sampleDuration: codec === 'ec-3' ? 1536 : 2048 });
    const progress = [];
    globalThis.fetch = async (url, options) => {
      const request = new Request(new URL(url, 'https://fantastic.test'), options);
      assert.equal(request.redirect, 'follow');
      const response = new Response(data, { headers: { 'content-length': String(data.length) } });
      Object.defineProperties(response, {
        redirected: { value: true },
        url: { value: 'https://isure.stream.qqmusic.qq.com/audio.m4a' },
      });
      return response;
    };
    const track = await openQqJocTrack(`/qq/song/WET/download?quality=${quality}`, undefined, (value) => progress.push(value));
    assert.equal(track.codec, codec);
    assert.ok(track.duration > 0);
    assert.equal(progress.at(-1).downloaded, data.length);
    assert.equal(progress.at(-1).percent, 100);
    const decoded = samplesFromFragment(new Uint8Array(await track.load(track.segments[0])));
    assert.equal(decoded.length, samples.length);
    samples.forEach((expected, i) => assert.deepEqual(decoded[i], expected));
  }
});

test('QQ JOC opener reports streamed byte progress with and without Content-Length', async () => {
  const original = globalThis.fetch;
  const progress = [];
  const source = new Uint8Array(mp4());
  const chunks = [source.slice(0, 13), source.slice(13)];
  const response = (withLength) => ({ ok: true,
    headers: withLength ? { get: (name) => name === 'content-length' ? String(chunks[0].length + chunks[1].length) : null } : undefined,
    body: { getReader() { let index = 0; return { async read() { return index < chunks.length ? { done: false, value: chunks[index++] } : { done: true }; } }; } },
  });
  try {
    globalThis.fetch = async () => response(true);
    const withLengthProgress = [];
    await openQqJocTrack('/qq/song/WET/download?quality=DAEAC3', undefined, (value) => withLengthProgress.push(value));
    assert.equal(withLengthProgress.at(-1).downloaded, chunks[0].length + chunks[1].length);
    assert.equal(withLengthProgress.at(-1).percent, 100);
    globalThis.fetch = async () => response(false);
    await openQqJocTrack('/qq/song/WET/download?quality=DAEAC3', undefined, (value) => progress.push(value));
    assert.equal(progress.at(-1).downloaded, chunks[0].length + chunks[1].length);
    assert.equal(progress.at(-1).percent, null);
  } finally { globalThis.fetch = original; }
});

test('an abort while the download body arrives prevents parsing and a segment load honors abort', async () => {
  const original = globalThis.fetch;
  let finish;
  globalThis.fetch = async () => ({ ok: true, arrayBuffer: () => new Promise((resolve) => { finish = resolve; }) });
  try {
    const controller = new AbortController();
    const pending = openQqJocTrack('/qq/song/WET/download?quality=DAEAC3', controller.signal);
    await Promise.resolve();
    controller.abort(); finish(mp4().buffer);
    await assert.rejects(pending, { name: 'AbortError' });
    const track = qqJocTrack(fixture);
    await assert.rejects(track.load(track.segments[0], controller.signal), { name: 'AbortError' });
  } finally { globalThis.fetch = original; }
});

class WorkerStub {
  constructor(url) { assert.equal(url, '/assets/joc/decode-worker.js'); }
  postMessage() {}
  terminate() {}
}
function contextStub() {
  const gain = () => ({ gain: { value: 1, cancelScheduledValues() {}, setValueAtTime() {}, linearRampToValueAtTime() {} }, connect() {}, disconnect() {} });
  return { sampleRate: 48000, currentTime: 0, destination: {}, createGain: gain, resume: async () => {}, suspend: async () => {}, close() {} };
}

async function withJocStubs(seek, body) {
  const original = { AudioContext: globalThis.AudioContext, Worker: globalThis.Worker, fetch: globalThis.fetch };
  const methods = Object.fromEntries(['openDecoder', 'pump', 'seek'].map((name) =>
    [name, Object.getOwnPropertyDescriptor(JocLiveEngine.prototype, name)]));
  const calls = [];
  globalThis.AudioContext = function AudioContext() { return contextStub(); };
  globalThis.Worker = WorkerStub;
  globalThis.fetch = async (url) => { calls.push(url); return { ok: true, arrayBuffer: async () => mp4().buffer }; };
  Object.assign(JocLiveEngine.prototype, { openDecoder: async () => {}, pump: async () => {}, seek });
  const player = Object.create(AmPlayer.prototype);
  Object.assign(player, { volume: 0.4, mixGain: 1, mixer: { volumeChanged() {}, reset() {} } });
  try { await body(player, calls); } finally {
    if (player.pcm && !player.pcm.controller?.signal.aborted) { if (player.pcm.decoder) player.pcm.decoder.broken = true; player.pcm.destroy(); }
    for (const [name, descriptor] of Object.entries(methods)) {
      if (descriptor) Object.defineProperty(JocLiveEngine.prototype, name, descriptor); else delete JocLiveEngine.prototype[name];
    }
    Object.assign(globalThis, original);
  }
}

test('JocEngine.load injects QQ input into the existing renderer, and destruction aborts pending input', async () => {
  const oldWorker = globalThis.Worker;
  globalThis.Worker = WorkerStub;
  const engine = Object.create(JocEngine.prototype);
  Object.assign(engine, { started: Promise.resolve(), generation: 0, mode: 'speaker', nodes: new Set(), context: contextStub(),
    openDecoder: async () => {}, pump: async () => {} });
  try {
    let signal;
    await engine.load('/qq/song/WET/download?quality=DAEAC3', async (url, inputSignal) => {
      assert.equal(url, '/qq/song/WET/download?quality=DAEAC3'); signal = inputSignal; return qqJocTrack(mp4());
    });
    assert.ok(Math.abs(engine.duration - packets.length * 1536 / 48000) < 1e-10);
    assert.ok(engine.decoder);
    // Avoid keeping the test Worker's idle reuse timer alive.
    engine.decoder.broken = true;
    engine.destroy();
    assert.equal(signal.aborted, true);

    const pendingEngine = Object.create(JocEngine.prototype);
    Object.assign(pendingEngine, { started: Promise.resolve(), generation: 0, nodes: new Set(), context: contextStub() });
    let finish;
    const pending = pendingEngine.load('unused', async (_url, inputSignal) => {
      signal = inputSignal;
      return new Promise((resolve) => { finish = resolve; });
    });
    await Promise.resolve();
    pendingEngine.destroy();
    assert.equal(signal.aborted, true);
    finish(qqJocTrack(fixture));
    await assert.rejects(pending, { name: 'AbortError' });
    assert.equal(pendingEngine.decoder, undefined);
  } finally { globalThis.Worker = oldWorker; }
});

test('AmPlayer.tryMode feeds the selected QQ download through JocLiveEngine and preserves paused seek', async () => {
  let seek;
  await withJocStubs(async (time) => { seek = time; }, async (player, calls) => {
    Object.assign(player, { playToken: 7, current: {}, spatial: {}, audio: { volume: 0.4 }, teardown() {}, emitSpatial() {} });
    const item = qq.qualityItem(qq.toTrack({ id: 'WET' }), 'DAEAC3');
    await player.tryMode('joc2', item, 0.25, 7, true);
    assert.ok(player.pcm instanceof JocLiveEngine);
    assert.deepEqual(calls, [item.downloadUrl]);
    assert.ok(Math.abs(player.current.duration - packets.length * 1536 / 48000) < 1e-10);
    assert.equal(seek, 0.25);
    assert.equal(player.pcm.paused, true);
    assert.equal(player.pcm.gain.gain.value, 0.4);
    player.pcm.decoder.broken = true;
    player.pcm.destroy();
    assert.equal(player.pcm.controller.signal.aborted, true);
  });
});

test('QQ PCM entry rejects download and native classifications before opening input or creating JocLiveEngine', async () => {
  const player = Object.create(AmPlayer.prototype);
  player.teardown = () => assert.fail('unsupported input must be rejected before teardown or engine creation');
  for (const quality of ['HQ', 'NAC', 'DTS', 'DTSX']) {
    const item = qq.qualityItem(qq.toTrack({ id: 'WET' }), quality);
    for (const mode of ['joc2', 'jocbin', 'ec3']) {
      await assert.rejects(player.tryMode(mode, item, 0, 1), /requires playback=joc/);
      assert.equal(player.pcm, undefined);
    }
  }
});

test('AmPlayer.tryMode feeds QQ AC-4 downloads into JocLiveEngine with codec=ac-4', async () => {
  await withJocStubs(async () => {}, async (player, calls) => {
    Object.assign(player, { playToken: 9, current: {}, spatial: {}, audio: { volume: 0.4 }, clock: {},
      teardown() {}, emitSpatial() {}, renderProgress() {}, renderToggle() {} });
    const ac4Mp4 = mp4({ codec: 'ac-4', sampleList: [ac4FrameBytes(1, 13, 12).payload], sampleDuration: 2048 });
    globalThis.fetch = async (url) => { calls.push(url); return { ok: true, arrayBuffer: async () => ac4Mp4.buffer }; };
    for (const quality of ['DA', 'DA448', 'DA1500']) {
      const item = qq.qualityItem(qq.toTrack({ id: 'WET' }), quality);
      await player.tryMode('joc2', item, 0, 9, true);
      assert.ok(player.pcm instanceof JocLiveEngine);
      assert.equal(player.pcm.codec, 'ac-4');
      assert.equal(calls.at(-1), item.downloadUrl);
      player.pcm.decoder.broken = true;
      player.pcm.destroy();
    }
  });
});

test('preparing a QQ PCM voice uses the download opener and releases it on failure', async () => {
  await withJocStubs(async () => {}, async (player, calls) => {
    player.spatial = {};
    const item = qq.qualityItem(qq.toTrack({ id: 'WET' }), 'DAEAC3');
    const voice = await player.prerollVoice(item, 'joc2', 0.25);
    assert.deepEqual(calls, [item.downloadUrl]);
    assert.equal(voice.engine.gain.gain.value, 0);
    voice.engine.decoder.broken = true;
    voice.release();
    assert.equal(voice.engine.controller.signal.aborted, true);
    globalThis.fetch = async () => { throw new Error('download failed'); };
    await assert.rejects(player.prerollVoice(item, 'joc2', 0), /download failed/);
  });
});

test('QQ quality switching enters JocLiveEngine through QQ input and resumes native playback without losing the queue', async () => {
  await withJocStubs(async function (time) { this.anchorTime = time; }, async (player, calls) => {
    Object.assign(player, {
      source: 'qq', playToken: 0, queueSerial: 0, nextSerial: 0, current: null, pcm: null, spatial: {},
      unsupportedListeners: new Set(), mse: { destroy() {} },
      audio: { volume: 0.4, duration: 240, paused: true, currentTime: 0, canPlayType: () => 'probably',
        async play() { this.paused = false; }, pause() { this.paused = true; }, removeAttribute() {}, load() {} },
    });
    player.clock = player.audio;
    for (const name of ['show', 'showError', 'setLoading', 'renderTrackText', 'renderArtwork', 'renderMode',
      'emit', 'updateMediaSession', 'renderProgress', 'prepareNext', 'extendQueue', 'emitSpatial', 'updatePcm']) player[name] = () => {};
    await player.playQueue([qq.entryOf(qq.toTrack({ id: 'WET' })), qq.entryOf(qq.toTrack({ id: 'next' }))]);
    const queue = player.queue;
    player.audio.currentTime = 0.25;
    await player.switchVariant(qq.qualityVariant('DAEAC3', { format: 'E-AC-3 JOC' }));
    const engine = player.pcm;
    assert.ok(engine instanceof JocLiveEngine);
    assert.deepEqual(calls, ['/qq/song/WET/qualities?basic=1', '/qq/song/WET/download?quality=DAEAC3']);
    assert.equal(player.current.mode, 'joc2');
    assert.equal(engine.currentTime, 0.25);
    assert.equal(engine.paused, false);
    assert.equal(player.queue, queue);
    assert.equal(queue.pos, 0);
    assert.equal(queue.entries[0].quality, 'DAEAC3');
    engine.decoder.broken = true; // Do not retain the idle Worker timer in this test.
    await player.switchVariant(qq.qualityVariant('SQ', { format: 'FLAC' }));
    assert.equal(engine.controller.signal.aborted, true);
    assert.equal(player.pcm, null);
    assert.equal(player.current.mode, 'native');
    assert.equal(player.audio.src, '/qq/song/WET/play?quality=SQ');
    assert.equal(player.audio.currentTime, 0.25);
    assert.equal(player.audio.paused, false);
    assert.equal(player.queue, queue);
    assert.equal(queue.entries[1].track, 'next');
  });
});
