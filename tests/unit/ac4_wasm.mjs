// AC-4 / A-JOC 浏览器 WASM 接口测试（web/joc/joc.wasm，crates/joc-wasm）：离线，只需 Node：
//   node --test tests/unit/ac4_wasm.mjs
// 无条件覆盖导出符号、错误处理、状态/控制/场景生命周期；当环境变量 AC4_SAMPLE_AJOC / AC4_SAMPLE_IMS
// 指向真实 AC-4 样本时，进一步端到端验证 MP4 解析 -> ac4_push_frame -> Speaker 2.0 & 双耳 HRTF 输出。
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import vm from 'node:vm';
import { qqJocTrack } from '../../web/qq-joc-input.mjs';

const root = new URL('../../', import.meta.url);
const wasmPath = process.env.JOC_WASM || new URL('web/joc/joc.wasm', root);
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const ex = instance.exports;
const worker = readFileSync(new URL('web/joc/decode-worker.js', root), 'utf8');
const samplesFromFragment = vm.runInNewContext(
  `${worker.slice(worker.indexOf('function boxes('), worker.indexOf('async function loadEc3('))}\nsamplesFromFragment`,
  { Uint8Array, DataView },
);
const hrtf = (name) => readFileSync(new URL(`web/joc/hrtf/${name}.jochrtf`, root));

function lastError() {
  return new TextDecoder().decode(new Uint8Array(ex.memory.buffer, ex.joc_error_ptr(), ex.joc_error_len()));
}

function copyIn(bytes) {
  const ptr = ex.joc_alloc(bytes.byteLength);
  new Uint8Array(ex.memory.buffer, ptr, bytes.byteLength).set(
    new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength),
  );
  return ptr;
}

test('joc.wasm 导出全部 AC-4 C ABI 符号', () => {
  for (const name of [
    'ac4_open', 'ac4_close', 'ac4_reset', 'ac4_push_frame', 'ac4_frame_samples',
    'ac4_pull', 'ac4_set_gain', 'ac4_set_controls', 'ac4_scene', 'ac4_buffered',
    'ac4_flush', 'ac4_status',
  ]) {
    assert.equal(typeof ex[name], 'function', `missing WASM export ${name}`);
  }
});

test('AC-4 WASM 生命周期、控制项、场景查询与错误处理不崩溃', () => {
  // 无效 HRTF 应返回空句柄并记录错误信息
  const junk = new Uint8Array(256).fill(0x5a);
  const junkPtr = copyIn(junk);
  assert.equal(ex.ac4_open(1, 0, 1, 5.0, junkPtr, junk.length), 0);
  ex.joc_free(junkPtr, junk.length);
  assert.match(lastError(), /hrtf/i);

  // Speaker 2.0 与 Binaural 打开、查询、复位、刷新与关闭
  const field = hrtf('D2');
  const fieldPtr = copyIn(field);
  const spk = ex.ac4_open(0, -3.0, 1, 5.0, 0, 0);
  const bin = ex.ac4_open(1, 0.0, 1, 5.0, fieldPtr, field.length);
  ex.joc_free(fieldPtr, field.length);
  assert.ok(spk > 0, lastError());
  assert.ok(bin > 0, lastError());

  const statusPtr = ex.joc_alloc(128);
  const scenePtr = ex.joc_alloc(64 * 4);
  const gainsPtr = ex.joc_alloc(16 * 4);
  new Float32Array(ex.memory.buffer, gainsPtr, 16).fill(1);
  try {
    assert.equal(ex.ac4_buffered(spk), 0);
    assert.equal(ex.ac4_buffered(bin), 0);
    assert.equal(ex.ac4_status(spk, statusPtr), 0, lastError());
    assert.equal(ex.ac4_status(bin, statusPtr), 0, lastError());
    assert.equal(ex.ac4_set_gain(spk, 0.0), 0);
    assert.equal(ex.ac4_set_controls(spk, 0.5, 0, 0, 0, 0, 0, 0, 1, gainsPtr), 0);
    assert.equal(ex.ac4_set_controls(bin, 0.5, 0.1, 0, 0.2, 0, 0, 0.5, 1, gainsPtr), 0);
    assert.equal(ex.ac4_scene(spk, scenePtr), 0);
    assert.equal(ex.ac4_scene(bin, scenePtr), 0);

    // 畸形帧数据返回 -1 且不触发 trap
    const bad = new Uint8Array([0xff, 0xff, 0xff, 0xff]);
    const badPtr = copyIn(bad);
    assert.equal(ex.ac4_frame_samples(badPtr, bad.length), -1);
    assert.ok(lastError().length > 0);
    assert.equal(ex.ac4_push_frame(spk, badPtr, bad.length), -1);
    assert.ok(lastError().length > 0);
    ex.joc_free(badPtr, bad.length);

    assert.equal(ex.ac4_reset(spk), 0);
    assert.equal(ex.ac4_reset(bin), 0);
    assert.equal(ex.ac4_flush(spk), 0);
    assert.equal(ex.ac4_flush(bin), 0);
  } finally {
    ex.ac4_close(spk);
    ex.ac4_close(bin);
    ex.joc_free(statusPtr, 128);
    ex.joc_free(scenePtr, 64 * 4);
    ex.joc_free(gainsPtr, 16 * 4);
  }
});

