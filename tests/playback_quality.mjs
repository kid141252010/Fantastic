// Offline default streaming quality and preference storage contracts.
import test from 'node:test';
import assert from 'node:assert/strict';
import { PLAYBACK_QUALITIES, pickAppleVariant } from '../src/ui/playback-quality.mjs';
import { pickDefaultQuality, qualityRows } from '../src/ui/sources/qq.mjs';

const qqTiers = ['RS', 'SQ', 'DAEAC3', 'HQ', 'WEB'];
const browser = { window: {}, MediaSource: { isTypeSupported: () => true },
  document: { createElement: () => ({ canPlayType: () => 'probably' }) } };
const qqRows = qualityRows({ qualities: qqTiers.map((quality) => ({ quality, available: true, sizeBytes: 100 })) },
  'track', () => ['supported'], browser);

test('QQ selects each preferred tier and every successive fallback, including Hi-Res → SQ', () => {
  for (const [i, preference] of PLAYBACK_QUALITIES.entries()) {
    for (let j = i; j < qqTiers.length; j++) {
      const rows = qqRows.filter((v) => !qqTiers.slice(i, j).includes(v.quality));
      const before = structuredClone(rows);
      assert.equal(pickDefaultQuality(rows, preference), rows.find((v) => v.quality === qqTiers[j]), `${preference} → ${qqTiers[j]}`);
      assert.deepEqual(rows, before);
    }
  }
  assert.equal(pickDefaultQuality(qqRows).quality, 'RS');
  assert.equal(pickDefaultQuality(qqRows, 'invalid').quality, 'RS');
});

test('QQ only chooses available native/JOC mapped tiers and honors capability filtering', () => {
  const rows = [
    { quality: 'RS', available: false, playback: 'native' },
    { quality: 'SQ', available: true, playback: 'download' },
    { quality: 'DAEAC3', available: true, playback: 'joc' },
    { quality: 'HQ', available: true, playback: 'native' },
    { quality: 'WEB', available: true, playback: 'native' },
    ...['DA', 'DA448', 'DA1500', 'QAI'].map((quality) => ({ quality, available: true, playback: 'native' })),
  ];
  assert.equal(pickDefaultQuality(rows).quality, 'DAEAC3');
  assert.equal(pickDefaultQuality(rows.map((v) => v.quality === 'DAEAC3' ? { ...v, playable: false } : v)).quality, 'HQ');
  assert.equal(pickDefaultQuality(rows.filter((v) => !qqTiers.includes(v.quality))), null);
  assert.equal(pickDefaultQuality(qqRows.filter((v) => v.quality !== 'WEB'), 'standard'), null);
  assert.equal(pickDefaultQuality([]), null);
});

const appleTiers = [
  { group_id: 'audio-alac-stereo-96000-24', codecs: 'alac', sample_rate: 96000, bit_depth: 24, bandwidth: 4000000 },
  { group_id: 'audio-alac-stereo-48000-24', codecs: 'alac', sample_rate: 48000, bit_depth: 24, bandwidth: 2000000 },
  { group_id: 'audio-atmos-2768', codecs: 'ec-3', bandwidth: 768000 },
  { group_id: 'audio-stereo-256', codecs: 'mp4a.40.2', bandwidth: 256000 },
  { group_id: 'audio-stereo-128', codecs: 'mp4a.40.2', bandwidth: 128000 },
];

test('Apple selects every preferred category and fallback without upgrading past the preference', () => {
  for (const [i, preference] of PLAYBACK_QUALITIES.entries()) {
    for (let j = i; j < appleTiers.length; j++) {
      const variants = appleTiers.filter((v, k) => k < i || k >= j);
      const before = structuredClone(variants);
      assert.equal(pickAppleVariant(variants, preference), appleTiers[j], `${preference} → ${PLAYBACK_QUALITIES[j]}`);
      assert.deepEqual(variants, before);
    }
  }
  assert.equal(pickAppleVariant(appleTiers), appleTiers[0]);
  assert.equal(pickAppleVariant(appleTiers, 'invalid'), appleTiers[0]);
});

