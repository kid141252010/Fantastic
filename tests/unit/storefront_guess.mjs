import assert from 'node:assert/strict';
import { test } from 'node:test';
import { storefrontCandidates, guessStorefront, regionlessResource } from '../../web/amp.mjs';
import { parseAppleLink } from '../../web/views/links.mjs';

test('pasted regionless Apple links preserve the missing region and query for guessing', () => {
  for (const path of [
    'song/name/123', 'album/1561058084', 'album/name/123', 'playlist/pl.abc',
    'playlist/name/pl.abc', 'artist/123', 'artist/name/123', 'music-video/name/123',
    'post/123', 'post/name/123', 'station/ra.123', 'station/name/ra.123',
  ]) {
    const url = `https://music.apple.com/${path}/?i=456#track`;
    assert.equal(parseAppleLink(url), url);
  }
  assert.equal(parseAppleLink('https://music.apple.com/us/song/name/123'), 'https://music.apple.com/us/song/name/123');
  assert.equal(parseAppleLink(''), null);
  assert.equal(parseAppleLink('https://music.apple.com/album/not-an-id'), null);
  assert.equal(parseAppleLink('https://example.com/album/123'), null);
});

test('storefront candidates normalize wrapper regions in order and fall back without valid codes', () => {
  assert.deepEqual(storefrontCandidates(['CA', 'jp', 'ca', '', 'USA', '12']), ['ca', 'jp']);
  assert.deepEqual(storefrontCandidates(['invalid']), ['cn', 'us', 'kr', 'jp', 'tw']);
  assert.deepEqual(storefrontCandidates([]), ['cn', 'us', 'kr', 'jp', 'tw']);
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
  assert.equal(await guessStorefront(resource, { catalog, status: async () => ({ regions: ['JP', 'US', 'jp'] }) }), 'jp');
  assert.deepEqual(calls, ['/v1/catalog/jp/songs/123', '/v1/catalog/us/songs/123']);
  calls.length = 0;
  assert.equal(await guessStorefront(resource, { catalog, status: async () => ({ regions: ['CA'] }) }), null);
  assert.deepEqual(calls, ['/v1/catalog/ca/songs/123']);
  assert.equal(await guessStorefront(resource, { catalog, status: async () => { throw new Error('offline'); } }), 'us');
  assert.equal(await guessStorefront(resource, { catalog, status: async () => ({ regions: [] }) }), 'us');
  assert.equal(await guessStorefront(resource, { catalog: async () => ({ data: [] }), status: async () => ({ regions: [] }) }), null);
});
