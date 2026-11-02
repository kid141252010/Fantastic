// Display artwork uses public QQ CDN URLs; blob consumers use the local cover adapter.
import { trackKey, withIdentity } from './track.mjs';
import { playbackQualityChain } from '../playback-quality.mjs';
export { trackKey } from './track.mjs';
const MID = /^[0-9A-Za-z]{1,32}$/;
export const enabled = () => globalThis.window?.FANTASTIC_HOST?.qqEnabled === true;
const mid = (id) => { if (!['string', 'number'].includes(typeof id) || !MID.test(String(id))) throw new Error('Invalid QQ ID'); return String(id); };
const FALLBACK_CAPABILITIES = [
  ['MP3128', 'mp3', 'audio/mpeg', 'native', 'size_128mp3'],
  ['AAC48', 'aac', 'audio/mp4; codecs="mp4a.40.2"', 'native', 'size_48aac'],
  ['WEB', 'aac', 'audio/mp4; codecs="mp4a.40.2"', 'native', 'size_96aac'],
  ['AAC192', 'aac', 'audio/mp4; codecs="mp4a.40.2"', 'native', 'size_192aac'],
  ['OGG96', 'vorbis', 'audio/ogg; codecs="vorbis"', 'native', 'size_96ogg'],
  ['OGG192', 'vorbis', 'audio/ogg; codecs="vorbis"', 'native', 'size_192ogg'],
  ['HQ', 'mp3', 'audio/mpeg', 'native', 'size_320mp3'],
  ['NAC', 'nac', 'audio/x-nac', 'download', null],
  ['SQ', 'flac', 'audio/flac', 'native', 'size_flac'],
  ['RS', 'flac', 'audio/flac', 'native', 'size_hires'],
  ['QAI', 'flac', 'audio/flac', 'native', null],
  ['Q360V1', 'flac', 'audio/flac', 'native', null],
  ['Q360V2', 'flac', 'audio/flac', 'native', null],
  ['RA360', 'unknown', 'application/octet-stream', 'download', 'size_360ra'],
  ['DTS', 'dts', 'audio/vnd.dts', 'download', 'size_dts'],
  ['DTSX', 'dtsx', 'audio/vnd.dts.hd; codecs="dtsx"', 'download', null],
  ['DA', 'ac-4', 'audio/mp4; codecs="ac-4"', 'ac-4-probe', 'size_dolby'],
  ['DAEAC3', 'ec-3', 'audio/mp4; codecs="ec-3"', 'joc', null],
  ['DA448', 'ac-4', 'audio/mp4; codecs="ac-4"', 'ac-4-probe', null],
  ['DA1500', 'ac-4', 'audio/mp4; codecs="ac-4"', 'ac-4-probe', null],
].map(([quality, defaultCodec, contentType, playbackClass, sizeField]) =>
  ({ quality, defaultCodec, contentType, playbackClass, sizeField }));
export const QUALITY_IDS = FALLBACK_CAPABILITIES.map(({ quality }) => quality);
const FALLBACK_BY_QUALITY = new Map(FALLBACK_CAPABILITIES.map((entry) => [entry.quality, entry]));

