import * as qq from '../sources/qq.mjs';
import { trackKey } from '../sources/track.mjs';
import { artUrl } from '../library.mjs';
import { parseTTML } from '../lyrics/ttml.mjs';
import { createDetailHeader } from './detail-header.mjs';
import { prefs } from '../prefs.mjs';

export const bodyClass = 'song-page qq-song-page';
export const styles = ['/assets/views/song.css', '/assets/views/qq-song.css'];

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
    document.title = `${track?.name || t('qq.songTitle')} · am-hook`;
    if (!track) return;
    $('title').textContent = track.name;
    $('subtitle').textContent = track.artist;
    $('meta').textContent = [track.album, track.duration ? formatTime(track.duration / 1000) : ''].filter(Boolean).join(' · ');
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
    for (const node of $('variant-list').children) {
      const current = active(node.dataset.quality);
      node.classList.toggle('playing', current);
      node.classList.toggle('paused', current && !playing());
      const button = node.querySelector('button');
      if (button) button.textContent = t(current && playing() ? 'player.pause' : 'qq.playQuality');
    }
  }

  function renderQualities() {
    $('quality-status').textContent = qualityState === 'loading' ? t('qq.qualitiesLoading')
      : qualityState === 'failed' ? t('qq.qualitiesFailed') : '';
    $('variant-list').replaceChildren(...rows.filter((row) => row.available).map((row) => {
      const format = row.format || t('qq.formatUnknown');
      const spec = [format,
        row.sizeBytes > 0 ? t('qq.sizeMb', { size: (row.sizeBytes / 1_000_000).toFixed(2) }) : ''].filter(Boolean).join(' · ');
      const label = t(`qq.quality.${row.quality}.label`);
      const node = el('div', { className: `variant qq-quality${row.playable ? '' : ' unsupported'}` },
        el('div', { className: 'variant-main' },
          el('div', { className: 'variant-name' }, el('span', { textContent: label }),
            ...(!row.playable ? [el('span', { className: 'variant-tag warn', textContent: t('row.downloadOnly'),
              title: t('row.unsupportedTitle', { codecs: row.codec, hint: t('player.hintDownload') }) })] : [])),
          el('div', { className: 'qq-quality-tip', textContent: t(`qq.quality.${row.quality}.tip`) }),
          el('div', { className: 'variant-spec', textContent: spec })));
      node.dataset.quality = row.quality;
      const actions = el('div', { className: 'qq-quality-actions' });
      if (row.playable) {
        const play = el('button', { className: 'song-reparse', type: 'button', disabled: !track, textContent: t('qq.playQuality') });
        play.setAttribute('aria-label', t('qq.playQualityAria', { quality: label }));
        play.addEventListener('click', () => playQuality(row.quality));
        actions.append(play);
      }
      const download = el('a', { className: 'song-reparse qq-quality-download', href: row.downloadUrl, download: '', textContent: t('qq.downloadQuality') });
      download.setAttribute('aria-label', t('qq.downloadQualityAria', { quality: label }));
      actions.append(download);
      node.append(actions);
      return node;
    }));
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
