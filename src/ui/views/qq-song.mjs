import * as qq from '../sources/qq.mjs';
import { trackKey } from '../sources/track.mjs';
import { artUrl } from '../library.mjs';
import { parseTTML } from '../lyrics/ttml.mjs';
import { createDetailHeader } from './detail-header.mjs';
import { prefs } from '../prefs.mjs';

export const bodyClass = 'song-page qq-song-page';
export const styles = ['/assets/views/song.css', '/assets/views/qq-song.css'];

// Keep the quality row controls identical to the Apple Music song page.
const ICON = {
  play: '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5Z"/></svg>',
  pause: '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>',
  download: '<svg class="dl-ring" viewBox="0 0 36 36" aria-hidden="true"><circle class="dl-ring-track" cx="18" cy="18" r="16"/><circle class="dl-ring-fill" cx="18" cy="18" r="16" pathLength="100"/></svg>'
    + '<svg class="dl-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 4v11"/><path d="m7 10 5 5 5-5"/><path d="M5 20h14"/></svg>'
    + '<svg class="dl-stop" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="8" y="8" width="8" height="8" rx="1.5"/></svg>',
};
const CODEC_NAMES = new Set(['mp3', 'aac', 'flac', 'vorbis', 'opus', 'pcm', 'ape', 'ec-3', 'ac-3', 'ac-4', 'dts', 'dtsx', 'nac', 'unknown']);

// actions.mjs is browser-only at module evaluation time. Load it when a menu
// is first opened so the Node UI tests can still import this view module.
let actionsPromise;
const loadActions = () => (actionsPromise ||= import('./actions.mjs'));

