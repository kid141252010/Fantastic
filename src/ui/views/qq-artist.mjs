import * as qq from '../sources/qq.mjs';
import { artUrl } from '../library.mjs';
import { trackKey } from '../sources/track.mjs';
import { createActions } from './actions.mjs';
import { syncTrackRows } from './library-ui.mjs';

export const bodyClass = 'artist-page qq-artist-page';
export const styles = ['/assets/views/qq-artist.css'];

const PLAY_ICON = '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5Z"/></svg>';

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
  let bioExpanded = false;
  let songsExpanded = false;
  let albumsExpanded = false;

  const el = (tag, props = {}, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children.filter((child) => child !== null && child !== undefined && child !== false));
    return node;
  };
  const queue = () => songs.map(qq.entryOf);
  const playSong = (index) => {
    const list = queue();
    if (index < 0 || index >= list.length) return;
    if (player.current && trackKey(player.current) === songs[index].key && typeof player.toggle === 'function') {
      player.toggle();
      return;
    }
    player.playQueue(list, index);
  };
  const year = (value) => {
    if (!value) return '';
    const date = new Date(Number(value));
    return Number.isNaN(date.getTime()) ? '' : String(date.getUTCFullYear());
  };
  const albumFor = (track) => {
    const mid = track.qq?.albumMid || track.albumId || '';
    return albums.find((album) => album.mid === mid) || null;
  };

  function image(container, src, alt = '') {
    container.replaceChildren();
    if (src) container.append(el('img', { src, alt, loading: 'eager', decoding: 'async' }));
  }

  function renderHero() {
    if (!artist) return;
    document.title = `${artist.name || t('artist.pageTitle')} · am-hook`;
    $('name').classList.remove('skeleton');
    $('name').textContent = artist.name || '';
    image($('portrait'), artist.avatar || artist.cover, t('artist.portraitAlt', { name: artist.name }));
    image($('hero-art'), artist.cover || artist.avatar);
    image($('portrait-glow'), artist.avatar || artist.cover);
    $('counts').textContent = [
      artist.musicSize && t('qq.artistSongs', { n: artist.musicSize }),
      artist.albumSize && t('qq.artistAlbums', { n: artist.albumSize }),
    ].filter(Boolean).join(' · ');
    $('play-all').disabled = $('shuffle').disabled = !songs.length;
  }

  function renderBio() {
    const bio = String(artist?.briefDesc || '').trim();
    const block = $('bio');
    block.hidden = !bio;
    $('bio-text').textContent = bio;
    block.classList.toggle('expanded', bioExpanded);
    const more = $('bio-more');
    more.textContent = t(bioExpanded ? 'qq.bioCollapse' : 'qq.bioExpand');
    more.hidden = true;
    if (bio) {
      const raf = globalThis.requestAnimationFrame || ((fn) => setTimeout(fn, 0));
      raf(() => {
        const text = $('bio-text');
        const scrollHeight = Number(text.scrollHeight) || 0;
        const clientHeight = Number(text.clientHeight) || 0;
        more.hidden = !bioExpanded && scrollHeight <= clientHeight + 1;
      });
    }
  }

  function songRow(track, index) {
    const album = albumFor(track);
    const albumName = track.album || album?.name || '';
    const albumMid = track.qq?.albumMid || track.albumId || '';
    const albumYear = album ? year(album.publishTime) : '';
    const src = artUrl(track.artwork, 80);
    const playBtn = el('button', { className: 'track-play', type: 'button', innerHTML: PLAY_ICON });
    playBtn.setAttribute('aria-label', t('album.playTrack', { name: track.name }));
    playBtn.addEventListener('click', (event) => { event.stopPropagation(); playSong(index); });
    const cover = el('span', { className: 'track-art' },
      src ? el('img', { src, alt: '', loading: 'lazy', decoding: 'async' }) : el('span', { className: 'ph' }),
      el('span', { className: 'eq', ariaHidden: 'true' }, el('i'), el('i'), el('i'), el('i')),
      playBtn);
    const title = el('a', { className: 'track-title', href: track.href, textContent: track.name });
    const albumParts = [];
    if (albumName) {
      albumParts.push(albumMid
        ? el('a', { href: `/qq-album/${albumMid}`, textContent: albumName })
        : albumName);
    }
    if (albumYear) albumParts.push(albumName ? ` · ${albumYear}` : albumYear);
    const main = el('div', { className: 'track-main' },
      el('div', { className: 'track-line' }, title,
        ...(track.qq?.vip ? [el('span', { className: 'qq-badge qq-vip', textContent: t('qq.vip') })] : []),
        ...(track.qq?.ttml ? [el('span', { className: 'qq-badge', textContent: t('qq.ttml') })] : [])),
      el('div', { className: 'track-artist' }, ...albumParts));
    const target = {
      kind: 'song', href: track.href, name: track.name, country: '', track,
      albumHref: albumMid ? `/qq-album/${albumMid}` : track.albumHref, onPlay: () => playSong(index),
    };
    const more = actions.moreButton(target);
    const row = el('div', { className: 'track ts-item qq-artist-track' }, cover, main, more);
    row.addEventListener('click', (event) => {
      if (event.target?.closest?.('a, button')) return;
      playSong(index);
    });
    return { row, playBtn, track };
  }

  function renderSongs() {
    const section = $('tracks');
    rows = (songsExpanded ? songs : songs.slice(0, 15)).map(songRow);
    const head = el('div', { className: 'shelf-head' },
      el('h2', { className: 'shelf-heading', textContent: t('qq.topSongs') }));
    if (!songsExpanded && songs.length > 15) {
      const more = el('button', { className: 'shelf-all', type: 'button', textContent: t('qq.moreSongs') });
      more.addEventListener('click', () => {
        if (songsExpanded) return;
        songsExpanded = true;
        const extra = songs.slice(15).map((track, index) => songRow(track, index + 15));
        rows.push(...extra);
        section.append(...extra.map(({ row }) => row));
        more.hidden = true;
        syncTrackRows(rows, player);
      });
      head.append(more);
    }
    section.replaceChildren(head, ...rows.map(({ row }) => row));
    section.hidden = !songs.length;
    syncTrackRows(rows, player);
    const footer = totalSongs > songs.length ? t('qq.artistTotal', { n: totalSongs }) : '';
    $('footer').replaceChildren(footer ? el('p', { textContent: footer }) : '');
    $('footer').hidden = !footer;
  }

  const albumCard = (album) => {
    const cover = artUrl(album.cover, 360);
    return el('a', { className: 'shelf-item', href: `/qq-album/${album.mid}` },
      el('span', { className: 'shelf-art' }, cover
        ? el('img', { src: cover, alt: '', loading: 'lazy', decoding: 'async' })
        : el('span', { className: 'ph' })),
      el('span', { className: 'shelf-title', textContent: album.name || '' }),
      el('span', { className: 'shelf-sub', textContent: year(album.publishTime) }));
  };

  function renderAlbums() {
    const visible = albumsExpanded ? albums : albums.slice(0, 6);
    const cards = visible.map(albumCard);
    const shelf = el('div', { className: `shelf${albumsExpanded ? ' expanded' : ''}` }, ...cards);
    const head = el('div', { className: 'shelf-head' },
      el('h2', { className: 'shelf-heading', textContent: t('search.albums') }));
    if (!albumsExpanded && albums.length > 6) {
      const more = el('button', { className: 'shelf-all', type: 'button', textContent: t('artist.seeAll') });
      more.addEventListener('click', () => {
        if (albumsExpanded) return;
        albumsExpanded = true;
        shelf.append(...albums.slice(6).map(albumCard));
        shelf.classList.toggle('expanded', true);
        more.hidden = true;
      });
      head.append(more);
    }
    $('shelves').replaceChildren(...(albums.length
      ? [el('section', { className: 'shelf-section qq-artist-shelf' }, head, shelf)] : []));
  }

  function render() {
    renderHero();
    renderSongs();
    renderAlbums();
    renderBio();
    $('alert').hidden = !error;
    $('alert').textContent = error;
  }

  $('play-all').addEventListener('click', () => playSong(0));
  $('shuffle').addEventListener('click', () => {
    const list = queue();
    for (let i = list.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [list[i], list[j]] = [list[j], list[i]];
    }
    if (list.length) player.playQueue(list, 0);
  });
  player.onChange(() => syncTrackRows(rows, player));
  $('bio-more').addEventListener('click', () => { bioExpanded = !bioExpanded; renderBio(); });
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