// Song detail quality groups. Keep the quality order here so every consumer
// renders the same order even when the adapter returns rows in another order.
export const QUALITY_GROUPS = Object.freeze([
  Object.freeze({ id: 'lossless', title: 'q.lossless', titleKey: true, qualities: Object.freeze(['RS', 'SQ']) }),
  Object.freeze({ id: 'atmos', title: 'q.atmos', titleKey: true, qualities: Object.freeze(['DA', 'DA1500', 'DA448', 'DAEAC3']) }),
  Object.freeze({ id: 'spatial', title: 'qq.groupSpatial', titleKey: true, qualities: Object.freeze(['QAI', 'Q360V1', 'Q360V2', 'RA360']) }),
  Object.freeze({ id: 'dts', title: 'DTS', titleKey: false, qualities: Object.freeze(['DTSX', 'DTS']) }),
  Object.freeze({ id: 'aac', title: 'AAC', titleKey: false, qualities: Object.freeze(['AAC192', 'WEB', 'AAC96', 'AAC48']) }),
  Object.freeze({ id: 'ogg', title: 'OGG', titleKey: false, qualities: Object.freeze(['OGG192', 'OGG96']) }),
  Object.freeze({ id: 'mp3', title: 'MP3', titleKey: false, qualities: Object.freeze(['HQ', 'MP3128', 'NAC']) }),
  Object.freeze({ id: 'other', title: '', titleKey: false, qualities: Object.freeze([]) }),
]);
export const QUALITY_GROUP_ORDER = Object.freeze(QUALITY_GROUPS.map(({ id }) => id));
export const QUALITY_GROUPS_BY_ID = new Map(QUALITY_GROUPS.map((group) => [group.id, group]));
const QUALITY_GROUP_BY_QUALITY = new Map(QUALITY_GROUPS.flatMap(({ id, qualities }) => qualities.map((quality) => [quality, id])));
export function qualityGroupFor(quality) { return QUALITY_GROUP_BY_QUALITY.get(quality) || 'other'; }
export function qualityGroupTitle(group, row = {}) {
  if (group !== 'other') return QUALITY_GROUPS_BY_ID.get(group)?.title || group;
  return row.format || row.quality || group;
}

let capabilityTable = FALLBACK_BY_QUALITY;

function validCapabilities(data) {
  if (!data?.enabled || !Array.isArray(data.qualities)) return null;
  const entries = data.qualities.map((entry) => {
    if (!entry || typeof entry.quality !== 'string' || !FALLBACK_BY_QUALITY.has(entry.quality)
      || typeof entry.defaultCodec !== 'string' || typeof entry.contentType !== 'string'
      || !['native', 'joc', 'download', 'ac-4-probe'].includes(entry.playbackClass)
      || (entry.sizeField !== null && typeof entry.sizeField !== 'string')) return null;
    return { quality: entry.quality, defaultCodec: entry.defaultCodec, contentType: entry.contentType,
      playbackClass: entry.playbackClass, sizeField: entry.sizeField };
  });
  if (entries.some((entry) => !entry) || new Set(entries.map((entry) => entry.quality)).size !== QUALITY_IDS.length
    || entries.length !== QUALITY_IDS.length) return null;
  return new Map(entries.map((entry) => [entry.quality, entry]));
}

/** Capabilities are static metadata; a failed or incomplete request keeps the bundled table. */
export const capabilitiesReady = (typeof globalThis.window !== 'undefined' && typeof fetch === 'function'
  ? fetch('/qq/capabilities', { cache: 'force-cache' })
    .then((response) => response.ok ? response.json() : null)
    .then((data) => { const table = validCapabilities(data); if (table) capabilityTable = table; return table; })
    .catch(() => null)
  : Promise.resolve(null));

export function capabilities() { return [...capabilityTable.values()].map((entry) => ({ ...entry })); }

// Cache by MIME and the runtime objects used for the probe. The object identity
// check keeps test/SSR environments from retaining a result after a fake browser
// is installed, while normal browser calls still use one result per MIME.
const mimeSupport = new Map();
function mimeEnvironment(environment) {
  const win = environment?.window;
  const mediaSource = environment?.ManagedMediaSource ?? win?.ManagedMediaSource
    ?? environment?.MediaSource ?? win?.MediaSource;
  const document = environment?.document ?? win?.document;
  return { win, mediaSource, document, audio: environment?.audio };
}

/** Runtime browser capability probe shared by QQ playback classification/tests. */
export function supportsMime(mime, environment = globalThis) {
  const context = mimeEnvironment(environment);
  const cached = mimeSupport.get(mime);
  if (cached && cached.win === context.win && cached.mediaSource === context.mediaSource
    && cached.document === context.document && cached.audio === context.audio) {
    return cached.supported;
  }
  let supported = false;
  if (context.win || context.document || context.mediaSource || context.audio) {
    try {
      supported = typeof context.mediaSource?.isTypeSupported === 'function'
        && context.mediaSource.isTypeSupported(mime) === true;
      if (!supported) {
        const audio = context.audio || context.document.createElement?.('audio');
        supported = typeof audio?.canPlayType === 'function' && audio.canPlayType(mime) !== '';
      }
    } catch { supported = false; }
  }
  mimeSupport.set(mime, { ...context, supported });
  return supported;
}