export function mount({ root, url, signal, player, onLangChange, toast }) {
  const { t } = window.AmI18n;
  const { formatTime } = window.AmHook;
  const $ = (id) => root.querySelector(`#${id}`);
  const mid = url.pathname.match(/^\/qq-song\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  let track = null;
  let rows = [];
  let qualityState = 'loading';
  let lyric = { song: null, rawQrc: false, rawTrans: false, rawRoma: false };
  let lyricsExpanded = false;
  let errorKey = '';
  const el = (tag, props, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children);
    return node;
  };
  const header = createDetailHeader({ $, signal, artUrl, rerender: () => renderHeader() });
  const active = (quality) => !!track && trackKey(player.current) === track.key && player.current?.quality === quality;
  const playing = () => !!player.current && !player.transport().paused;
  const defaultQuality = () => qq.pickDefaultQuality(rows, prefs.get('playback').quality);
  const currentTrack = () => !!track && trackKey(player.current) === track.key;

  function renderHeader() {
    document.title = `${track?.name || t('qq.songTitle')} · Fantastic`;
    if (!track) return;
    $('title').textContent = track.name;
    const subtitle = $('subtitle');
    const artistParts = [];
    for (const [index, artist] of track.artists.entries()) {
      if (index) artistParts.push(' / ');
      artistParts.push(artist.href ? el('a', { href: artist.href, textContent: artist.name }) : artist.name);
    }
    subtitle.replaceChildren(...artistParts);
    const meta = $('meta');
    const metaParts = [];
    if (track.album) metaParts.push(track.albumHref ? el('a', { href: track.albumHref, textContent: track.album }) : track.album);
    if (track.duration) { if (metaParts.length) metaParts.push(' · '); metaParts.push(formatTime(track.duration / 1000)); }
    meta.replaceChildren(...metaParts);
    $('badges').replaceChildren(el('span', { className: 'qq-badge', textContent: t('source.qq') }),
      ...(track.qq.vip ? [el('span', { className: 'qq-badge qq-vip', textContent: t('qq.vip') })] : []));
    header({ cover: track.artwork, attrs: {}, alt: t('song.coverAlt', { title: track.name }) });
    $('enqueue').disabled = false;
    $('enqueue').textContent = t('qq.addQueue');
    syncPlaying();
  }

  function playQuality(quality) {
    const row = rows.find((row) => row.quality === quality);
    if (!track || !row?.playable) return;
    if (active(quality)) { player.toggle(); return; }
    player.play(qq.qualityItem(track, quality, row));
  }

  function syncPlaying() {
    $('play-best').disabled = !track || (!currentTrack() && !defaultQuality());
    $('play-best').textContent = t(currentTrack() && playing() ? 'song.pause' : 'song.play');
    for (const node of $('variant-list').querySelectorAll('.variant')) {
      const current = active(node.dataset.quality);
      node.classList.toggle('playing', current);
      node.classList.toggle('paused', current && !playing());
      const button = node.querySelector('.variant-play');
      if (button) button.innerHTML = current && playing() ? ICON.pause : ICON.play;
    }
  }

  function renderQualities() {
    $('quality-status').textContent = qualityState === 'loading' ? t('qq.qualitiesLoading')
      : qualityState === 'failed' ? t('qq.qualitiesFailed') : '';
    const renderRow = (row) => {
      const format = typeof row.format === 'string' ? row.format : '';
      const rawCodec = typeof row.codec === 'string' ? row.codec.trim() : '';
      const codec = rawCodec && CODEC_NAMES.has(rawCodec) ? rawCodec : (format || rawCodec || t('qq.formatUnknown'));
      const size = row.sizeBytes > 0 ? t('qq.sizeMb', { size: (row.sizeBytes / 1_000_000).toFixed(2) }) : '';
      const label = t(`qq.quality.${row.quality}.label`);
      const unsupported = t('row.unsupportedTitle', { codecs: row.codec, hint: t('player.hintDownload') });
      const play = el('button', { className: 'variant-play', type: 'button', innerHTML: ICON.play, disabled: !row.playable });
      play.setAttribute('aria-label', row.playable ? t('qq.playQualityAria', { quality: label }) : t('row.unsupported'));
      play.title = row.playable ? t('row.playTitle') : unsupported;
      play.addEventListener('click', () => playQuality(row.quality));
      const num = el('span', { className: 'variant-num' }, play,
        el('span', { className: 'eq', ariaHidden: 'true' }, el('i'), el('i'), el('i'), el('i')));
      const name = el('div', { className: 'variant-name' },
        el('span', { className: 'variant-label', textContent: label }),
        ...(!row.playable ? [el('span', { className: 'variant-tag warn', textContent: t('row.downloadOnly'), title: unsupported })] : []));
      const spec = el('div', { className: 'variant-spec' },
        [codec, size].filter(Boolean).join(' · ') + ' · ', el('code', { textContent: row.quality }));
      const node = el('div', { className: `variant${row.playable ? '' : ' unsupported'}` }, num,
        el('div', { className: 'variant-main' }, name, spec));
      node.dataset.quality = row.quality;
      const download = el('a', { className: 'variant-dl', href: row.downloadUrl, download: '', innerHTML: ICON.download });
      download.setAttribute('aria-label', t('qq.downloadQualityAria', { quality: label }));
      download.title = t('menu.download');
      const more = el('button', { className: 'variant-more more-btn', type: 'button', title: t('row.more'), innerHTML: '<svg viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="19" cy="12" r="1.8"/></svg>' });
      more.setAttribute('aria-label', t('row.moreAria', { name: label }));
      more.setAttribute('aria-haspopup', 'menu');
      more.setAttribute('aria-expanded', 'false');
      more.addEventListener('click', async (event) => {
        event.stopPropagation();
        const { openMenu, copyText, ICON: ACTION_ICON } = await loadActions();
        openMenu(more, [
          ...(row.playable ? [{ icon: ACTION_ICON.play, label: t('qq.playQuality'), onSelect: () => playQuality(row.quality) }] : []),
          { icon: ACTION_ICON.download, label: t('menu.download'), onSelect: () => download.click() },
          { icon: ACTION_ICON.link, label: t('qq.copyDownload'), title: row.downloadUrl,
            onSelect: () => copyText(row.downloadUrl, () => toast(t('action.copied'))) },
        ]);
      });
      node.append(download, more);
      node.addEventListener('dblclick', (event) => { if (!event.target?.closest?.('a, button')) playQuality(row.quality); });
      return node;
    };
    const available = rows.filter((row) => row.available);
    const groups = [];
    for (const group of qq.QUALITY_GROUPS) {
      const grouped = available.filter((row) => row.group === group.id);
      if (!grouped.length) continue;
      if (group.id === 'other') {
        const fallback = new Map();
        for (const row of grouped) {
          const title = qq.qualityGroupTitle(group.id, row);
          if (!fallback.has(title)) fallback.set(title, el('div', { className: 'quality-group' },
            el('h3', { className: 'quality-group-title', textContent: title })));
          fallback.get(title).append(renderRow(row));
        }
        groups.push(...fallback.values());
        continue;
      }
      if (group.qualities.length) {
        const order = new Map(group.qualities.map((quality, index) => [quality, index]));
        grouped.sort((a, b) => (order.get(a.quality) ?? group.qualities.length)
          - (order.get(b.quality) ?? group.qualities.length));
      }
      const title = group.titleKey ? t(group.title) : group.title;
      const container = el('div', { className: 'quality-group' },
        el('h3', { className: 'quality-group-title', textContent: title }),
        ...grouped.map(renderRow));
      groups.push(container);
    }
    $('variant-list').replaceChildren(...groups);
    syncPlaying();
  }

  const controls = { preview: $('lyrics-preview'), qrc: $('lyrics-qrc'), trans: $('lyrics-trans'), roma: $('lyrics-roma') };
  controls.preview.addEventListener('click', () => { lyricsExpanded = !lyricsExpanded; renderLyrics(); });

  function renderLyrics() {
    const lines = qq.lyricLines(lyric);
    const hasRaw = lyric.rawQrc || lyric.rawTrans || lyric.rawRoma;
    $('lyrics').hidden = !(lines.length || hasRaw);
    controls.preview.hidden = !lines.length;
    controls.preview.textContent = t(lyricsExpanded ? 'qq.lyricsCollapse' : 'qq.lyricsPreview');
    controls.preview.setAttribute('aria-expanded', String(lyricsExpanded));
    controls.preview.setAttribute('aria-controls', 'lyrics-lines');
    controls.qrc.hidden = !lyric.rawQrc;
    controls.qrc.href = lyric.rawQrc ? qq.rawLyricsUrl(mid, 'qrc') : '';
    controls.trans.hidden = !lyric.rawTrans;
    controls.trans.href = lyric.rawTrans ? qq.rawLyricsUrl(mid, 'qrctrans') : '';
    controls.roma.hidden = !lyric.rawRoma;
    controls.roma.href = lyric.rawRoma ? qq.rawLyricsUrl(mid, 'qrcroma') : '';
    $('lyrics-source').textContent = lyric.source ? t('qq.lyricsSource', { source: lyric.source.toUpperCase() }) : '';
    $('lyrics-source').hidden = !lyricsExpanded;
    $('lyrics-lines').hidden = !lyricsExpanded;
    $('lyrics-lines').replaceChildren(...lines.map((line) => el('p', { className: 'qq-lyric-line' },
      el('span', { textContent: line.text }),
      ...(line.translation ? [el('span', { className: 'qq-lyric-translation', textContent: line.translation })] : []))));
  }

  function render() {
    renderHeader(); renderQualities(); renderLyrics();
    $('alert').hidden = !errorKey;
    $('alert').textContent = errorKey ? t(errorKey) : '';
  }
  $('play-best').addEventListener('click', () => {
    if (currentTrack()) { player.toggle(); return; }
    const row = defaultQuality();
    if (row) playQuality(row.quality);
  });
  $('enqueue').addEventListener('click', () => {
    if (!track) return;
    player.appendQueue(qq.entryOf(track));
    toast(t('qq.queued'));
  });
  player.onChange(syncPlaying);
  signal.addEventListener('abort', prefs.onChange((section) => { if (section === 'playback') syncPlaying(); }), { once: true });
  onLangChange(render);
  render();

  // Capabilities may replace the bundled table. Wait before the first quality
  // row is read; a failed request leaves the bundled fallback in place.
  qq.capabilitiesReady.then(() => {
    if (signal.aborted) return;
    rows = qq.qualityRows(null, mid);
    renderQualities();
  });

  // Metadata, qualities and lyrics fail independently; a missing lyric never hides the song.
  qq.song(mid, signal).then((value) => {
    if (signal.aborted) return;
    track = value; render();
  }).catch(() => { if (!signal.aborted) { errorKey = 'qq.songFailed'; render(); } });
  qq.qualities(mid, signal).then((value) => {
    if (signal.aborted) return;
    rows = value; qualityState = 'ready'; renderQualities();
  }).catch(() => { if (!signal.aborted) { qualityState = 'failed'; renderQualities(); } });
  qq.lyrics(mid, parseTTML, signal).then((value) => {
    if (signal.aborted) return;
    lyric = value; renderLyrics();
  }).catch(() => { /* Absent or unavailable lyrics omit the entire section. */ });
}
