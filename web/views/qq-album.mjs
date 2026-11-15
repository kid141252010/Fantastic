import * as qq from '../sources/qq.mjs';
import { artUrl } from '../library.mjs';
import { createActions } from './actions.mjs';
import { createDetailHeader } from './detail-header.mjs';
import { trackRow, syncTrackRows } from './library-ui.mjs';

export const bodyClass = 'album-page qq-album-page';
export const styles = ['/assets/views/qq-album.css'];

export function mount({ root, url, signal, player, navigate, onLangChange, toast }) {
  const { t } = window.AmI18n;
  const $ = (id) => root.querySelector(`#${id}`);
  const albumMid = url.pathname.match(/^\/qq-album\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  const actions = createActions({ signal, player, navigate, toast });
  const header = createDetailHeader({ $, signal, artUrl, rerender: renderHeader });
  let album = null;
  let tracks = [];
  let rows = [];
  let error = '';
  let notesExpanded = false;
  // 「更多」：添加到歌单、复制本站与 QQ 音乐链接
  const moreBtn = actions.headerMenu(() => (album ? {
    target: { kind: 'album', source: 'qq', href: qq.albumPath(albumMid), name: album.name, getTracks: () => playableTracks() },
    entries: {},
  } : null));
  moreBtn.disabled = true;

  const el = (tag, props = {}, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children.filter((child) => child !== null && child !== undefined && child !== false));
    return node;
  };
  const playableTracks = () => tracks.filter((track) => track.kind === 'song');
  const queue = () => playableTracks().map(qq.entryOf);
  const year = (value) => {
    if (!value) return '';
    const date = new Date(Number(value));
    return Number.isNaN(date.getTime()) ? '' : String(date.getUTCFullYear());
  };
  const artistLinks = (artists) => (artists || []).flatMap((artist, index) => [
    index ? ' / ' : '',
    artist.mid ? el('a', { href: `/qq-artist/${artist.mid}`, textContent: artist.name }) : artist.name,
  ]);

  function renderHeader() {
    if (!album) return;
    document.title = `${album.name || t('album.pageTitle')} · Fantastic`;
    const title = $('title');
    title.classList.remove('skeleton');
    title.textContent = album.name || '';
    const artist = $('artist');
    artist.classList.remove('skeleton');
    artist.replaceChildren(...artistLinks(album.artists));
    const meta = [year(album.publishTime), album.language, album.subType, album.size ? t('album.songs', { n: album.size }) : ''].filter(Boolean);
    $('sub').textContent = meta.join(' · ');
    const notes = String(album.description || '').trim();
    $('notes').hidden = !notes;
    $('notes-text').textContent = notes;
    $('notes').classList.toggle('expanded', notesExpanded);
    const notesMore = $('notes-more');
    if (notesMore) {
      notesMore.textContent = t(notesExpanded ? 'album.less' : 'album.more');
      notesMore.hidden = true;
      if (notes) {
        const raf = globalThis.requestAnimationFrame || ((fn) => setTimeout(fn, 0));
        raf(() => {
          const text = $('notes-text');
          notesMore.hidden = !notesExpanded && (Number(text.scrollHeight) || 0) <= (Number(text.clientHeight) || 0) + 1;
        });
      }
    }
    moreBtn.disabled = false;
    header({ cover: album.cover, attrs: {}, alt: t('album.coverAlt', { title: album.name }) });
    $('play-all').disabled = $('shuffle').disabled = !queue().length;
  }

  function renderRows() {
    rows = tracks.map((track, index) => trackRow(track, {
      actions,
      onPlay: () => {
        const list = queue();
        const at = list.findIndex((entry) => entry.track === track.id);
        if (at >= 0) player.playQueue(list, at);
      },
    }));
    $('tracks').replaceChildren(...rows.map((row) => row.row));
    $('tracks').hidden = !rows.length;
    syncTrackRows(rows, player);
  }

  function renderFooter() {
    const count = playableTracks().length;
    const duration = Math.round(playableTracks().reduce((sum, track) => sum + (track.duration || 0), 0) / 60000);
    $('footer').replaceChildren(el('p', { textContent: [count && t('album.songs', { n: count }), duration && t('album.minutes', { n: duration })].filter(Boolean).join(' · ') }));
    $('footer').hidden = !count && !duration;
  }

  function render() {
    renderHeader();
    renderRows();
    renderFooter();
    $('alert').hidden = !error;
    $('alert').textContent = error;
  }

  $('play-all').addEventListener('click', () => { const list = queue(); if (list.length) player.playQueue(list, 0); });
  $('shuffle').addEventListener('click', () => {
    const list = queue();
    for (let i = list.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [list[i], list[j]] = [list[j], list[i]]; }
    if (list.length) player.playQueue(list, 0);
  });
  player.onChange(() => syncTrackRows(rows, player));
  $('hero').querySelector('.detail-actions').append(moreBtn);
  $('notes-more')?.addEventListener('click', () => { notesExpanded = !notesExpanded; renderHeader(); });
  onLangChange(render);
  render();
  qq.album(albumMid, signal).then((data) => {
    if (signal.aborted) return;
    album = data?.album || null;
    tracks = Array.isArray(data?.songs) ? data.songs.map(qq.toTrack) : [];
    if (!album) throw new Error('empty response');
    render();
  }).catch((err) => {
    if (signal.aborted) return;
    error = t('album.failed', { msg: err.message });
    render();
  });
}