export function clearMimeSupportCache() { mimeSupport.clear(); }

/** WET stsd: DA/DA448/DA1500 = ac-4/dac4; DAEAC3 = ec-3/dec3 (2026-10-31).
 * QQ's fixed Dolby pipelines determine those codecs, including for saved queues.
 * EC-3 remains on the JOC pipeline; baseline codecs are native, while
 * AC-4 (support varies with browser licensing) is probed at runtime. */
export function playbackInfo(quality = 'HQ', row = {}, environment = globalThis) {
  qualityId(quality);
  const capability = capabilityTable.get(quality) || FALLBACK_BY_QUALITY.get(quality);
  const dolby = ['DA', 'DA448', 'DA1500', 'DAEAC3'].includes(quality);
  const codec = !dolby && typeof row.codec === 'string' ? row.codec : capability.defaultCodec;
  const contentType = !dolby && typeof row.contentType === 'string' ? row.contentType : capability.contentType;
  const playback = codec === 'ec-3' ? 'joc'
    : codec === 'ac-4' ? (supportsMime(contentType, environment) ? 'native' : 'download')
      : capability.playbackClass === 'download' ? 'download'
        : capability.playbackClass === 'joc' ? 'joc' : capability.playbackClass === 'native' ? 'native' : 'download';
  return { playback, codec, codecs: codec, contentType };
}
export function qualityId(value = 'HQ') {
  if (!QUALITY_IDS.includes(value)) throw new Error('Invalid QQ quality');
  return value;
}
const qualityQuery = (quality) => quality === undefined ? '' : `?quality=${qualityId(quality)}`;
export const songPath = (id) => `/qq-song/${mid(id)}`;
export const playUrl = (id, quality) => `/qq/song/${mid(id)}/play${qualityQuery(quality)}`;
export const downloadUrl = (id, quality) => `/qq/song/${mid(id)}/download${qualityQuery(quality)}`;
const RAW_LYRIC_KINDS = ['qrc', 'qrctrans', 'qrcroma'];
export function rawLyricsUrl(id, kind) {
  if (!RAW_LYRIC_KINDS.includes(kind)) throw new Error('Invalid QQ raw lyric kind');
  return `/qq/song/${mid(id)}/lyrics/raw?kind=${kind}`;
}
export const coverUrl = (albumMid) => albumMid === '' ? '' : `/qq/cover/${mid(albumMid)}`;
/** photo_new only serves discrete sizes. Resolve before Apple-sized consumers see it. */
export function artworkUrl(artwork, albumMid) {
  if (albumMid !== undefined && !MID.test(String(albumMid))) return '';
  const url = typeof artwork === 'string' ? artwork : artwork?.url;
  return typeof url === 'string' ? url.replaceAll('{w}', '500').replaceAll('{h}', '500') : '';
}
// Album/artist DTOs must use the same fixed size as song artwork before
// consumers request arbitrary display sizes (for example, 360px cards).
function normalizeArtwork(value) {
  if (!value || typeof value !== 'object') return value;
  const normalized = { ...value };
  for (const field of ['cover', 'avatar', 'artwork']) {
    if (!Object.hasOwn(normalized, field)) continue;
    const artwork = normalized[field];
    normalized[field] = artwork && typeof artwork === 'object'
      ? { ...artwork, url: artworkUrl(artwork) } : artworkUrl(artwork);
  }
  return normalized;
}
export function toTrack(dto) {
  const id = mid(dto.id);
  const albumMid = MID.test(dto.albumMid || '') ? dto.albumMid : '';
  const artists = (Array.isArray(dto.artists) ? dto.artists : []).map((artist) => {
    const name = typeof artist === 'string' ? artist : artist?.name;
    if (typeof name !== 'string') return null;
    const artistMid = typeof artist === 'object' && artist
      ? artist.mid || String(artist.href || '').match(/^\/qq-artist\/([0-9A-Za-z]{1,32})\/?$/)?.[1] || '' : '';
    return { name, href: MID.test(String(artistMid || '')) ? `/qq-artist/${artistMid}` : '' };
  }).filter(Boolean);
  const album = String(dto.album || '');
  return withIdentity({
    source: 'qq', kind: 'song', id, name: String(dto.name || ''), title: String(dto.name || ''),
    artists, artist: artists.map((a) => a.name).join(' / '), album,
    albumId: albumMid, albumHref: albumMid && album ? `/qq-album/${albumMid}` : '', href: songPath(id), country: '',
    duration: Number.isFinite(dto.durationMs) ? Math.max(0, dto.durationMs) : 0,
    artwork: { url: albumMid ? artworkUrl(dto.cover, albumMid) : '' }, explicit: false,
    qq: { mid: id, albumMid, vip: dto.vip === true, ttml: dto.ttml === true },
  });
}
export function entryOf(track) {
  const albumMid = track.qq?.albumMid || '';
  return withIdentity({ ...track, track: track.id, artwork: albumMid ? artworkUrl(track.artwork, albumMid) : '',
    artworkFetchUrl: albumMid ? coverUrl(albumMid) : '' });
}
export function qualityEntry(track, quality = 'HQ', row) {
  return { ...entryOf(track), quality: qualityId(quality), ...playbackInfo(quality, row) };
}
export function qualityVariant(quality, row = {}) {
  return { source: 'qq', group_id: qualityId(quality), quality, ...playbackInfo(quality, row),
    format: typeof row.format === 'string' ? row.format : '' };
}
/** Local adapter URLs only; no resolved upstream URL is cached in a variant. */
export function variantItem(id, variant) {
  const quality = qualityId(variant.quality);
  return { id: `qq:${mid(id)}:${quality}`, quality, ...playbackInfo(quality, variant), variant,
    streamUrl: playUrl(id, quality), downloadUrl: downloadUrl(id, quality), label: `QQ · ${quality}` };
}
export function qualityItem(track, quality = 'HQ', row) {
  return { ...qualityEntry(track, quality, row), ...variantItem(track.id, qualityVariant(quality, row)), title: track.name };
}
export function qualityVariants(rows) {
  return rows.filter((row) => row.available === true && ['native', 'joc'].includes(row.playback))
    .map((row) => qualityVariant(row.quality, row));
}
async function request(path, signal) {
  const response = await fetch(path, { signal });
  if (!response.ok) {
    const error = new Error(`QQ HTTP ${response.status}`);
    error.status = response.status;
    throw error;
  }
  return response.json();
}
export async function searchSongs(q, { limit = 30, offset = 0, signal } = {}) {
  const data = await request(`/qq/search?${new URLSearchParams({ q, limit, offset })}`, signal);
  return { songs: (data.songs || []).map(toTrack), total: data.total || 0,
    ...(Array.isArray(data.albums) ? { albums: data.albums.map(normalizeArtwork) } : {}),
    ...(Array.isArray(data.artists) ? { artists: data.artists.map(normalizeArtwork) } : {}),
  };
}
export async function song(id, signal) { return toTrack(await request(`/qq/song/${mid(id)}`, signal)); }
export async function album(id, signal) {
  const data = await request(`/qq/album/${mid(id)}`, signal);
  return { ...data,
    album: normalizeArtwork(data?.album),
    ...(Array.isArray(data?.songs) ? { songs: data.songs.map(normalizeArtwork) } : {}),
  };
}
export async function artist(id, signal) {
  const data = await request(`/qq/artist/${mid(id)}`, signal);
  return { ...data,
    artist: normalizeArtwork(data?.artist),
    ...(Array.isArray(data?.songs) ? { songs: data.songs.map(normalizeArtwork) } : {}),
    ...(Array.isArray(data?.albums) ? { albums: data.albums.map(normalizeArtwork) } : {}),
  };
}
export function qualityRows(data, id, detect, environment = globalThis) {
  const rows = Array.isArray(data?.qualities) ? data.qualities : [];
  detect ||= globalThis.window?.AmHook?.detectQqModes;
  const supplied = new Map();
  for (const row of rows) {
    if (typeof row?.quality === 'string' && row.quality && !supplied.has(row.quality)) supplied.set(row.quality, row);
  }
  return [...new Set([...QUALITY_IDS, ...supplied.keys()])].map((quality) => {
    const row = supplied.get(quality);
    const sizeBytes = Number.isSafeInteger(row?.sizeBytes) && row.sizeBytes >= 0 ? row.sizeBytes : null;
    const available = row?.available === true && sizeBytes > 0;
    const known = FALLBACK_BY_QUALITY.has(quality);
    const info = known ? playbackInfo(quality, row, environment) : {
      // Preserve new adapter tiers for display/download without putting an
      // unsupported quality ID into the player's capability-based menu.
      playback: 'download',
      codec: typeof row?.codec === 'string' ? row.codec : '',
      codecs: typeof row?.codec === 'string' ? row.codec : '',
      contentType: typeof row?.contentType === 'string' ? row.contentType : 'application/octet-stream',
    };
    const playable = available && info.playback !== 'download' && (!detect || detect(info).length > 0);
    const group = qualityGroupFor(quality);
    return { quality, group, available, playable, sizeBytes, ...info, format: typeof row?.format === 'string' ? row.format : '',
      ...(available ? { downloadUrl: known ? downloadUrl(id, quality)
        : `/qq/song/${mid(id)}/download?quality=${encodeURIComponent(quality)}` } : {}),
      ...(playable ? { playUrl: playUrl(id, quality) } : {}) };
  });
}
export async function qualities(id, signal, { basic = false } = {}) {
  const query = basic ? '?basic=1' : '';
  await capabilitiesReady;
  return qualityRows(await request(`/qq/song/${mid(id)}/qualities${query}`, signal), id);
}
const DEFAULT_QUALITY = { hires: 'RS', lossless: 'SQ', dolby: 'DAEAC3', high: 'HQ', standard: 'WEB' };
/** Pure: normalized /qualities rows + preference → row | null.
 * Only mapped, available native/JOC tiers qualify; respect capability filtering
 * when qualityRows supplied playable. Never override an explicit user choice. */
