import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { test } from 'node:test';

const require = createRequire(import.meta.url);
const { AmPlayer } = require('../../web/player/player.js');
const workerSource = readFileSync(new URL('../../web/joc/decode-worker.js', import.meta.url), 'utf8')
  .replace(/^import createEc3 from '\/assets\/ec3\/runtime\.mjs';\r?\n/m, '')
  + '\nmodule.exports = { liveTarget, stalledTarget };';
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
