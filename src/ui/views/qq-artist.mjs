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
  const { AmI18n } = window;
  const $ = (id) => root.querySelector(`#${id}`);
  const singerMid = url.pathname.match(/^\/qq-artist\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  const actions = createActions({ signal, player, navigate, toast });
  let artist = null;
  let songs = [];
  let totalSongs = 0;
  let moreSongs = false;
  // 上游偏移按返回的条数累计，与去重后的列表长度无关
  let songOffset = 0;
  let albumOffset = 0;
  let mvOffset = 0;
  let moreMvs = false;
  let albums = [];
  let moreAlbums = false;
  let mvs = [];
  let loading = '';
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
    document.title = `${artist.name || t('artist.pageTitle')} · Fantastic`;
    $('name').classList.remove('skeleton');
    $('name').textContent = artist.name || '';
    image($('portrait'), artist.avatar || artist.cover, t('artist.portraitAlt', { name: artist.name }));
    image($('hero-art'), artist.cover || artist.avatar);
    image($('portrait-glow'), artist.avatar || artist.cover);
    $('counts').textContent = [
      (artist.transNames || []).join(' / '),
      artist.musicSize && t('qq.artistSongs', { n: artist.musicSize }),
      artist.albumSize && t('qq.artistAlbums', { n: artist.albumSize }),
      artist.mvSize && t('qq.artistMvs', { n: artist.mvSize }),
    ].filter(Boolean).join(' · ');
    $('play-all').disabled = $('shuffle').disabled = !songs.length;
  }

  function renderBio() {
    const bio = String(artist?.description || artist?.briefDesc || '').trim();
    const block = $('bio');
    const born = $('born');
    const birthday = artist?.birthday ? new Date(Number(artist.birthday)) : null;
    const bornText = birthday && !Number.isNaN(birthday.getTime())
      ? t('qq.birthday', { date: birthday.toLocaleDateString(AmI18n.lang === 'zh' ? 'zh-CN' : 'en', { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'Asia/Shanghai' }) }) : '';
    if (born) { born.hidden = !bornText; born.textContent = bornText; }
    block.hidden = !bio && !bornText;
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
    if (!songsExpanded && (songs.length > 15 || moreSongs)) {
      const more = el('button', { className: 'shelf-all', type: 'button', textContent: t('qq.moreSongs') });
      more.addEventListener('click', () => {
        if (songsExpanded) return;
        songsExpanded = true;
        renderSongs();
      });
      head.append(more);
    }
    const tail = [];
    if (songsExpanded && moreSongs) {
      tail.push(loadMoreButton('songs', async () => {
        const page = await qq.artistSongs(singerMid, { offset: songOffset, signal });
        songOffset += page.songs.length;
        const keys = new Set(songs.map((track) => track.key));
        songs.push(...page.songs.filter((track) => !keys.has(track.key)));
        moreSongs = page.more && page.songs.length > 0;
        renderSongs();
      }));
    }
    section.replaceChildren(head, ...rows.map(({ row }) => row), ...tail);
    section.hidden = !songs.length;
    syncTrackRows(rows, player);
    const footer = totalSongs > songs.length ? t('qq.artistTotal', { n: totalSongs }) : '';
    $('footer').replaceChildren(footer ? el('p', { textContent: footer }) : '');
    $('footer').hidden = !footer;
  }

  /** 「加载更多」：分页取下一批（歌曲 / 专辑），请求期间不可再点 */
  function loadMoreButton(kind, load) {
    const button = el('button', { className: 'btn load-more qq-load-more', type: 'button', textContent: t('search.more'), disabled: loading === kind });
    button.addEventListener('click', async () => {
      if (loading) return;
      loading = kind;
      button.disabled = true;
      try { await load(); } catch (err) { if (!signal.aborted && err.name !== 'AbortError') toast(t('artist.failed', { msg: err.message })); }
      finally { loading = ''; button.disabled = false; }
    });
    return button;
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
    if (!albumsExpanded && (albums.length > 6 || moreAlbums)) {
      const more = el('button', { className: 'shelf-all', type: 'button', textContent: t('artist.seeAll') });
      more.addEventListener('click', () => {
        if (albumsExpanded) return;
        albumsExpanded = true;
        renderAlbums();
      });
      head.append(more);
    }
    const tail = albumsExpanded && moreAlbums ? [loadMoreButton('albums', async () => {
      const page = await qq.artistAlbums(singerMid, { offset: albumOffset, signal });
      albumOffset += page.albums.length;
      const seen = new Set(albums.map((album) => album.mid));
      albums.push(...page.albums.filter((album) => !seen.has(album.mid)));
      moreAlbums = page.more && page.albums.length > 0;
      renderAlbums();
    })] : [];
    $('shelves').replaceChildren(...(albums.length
      ? [el('section', { className: 'shelf-section qq-artist-shelf' }, head, shelf, ...tail)] : []));
  }

  /** MV：16:9 封面，链接到 QQ MV 页 */
  function renderMvs() {
    const cards = mvs.map((mv) => {
      const cover = artUrl(mv.cover, 480, 270);
      return el('a', { className: 'shelf-item mv', href: qq.mvPath(mv.vid) },
        el('span', { className: 'shelf-art mv' }, cover
          ? el('img', { src: cover, alt: '', loading: 'lazy', decoding: 'async' })
          : el('span', { className: 'ph' })),
        el('span', { className: 'shelf-title', textContent: mv.name || '' }),
        el('span', { className: 'shelf-sub', textContent: year(mv.publishTime) }));
    });
    const head = el('div', { className: 'shelf-head' }, el('h2', { className: 'shelf-heading', textContent: t('qq.mvs') }));
    const tail = moreMvs ? [loadMoreButton('mvs', async () => {
      const page = await qq.artistMvs(singerMid, { offset: mvOffset, signal });
      mvOffset += page.mvs.length;
      const seen = new Set(mvs.map((mv) => mv.vid));
      mvs.push(...page.mvs.filter((mv) => !seen.has(mv.vid)));
      moreMvs = page.more && page.mvs.length > 0;
      renderMvs();
    })] : [];
    $('mv-shelf')?.replaceChildren(...(cards.length
      ? [el('section', { className: 'shelf-section qq-artist-shelf' }, head,
        el('div', { className: `shelf${moreMvs || mvs.length > 12 ? ' expanded' : ''}` }, ...cards), ...tail)] : []));
  }

  function render() {
    renderHero();
    renderSongs();
    renderAlbums();
    renderMvs();
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
    songOffset = songs.length;
    totalSongs = Number(data?.totalSongs) || 0;
    moreSongs = data?.moreSongs === true;
    albums = Array.isArray(data?.albums) ? data.albums : [];
    moreAlbums = data?.moreAlbums === true;
    albumOffset = albums.length;
    mvs = Array.isArray(data?.mvs) ? data.mvs : [];
    mvOffset = mvs.length;
    moreMvs = data?.moreMvs === true;
    if (!artist) throw new Error('empty response');
    render();
  }).catch((err) => {
    if (signal.aborted) return;
    error = t('artist.failed', { msg: err.message });
    render();
  });
}
