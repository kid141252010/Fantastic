// One preference ladder for QQ and Apple. Callers supply candidates already
// filtered by the player's capability detection; manual selections bypass this.
export const PLAYBACK_QUALITIES = ['hires', 'lossless', 'dolby', 'high', 'standard'];

export function playbackQualityChain(preference = 'hires') {
  const index = PLAYBACK_QUALITIES.indexOf(preference);
  return PLAYBACK_QUALITIES.slice(Math.max(0, index));
}

function appleQuality(v) {
  const metadata = [v.codecs, v.group_id, v.badge, v.label, v.audio].filter((s) => typeof s === 'string').join(' ').toLowerCase();
  if (/hi[ -]?res|高解析/.test(metadata)) return 'hires';
  if (/alac|flac|lossless|无损/.test(metadata)) return v.sample_rate > 48000 ? 'hires' : 'lossless';
  if (/ec-?3|e-?ac-?3|atmos|dolby|杜比|全景声/.test(metadata)) return 'dolby';
  if (/aac|mp4a|stereo-\d/.test(metadata)) {
    const kbps = Number((metadata.match(/stereo-(\d+)|([\d.]+)\s*kbps/) || []).slice(1).find(Boolean)) || (v.bandwidth || 0) / 1000;
    return /he-aac|he-stereo|mp4a\.40\.(5|29)\b/.test(metadata) || (kbps > 0 && kbps <= 128) ? 'standard' : 'high';
  }
  return null;
}

/** Pure: playable variants + preference → variant | null.
 * Input order is the existing best order, also used when no ladder category
 * matches. Within a category prefer the greatest bandwidth, retaining ties. */
export function pickAppleVariant(variants, preference = 'hires') {
  for (const quality of playbackQualityChain(preference)) {
    const candidates = variants.filter((v) => appleQuality(v) === quality);
    if (candidates.length) return candidates.reduce((best, v) => (v.bandwidth || 0) > (best.bandwidth || 0) ? v : best);
  }
  return variants[0] || null;
}
