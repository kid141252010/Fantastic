import * as qq from '../sources/qq.mjs';
import { artUrl } from '../library.mjs';
import { createActions } from './actions.mjs';
import { trackRow, syncTrackRows } from './library-ui.mjs';

export const bodyClass = 'artist-page qq-artist-page';
export const styles = ['/assets/views/qq-artist.css'];

export function mount({ root, url, signal, player, navigate, onLangChange, toast }) {
  const { t } = window.AmI18n;
  const $ = (id) => root.querySelector(`#${id}`);
  const singerMid = url.pathname.match(/^\/qq-artist\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  const actions = createActions({ signal, player, navigate, toast });
  let artist = null;
  let songs = [];
  let totalSongs = 0;
  let albums = [];
  let rows = [];
  let error = '';
  const el = (tag, props = {}, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children.filter((child) => child !== null && child !== undefined && child !== false));
    return node;
  };
  const queue = () => songs.map(qq.entryOf);
  const year = (value) => {
    if (!value) return '';
    const date = new Date(Number(value));
    return Number.isNaN(date.getTime()) ? '' : String(date.getUTCFullYear());
  };

  function image(container, src, alt = '') {
    container.replaceChildren();
    if (src) container.append(el('img', { src, alt, loading: 'eager', decoding: 'async' }));
  }

  function renderHero() {
    if (!artist) return;
    document.title = `${artist.name || t('artist.pageTitle')} · Fantastic`;
    $('name').classList.remove('skeleton');
    $('name').textContent = artist.name || '';
    image($('portrait'), artist.avatar || artist.cover, t('artist.portraitAlt', { name: artist.name }));
    image($('hero-art'), artist.cover || artist.avatar);
    image($('portrait-glow'), artist.avatar || artist.cover);
    $('counts').textContent = [artist.musicSize && t('qq.artistSongs', { n: artist.musicSize }), artist.albumSize && t('qq.artistAlbums', { n: artist.albumSize })].filter(Boolean).join(' · ');
    $('bio').hidden = !String(artist.briefDesc || '').trim();
    $('bio').textContent = String(artist.briefDesc || '').trim();
    $('play-all').disabled = $('shuffle').disabled = !songs.length;
  }

  function renderSongs() {
    rows = songs.map((track, index) => trackRow(track, {
      actions,
      onPlay: () => player.playQueue(queue(), index),
    }));
    $('tracks').replaceChildren(...rows.map(({ row }) => row));
    $('tracks').hidden = !rows.length;
    syncTrackRows(rows, player);
    const more = totalSongs > songs.length ? t('qq.artistTotal', { n: totalSongs }) : '';
    $('footer').replaceChildren(more ? el('p', { textContent: more }) : '');
    $('footer').hidden = !more;
  }

  function renderAlbums() {
    const cards = albums.map((album) => {
      const cover = artUrl(album.cover, 360);
      return el('a', { className: 'shelf-item', href: `/qq-album/${album.mid}` },
        el('span', { className: 'shelf-art' }, cover ? el('img', { src: cover, alt: '', loading: 'lazy', decoding: 'async' }) : el('span', { className: 'ph' })),
        el('span', { className: 'shelf-title', textContent: album.name || '' }),
        el('span', { className: 'shelf-sub', textContent: year(album.publishTime) }));
    });
    const shelf = $('shelves');
    shelf.replaceChildren(...(cards.length ? [el('section', { className: 'shelf-section qq-artist-shelf' },
      el('h2', { className: 'shelf-heading', textContent: t('search.albums') }),
      el('div', { className: 'shelf' }, ...cards))] : []));
  }

  function render() {
    renderHero();
    renderSongs();
    renderAlbums();
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
  onLangChange(render);
  render();
  qq.artist(singerMid, signal).then((data) => {
    if (signal.aborted) return;
    artist = data?.artist || null;
    songs = Array.isArray(data?.songs) ? data.songs.map(qq.toTrack) : [];
    totalSongs = Number(data?.totalSongs) || 0;
    albums = Array.isArray(data?.albums) ? data.albums : [];
    if (!artist) throw new Error('empty response');
    render();
  }).catch((err) => {
    if (signal.aborted) return;
    error = t('artist.failed', { msg: err.message });
    render();
  });
}
