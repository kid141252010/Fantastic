import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { test } from 'node:test';

const require = createRequire(import.meta.url);
const { AmPlayer, JocLiveEngine } = require('../../web/player/player.js');
const workerSource = readFileSync(new URL('../../web/joc/decode-worker.js', import.meta.url), 'utf8')
  .replace(/^import createEc3 from '\/assets\/ec3\/runtime\.mjs';\r?\n/m, '')
  + '\nmodule.exports = { liveTarget, stalledTarget, attachLive, detachLive, setVisibility, getLive: () => live };';
const workerModule = { exports: {} };
class MessageChannelStub {
  constructor() { this.port1 = {}; this.port2 = { postMessage() {} }; }
}
runInNewContext(workerSource, { module: workerModule, self: { postMessage() {} }, MessageChannel: MessageChannelStub });
const { liveTarget, stalledTarget } = workerModule.exports;

test('JOC Worker uses a robust 2.5-second default and grows the lead by 1.5 up to 5 seconds after stalls', () => {
  assert.equal(liveTarget(false, false, undefined), 120000);
  const afterFirstStall = stalledTarget(false, liveTarget(false, false, undefined));
  assert.equal(afterFirstStall, 180000);
  assert.equal(liveTarget(false, false, afterFirstStall), 180000);
  assert.equal(stalledTarget(false, afterFirstStall), 240000);
  assert.equal(stalledTarget(false, 240000), 240000);
});

test('JOC Worker keeps the interactive lead tiers and raises hidden robust lead to 4 seconds', () => {
  assert.equal(liveTarget(true, false, undefined), 3 * 1536);
  assert.equal(stalledTarget(true, 3 * 1536), 4 * 1536);
  assert.equal(stalledTarget(true, 11 * 1536), 12 * 1536);
  assert.equal(liveTarget(true, true, 6 * 1536), 6 * 1536);
  assert.equal(liveTarget(false, true, 120000), 192000);
  assert.equal(liveTarget(false, true, 240000), 240000);
});

test('JOC Worker grows the effective hidden lead from 4 to 5 seconds on the first stall', () => {
  const hiddenTarget = liveTarget(false, true, 120000);
  assert.equal(hiddenTarget, 192000);
  const afterFirstStall = stalledTarget(false, hiddenTarget);
  assert.equal(afterFirstStall, 240000);
  assert.equal(liveTarget(false, true, afterFirstStall), 240000);
  assert.equal(stalledTarget(false, afterFirstStall), 240000);

  const { attachLive, detachLive, setVisibility, getLive } = workerModule.exports;
  const port = { postMessage() {}, close() {} };
  setVisibility(true);
  attachLive(port);
  try {
    assert.equal(getLive().target, 192000);
    port.onmessage({ data: { epoch: 0, played: 0, starved: true } });
    assert.equal(getLive().target, 240000);
    assert.equal(getLive().adaptiveTarget, 240000);
    setVisibility(false);
    assert.equal(getLive().target, 240000);
    port.onmessage({ data: { epoch: 0, played: 0, starved: true } });
    assert.equal(getLive().target, 240000);
  } finally {
    detachLive();
    setVisibility(false);
  }
});

test('entering JOC live interaction seeks from the current position to rerender queued audio', () => {
  const messages = [];
  const seeks = [];
  const engine = Object.assign(Object.create(JocLiveEngine.prototype), {
    interactive: false,
    epoch: 1,
    node: {},
    playlist: {},
    decoder: { post: (op, payload) => messages.push({ op, ...payload }) },
    seek: (time) => { seeks.push(time); return Promise.resolve(); },
  });
  Object.defineProperty(engine, 'currentTime', { get: () => 37.125 });

  engine.setInteractive(true);
  engine.setInteractive(true);
  engine.setInteractive(false);

  assert.deepEqual(seeks, [37.125]);
  assert.deepEqual(messages, [
    { op: 'interactive', value: true },
    { op: 'interactive', value: true },
    { op: 'interactive', value: false },
  ]);
  assert.equal(engine.interactive, false);
});

