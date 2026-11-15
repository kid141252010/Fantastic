// Playback identity: old snapshots without a source belong to Apple Music.
export const sourceOf = (track) => track?.source === 'qq' ? 'qq' : 'apple';
export const trackKey = (track) => `${sourceOf(track)}:${String(track?.track ?? track?.id ?? '')}`;
export const withIdentity = (track) => ({ ...track, source: sourceOf(track), key: trackKey(track) });