test('Apple recognizes codec, badge and label metadata, with the existing 48 kHz Hi-Res boundary', () => {
  for (const v of [
    { codecs: 'alac', sample_rate: 88200 },
    { badge: 'hires' },
    { label: 'Hi-Res Lossless' },
  ]) assert.equal(pickAppleVariant([appleTiers[1], v]), v);
  assert.equal(pickAppleVariant([appleTiers[0], { codecs: 'alac', sample_rate: 48000 }], 'lossless').sample_rate, 48000);
  for (const v of [{ badge: 'lossless' }, { label: 'Lossless' }]) {
    assert.equal(pickAppleVariant([appleTiers[2], v]), v);
  }
  for (const v of [{ codecs: 'ec-3' }, { label: 'EAC3-JOC' }, { badge: 'atmos' }]) {
    assert.equal(pickAppleVariant([appleTiers[3], v], 'dolby'), v);
  }
  const high = { label: 'AAC · 256 kbps' };
  const standard = { label: 'AAC · 128 kbps' };
  const he = { codecs: 'mp4a.40.5', bandwidth: 64000 };
  assert.equal(pickAppleVariant([standard, high], 'high'), high);
  assert.equal(pickAppleVariant([high, he], 'standard'), he);
});

test('Apple selects the highest bandwidth in a category and retains existing best as last resort', () => {
  const faster = { ...appleTiers[0], bandwidth: 5000000 };
  assert.equal(pickAppleVariant([appleTiers[0], faster]), faster);
  const best = { codecs: 'unknown', label: 'New format' };
  assert.equal(pickAppleVariant([best, { codecs: 'another' }]), best);
  assert.equal(pickAppleVariant([appleTiers[0]], 'standard'), appleTiers[0]);
  assert.equal(pickAppleVariant([]), null);
});

test('one streaming preference persists, validates, resets and synchronizes across tabs', async () => {
  const saved = { storage: globalThis.localStorage, listener: globalThis.addEventListener };
  const values = new Map([['am-hook:settings', JSON.stringify({ download: { codec: 'aac' } })]]);
  let storageChanged;
  globalThis.localStorage = { getItem: (k) => values.get(k) ?? null, setItem: (k, v) => values.set(k, v) };
  globalThis.addEventListener = (name, fn) => { if (name === 'storage') storageChanged = fn; };
  try {
    const { prefs, OPTIONS, DEFAULTS } = await import('../src/ui/prefs.mjs?playback-quality-test');
    assert.deepEqual(OPTIONS.playback.quality, PLAYBACK_QUALITIES);
    assert.deepEqual(DEFAULTS.playback, { quality: 'hires' });
    assert.deepEqual(prefs.get('playback'), { quality: 'hires' });
    assert.equal(prefs.get('download').codec, 'aac');
    const changes = [];
    const off = prefs.onChange((section, v) => { if (section === 'playback') changes.push(v.quality); });
    for (const quality of PLAYBACK_QUALITIES) {
      prefs.set('playback', { quality });
      assert.equal(JSON.parse(values.get('am-hook:settings')).playback.quality, quality);
    }
    const snapshot = prefs.get('playback');
    snapshot.quality = 'dolby';
    assert.equal(prefs.get('playback').quality, 'standard');
    assert.equal(prefs.set('playback', { quality: 'RS' }).quality, 'hires');
    values.set('am-hook:settings', JSON.stringify({ playback: { quality: 'lossless' }, download: { codec: 'aac' } }));
    storageChanged({ key: 'am-hook:settings' });
    assert.equal(prefs.get('playback').quality, 'lossless');
    assert.equal(changes.at(-1), 'lossless');
    assert.deepEqual(prefs.reset('playback'), { quality: 'hires' });
    assert.equal(prefs.get('download').codec, 'aac');
    off();
  } finally {
    if (saved.storage === undefined) delete globalThis.localStorage; else globalThis.localStorage = saved.storage;
    if (saved.listener === undefined) delete globalThis.addEventListener; else globalThis.addEventListener = saved.listener;
  }
});