test('AC-4 WASM 端到端解码真实样本（若通过环境变量提供）', async (t) => {
  const samples = [
    { path: process.env.AC4_SAMPLE_AJOC, expectAjoc: true },
    { path: process.env.AC4_SAMPLE_IMS, expectAjoc: false },
  ].filter((entry) => entry.path && existsSync(entry.path));
  if (!samples.length) {
    t.skip('未设置 AC4_SAMPLE_AJOC / AC4_SAMPLE_IMS 样本路径');
    return;
  }

  const field = hrtf('D2');
  const outPtr = ex.joc_alloc(4096 * 2 * 4);
  const scenePtr = ex.joc_alloc(64 * 4);
  try {
    for (const sample of samples) {
      const track = qqJocTrack(readFileSync(sample.path));
      assert.equal(track.codec, 'ac-4');
      const frag = new Uint8Array(await track.load(track.segments[0]));
      const packets = samplesFromFragment(frag).slice(0, 10);

      for (const mode of [0, 1]) {
        const fieldPtr = mode === 1 ? copyIn(field) : 0;
        const handle = ex.ac4_open(mode, 0.0, 1, 0.5, fieldPtr, mode === 1 ? field.length : 0);
        if (fieldPtr) ex.joc_free(fieldPtr, field.length);
        assert.ok(handle > 0, lastError());

        let totalSamples = 0;
        let peak = 0;
        let gotScene = false;
        try {
          for (const pkt of packets) {
            const ptr = copyIn(pkt);
            const fs = ex.ac4_frame_samples(ptr, pkt.length);
            assert.equal(fs, 2048, lastError());
            assert.equal(ex.ac4_push_frame(handle, ptr, pkt.length), 0, lastError());
            ex.joc_free(ptr, pkt.length);
            if (mode === 1 && ex.ac4_scene(handle, scenePtr) === 1) {
              gotScene = true;
              const scene = new Float32Array(ex.memory.buffer, scenePtr, 64);
              assert.ok(scene.every(Number.isFinite));
            }
          }
          assert.equal(ex.ac4_flush(handle), 0, lastError());
          for (;;) {
            const n = ex.ac4_pull(handle, outPtr, 4096);
            assert.ok(n >= 0, lastError());
            if (!n) break;
            totalSamples += n;
            const pcm = new Float32Array(ex.memory.buffer, outPtr, n * 2);
            for (const v of pcm) {
              assert.ok(Number.isFinite(v), 'non-finite WASM sample');
              peak = Math.max(peak, Math.abs(v));
            }
          }
          assert.ok(totalSamples > 10 * 2048 * 0.5, `too few output samples: ${totalSamples}`);
          assert.ok(peak > 1e-4, `silent WASM output: ${peak}`);
          if (mode === 1 && sample.expectAjoc) {
            assert.equal(gotScene, true, 'expected A-JOC binaural scene');
          } else if (mode === 0) {
            assert.equal(ex.ac4_scene(handle, scenePtr), 0);
          }
        } finally {
          ex.ac4_close(handle);
        }
      }
    }
  } finally {
    ex.joc_free(outPtr, 4096 * 2 * 4);
    ex.joc_free(scenePtr, 64 * 4);
  }
});