test('interaction reuses Seek preroll, flush and fades while preserving playing or paused state', async () => {
  const outputSource = readFileSync(new URL('../../web/joc/output-worklet.js', import.meta.url), 'utf8');
  for (const paused of [false, true]) {
    let Output;
    const progress = [];
    const scope = {
      sampleRate: 48000,
      currentTime: 10,
      AudioWorkletProcessor: class { constructor() { this.port = { postMessage() {} }; } },
      registerProcessor: (_name, processor) => { Output = processor; },
    };
    runInNewContext(outputSource, scope);
    const output = new Output();
    output.worker = { postMessage: (data) => progress.push(data) };
    output.receive({ flush: true, epoch: 7 });
    output.receive({ epoch: 7, frames: new Float32Array(120000 * 2).fill(1) });
    output.startAt = 10;
    for (let played = 0; played < 12000; played += 128) {
      const n = Math.min(128, 12000 - played);
      scope.currentTime = 10 + played / 48000;
      output.process([], [[new Float32Array(n), new Float32Array(n)]]);
    }
    assert.equal(output.played, 12000);
    output.receive({ epoch: 7, end: true });

    const resets = [];
    const fades = [];
    let aborts = 0;
    let pumps = 0;
    const context = { currentTime: 10.25, suspend: async () => {}, resume: async () => {} };
    const engine = Object.assign(Object.create(JocLiveEngine.prototype), {
      paused, interactive: false, epoch: 7, generation: 0, duration: 60, rate: 48000,
      anchorTime: paused ? 31.25 : 31, anchorContextTime: 10,
      context, scenes: [], onUpdate() {},
      envelope: { gain: { value: 1, cancelScheduledValues() {}, setValueAtTime() {},
        linearRampToValueAtTime: (value) => fades.push(value) } },
      node: { port: { postMessage: (data) => output.port.onmessage({ data }) } },
      playlist: { segments: [0, 15, 30, 45].map((time) => ({ time })) },
      decoder: {
        post() {},
        abort: () => { aborts++; },
        reset: async (skip) => {
          resets.push(skip);
          output.receive({ flush: true, epoch: 8 });
          // 旧 epoch 的音频和 end 均不能污染重渲染这一轮。
          output.receive({ epoch: 7, frames: new Float32Array(128 * 2).fill(1) });
          output.receive({ epoch: 7, end: true });
          assert.equal(output.queue.length, 0);
          assert.equal(output.ended, false);
          return { epoch: 8 };
        },
      },
      pump: async (first) => {
        if (!first) return;
        pumps++;
        assert.equal(engine.controls.yaw, 0.3);
        output.receive({ epoch: 8, frames: new Float32Array(4608 * 2).fill(0.25) });
        engine.start(8);
      },
    });

    engine.setInteractive(true);
    engine.setControls({ yaw: 0.3 });
    engine.setInteractive(true);
    await engine.seekTask;
    assert.equal(engine.nextIndex, 1, '64-frame preroll starts in the preceding segment');
    assert.deepEqual(resets, [16.25 * 48000]);
    assert.equal(aborts, 1);
    assert.equal(pumps, 1);
    assert.equal(engine.paused, paused);
    assert.equal(engine.currentTime, 31.25);
    assert.deepEqual(fades, [0, 1]);

    engine.setInteractive(false);
    assert.equal(output.queue.length, 1);
    assert.equal(engine.epoch, 8);
    if (paused) await engine.play();
    output.receive({ epoch: 8, end: true });
    progress.length = 0;
    scope.currentTime = engine.anchorContextTime;
    const left = new Float32Array(4608);
    output.process([], [[left, new Float32Array(4608)]]);
    assert.ok(left.every((sample) => sample === 0.25));
    assert.equal(output.played, 4608);
    scope.currentTime += 4608 / 48000;
    output.process([], [[new Float32Array(128), new Float32Array(128)]]);
    assert.equal(progress.at(-1).starved, false, 'end silence is not starvation');
    assert.equal(resets.length, 1);
  }
});

test('spatial interaction returns to robust mode after a restartable five-second grace period', () => {
  const realSetTimeout = global.setTimeout;
  const realClearTimeout = global.clearTimeout;
  const timers = [];
  global.setTimeout = (callback, delay) => {
    const timer = { callback, delay, cleared: false };
    timers.push(timer);
    return timer;
  };
  global.clearTimeout = (timer) => { if (timer) timer.cleared = true; };
  try {
    const modes = [];
    const engine = { setInteractive: (value) => modes.push(value) };
    const player = Object.assign(Object.create(AmPlayer.prototype), {
      spatialInteractive: false,
      spatialInteractionTimer: null,
    });
    player.spatialEngine = () => engine;

    player.setSpatialInteractive(true);
    player.setSpatialInteractive(false);
    const firstGrace = timers[0];
    assert.equal(firstGrace.delay, 5000);
    player.setSpatialInteractive(true);
    assert.equal(firstGrace.cleared, true);
    player.setSpatialInteractive(false);
    const lastGrace = timers[1];
    assert.equal(lastGrace.delay, 5000);
    assert.deepEqual(modes, [true]);

    lastGrace.callback();
    assert.equal(player.spatialInteractive, false);
    assert.deepEqual(modes, [true, false]);
  } finally {
    global.setTimeout = realSetTimeout;
    global.clearTimeout = realClearTimeout;
  }
});