export function pickDefaultQuality(rows, preference = 'hires') {
  for (const choice of playbackQualityChain(preference)) {
    const row = rows.find((v) => v.quality === DEFAULT_QUALITY[choice] && v.available === true
      && ['native', 'joc'].includes(v.playback) && v.playable !== false);
    if (row) return row;
  }
  return null;
}
/** The detail page and full-screen lyrics share the parsed lyric model. */
export function lyricLines(result) {
  return (result?.song?.lines || []).filter((line) => typeof line.text === 'string' && line.text.trim())
    .map((line) => ({ text: line.text, translation: line.translation || '' }));
}
export const LYRIC_PRIORITY = ['amll', 'ttml', 'qrc', 'lrc'];
// Try the next source if a candidate is empty, malformed or has no timed lines.
export function selectLyrics(candidates, parse) {
  for (const source of LYRIC_PRIORITY) {
    for (const candidate of candidates || []) {
      if (candidate.source !== source || typeof candidate.text !== 'string' || !candidate.text.trim()) continue;
      try {
        const song = parse(candidate.text);
        if (song?.lines?.length) return { source, text: candidate.text, song, authors: [], amllStatus: null };
      } catch { /* fall through to the next candidate */ }
    }
  }
  return { song: null };
}
export async function lyrics(id, parse, signal) {
  const data = await request(`/qq/song/${mid(id)}/lyrics`, signal);
  return { ...selectLyrics(data.candidates, parse),
    rawQrc: data.rawQrc === true, rawTrans: data.rawTrans === true, rawRoma: data.rawRoma === true };
}
export function download(track, quality) {
  const link = document.createElement('a');
  link.href = downloadUrl(track.track ?? track.id, quality);
  link.download = '';
  document.body.append(link); link.click(); link.remove();
}
