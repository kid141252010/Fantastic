import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { test } from 'node:test';
import { Node as UiNode } from '../support/ui-dom.mjs';

const sceneSource = readFileSync(new URL('../../web/joc/scene.mjs', import.meta.url), 'utf8')
  .replace(/^export /gm, '') + '\nmodule.exports = { open };';

/** 用最小 DOM 运行真实构造器与事件处理；不启动绘画循环或音频引擎 */
function sceneFixture(t) {
  class Node extends UiNode {
    get firstChild() { return this.children[0]; }
    getContext() { return {}; }
    getBoundingClientRect() { return { width: 800, height: 600 }; }
    setPointerCapture() {}
    focus() { document.activeElement = this; }
    remove() { document.body.children = document.body.children.filter((node) => node !== this); }
    contains(node) { return this === node || this.children.some((child) => child instanceof Node && child.contains(node)); }
    removeEventListener(type, fn) { this.events[type] = (this.events[type] || []).filter((listener) => listener !== fn); }
    emit(type, patch = {}) {
      for (const fn of this.events[type] || []) fn({
        type, target: this, preventDefault() {}, stopPropagation() {},
        pointerId: 1, pointerType: 'mouse', button: 0, clientX: 0, clientY: 0,
        movementX: 0, movementY: 0, ...patch,
      });
    }
    requestPointerLock() {
      document.pointerLockElement = this;
      document.emit('pointerlockchange');
    }
  }
  const document = Object.assign(new Node('document'), {
    createElement: (tag) => new Node(tag), body: new Node('body'), activeElement: null,
    pointerLockElement: null,
    exitPointerLock() { this.pointerLockElement = null; this.emit('pointerlockchange'); },
  });
  const module = { exports: {} };
  runInNewContext(sceneSource, {
    module, document, performance,
    window: { AmI18n: { t: (key) => key }, innerWidth: 1024, matchMedia: (query) => ({ matches: !query.includes('coarse') }) },
    HTMLInputElement: class {},
    ResizeObserver: class { observe() {} disconnect() {} },
    requestAnimationFrame() { return 1; }, cancelAnimationFrame() {},
  });
  const modes = [];
  const scene = module.exports.open({
    spatial: {}, onSpatial: () => () => {}, setSceneOpen() {},
    setSpatialInteractive: (active) => modes.push(active),
  });
  t.after(() => { if (document.body.contains(scene.root)) scene.close(); });
  return { scene, modes, document };
}

const cameraGestures = [
  ['right-button', { button: 2 }],
  ['Shift', { shiftKey: true }],
  ['two-finger', { pointerType: 'touch' }, { pointerId: 2, pointerType: 'touch', clientX: 20 }],
];

for (const [kind, first, second] of cameraGestures) {
  for (const view of ['orbit', 'first']) {
    test(`${view} view ${kind} orbit enables interaction only when it turns the listener`, (t) => {
      const { scene, modes } = sceneFixture(t);
      scene.setView(view);
      scene.canvas.emit('pointerdown', first);
      if (second) scene.canvas.emit('pointerdown', second);
      scene.canvas.emit('pointermove', { ...first, clientX: 40, clientY: 10 });
      if (view === 'orbit') {
        assert.notEqual(scene.camera.yaw, 0);
        assert.equal(scene.state.yaw, undefined);
        assert.deepEqual(modes, []);
      } else {
        assert.ok(Math.abs(scene.state.yaw) > 0);
        assert.deepEqual(modes, [true]);
      }
      scene.canvas.emit('pointerup', first);
      if (second) {
        assert.deepEqual(modes, view === 'first' ? [true] : []);
        scene.canvas.emit('pointerup', second);
      }
      assert.deepEqual(modes, view === 'first' ? [true, false] : []);
    });
  }
}

