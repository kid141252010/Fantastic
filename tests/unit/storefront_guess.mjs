import assert from 'node:assert/strict';
import { test } from 'node:test';
import { chooseStorefront, guessStorefront, regionlessResource } from '../../web/amp.mjs';

test('storefront choice prefers valid online regions then candidate order', () => {
  assert.equal(chooseStorefront(['jp', 'cn', 'us'], ['JP', 'US']), 'us');
  assert.equal(chooseStorefront(['jp', 'cn'], ['US']), 'cn');
  assert.equal(chooseStorefront(['tw']), 'tw');
  assert.equal(chooseStorefront([]), null);
});

test('regionless resources support merged slashes and optional slugs', () => {
  assert.deepEqual(regionlessResource('/https:/music.apple.com/song/name/123'), { type: 'songs', id: '123' });
  assert.deepEqual(regionlessResource('/https://music.apple.com/post/123'), { type: 'uploaded-videos', id: '123' });
  assert.deepEqual(regionlessResource('/https://music.apple.com/playlist/pl.abc/'), { type: 'playlists', id: 'pl.abc' });
  assert.equal(regionlessResource('/https://music.apple.com/us/song/name/123'), null);
  assert.equal(regionlessResource('/https://music.apple.com/song/name/nope'), null);
});

test('catalog probes all candidates and tolerates catalog and wrapper errors', async () => {
  const calls = [];
  const catalog = async (path) => {
    calls.push(path);
    if (path.includes('/kr/')) throw new Error('unavailable');
    return { data: path.includes('/jp/') || path.includes('/us/') ? [{ id: '123' }] : [] };
  };
  const resource = { type: 'songs', id: '123' };
  assert.equal(await guessStorefront(resource, { catalog, status: async () => ({ regions: ['JP'] }) }), 'jp');
  assert.equal(calls.length, 5);
  assert.equal(await guessStorefront(resource, { catalog, status: async () => { throw new Error('offline'); } }), 'us');
  assert.equal(await guessStorefront(resource, { catalog: async () => ({ data: [] }), status: async () => ({ regions: [] }) }), null);
});