for (const [view, walk] of [['first', false], ['orbit', false], ['orbit', true]]) {
  test(`${view} view listener ${walk ? 'walking' : 'head-turn'} drag enables interaction after the movement threshold`, (t) => {
    const { scene, modes } = sceneFixture(t);
    scene.setView(view);
    scene.walk = walk;
    scene.canvas.emit('pointerdown');
    scene.canvas.emit('pointermove', { clientX: 3 });
    assert.deepEqual(modes, []);
    scene.canvas.emit('pointermove', { clientX: 10 });
    scene.canvas.emit('pointermove', { clientX: 15 });
    assert.deepEqual(modes, [true]);
    assert.ok(Math.abs(walk ? scene.state.x : scene.state.yaw) > 0);
    scene.canvas.emit('pointerup');
    assert.deepEqual(modes, [true, false]);
  });
}

test('releasing the last movement key keeps interaction active while pointer-locked looking continues', (t) => {
  const { scene, modes, document } = sceneFixture(t);
  scene.setView('first');
  scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
  scene.canvas.requestPointerLock();
  scene.canvas.emit('pointermove', { movementX: 10 });
  scene.root.emit('keyup', { code: 'KeyW' });
  const yaw = scene.state.yaw;
  scene.canvas.emit('pointermove', { movementX: 10 });
  assert.notEqual(scene.state.yaw, yaw);
  assert.deepEqual(modes, [true]);
  document.exitPointerLock();
  assert.deepEqual(modes, [true, false]);
});

for (const end of ['pointerup', 'pointercancel']) {
  test(`${end} keeps interaction active while keyboard walking continues`, (t) => {
    const { scene, modes } = sceneFixture(t);
    scene.setView('first');
    scene.canvas.emit('pointerdown');
    scene.canvas.emit('pointermove', { clientX: 10 });
    scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
    scene.canvas.emit(end);
    assert.equal(scene.keys.has('KeyW'), true);
    assert.deepEqual(modes, [true]);
    scene.root.emit('keyup', { code: 'KeyW' });
    assert.deepEqual(modes, [true, false]);
  });
}

test('keyboard interaction ends only after all movement keys are released', (t) => {
  const { scene, modes } = sceneFixture(t);
  scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
  scene.root.emit('keydown', { key: 'w', code: 'KeyW', repeat: true });
  scene.root.emit('keydown', { key: 'd', code: 'KeyD' });
  scene.root.emit('keyup', { code: 'KeyW' });
  scene.root.emit('keyup', { code: 'Space' });
  assert.deepEqual(modes, [true]);
  scene.root.emit('keyup', { code: 'KeyD' });
  assert.deepEqual(modes, [true, false]);
});

test('focusout clears keyboard movement while keeping pointer-locked looking interactive', (t) => {
  const { scene, modes, document } = sceneFixture(t);
  scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
  scene.canvas.focus();
  scene.root.emit('focusout');
  assert.equal(scene.keys.size, 1);
  scene.canvas.requestPointerLock();
  document.body.focus();
  scene.root.emit('focusout');
  assert.equal(scene.keys.size, 0);
  assert.deepEqual(modes, [true]);
  document.exitPointerLock();
  assert.deepEqual(modes, [true, false]);
});

test('exiting pointer lock keeps interaction active while keyboard walking continues', (t) => {
  const { scene, modes, document } = sceneFixture(t);
  scene.canvas.requestPointerLock();
  scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
  document.exitPointerLock();
  assert.deepEqual(modes, [true]);
  scene.root.emit('keyup', { code: 'KeyW' });
  assert.deepEqual(modes, [true, false]);
});

test('closing the scene ends all interaction sources with a single mode change', (t) => {
  const { scene, modes } = sceneFixture(t);
  scene.root.emit('keydown', { key: 'w', code: 'KeyW' });
  scene.canvas.emit('pointerdown');
  scene.canvas.emit('pointermove', { clientX: 10 });
  scene.canvas.requestPointerLock();
  scene.close();
  assert.deepEqual(modes, [true, false]);
});
