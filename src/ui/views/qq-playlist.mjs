// QQ 歌单页（/qq-playlist/{id}），由 app.mjs 挂载。部分歌单上游取不到头部信息（作者），此时只显示歌曲
import * as qq from '../sources/qq.mjs';
import { artUrl } from '../library.mjs';
import { trackKey } from '../sources/track.mjs';
import { createActions } from './actions.mjs';
import { createDetailHeader } from './detail-header.mjs';
import { trackRow, syncTrackRows } from './library-ui.mjs';

export const bodyClass = 'album-page qq-album-page';
export const styles = ['/assets/views/qq-album.css'];

export function mount({ root, url, signal, player, navigate, onLangChange, toast }) {
  const { t } = window.AmI18n;
  const $ = (id) => root.querySelector(`#${id}`);
  const id = url.pathname.match(/^\/qq-playlist\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  const actions = createActions({ signal, player, navigate, toast });
  const header = createDetailHeader({ $, signal, artUrl, rerender: renderHeader });
  let info = null;
  let tracks = [];
  let more = false;
  // 上游偏移按返回的行数累计；去重后的 tracks.length 可能更少（歌单里有重复歌曲）
  let offset = 0;
  let loaded = false;
  let rows = [];
  let error = '';
  let notesExpanded = false;
  const el = (tag, props = {}, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children.filter((child) => child !== null && child !== undefined && child !== false));
    return node;
  };
  const queue = () => tracks.map(qq.entryOf);
  const name = () => info?.name || t('qq.playlistTitle');
  const moreBtn = actions.headerMenu(() => (loaded ? {
    target: { kind: 'playlist', source: 'qq', href: qq.playlistPath(id), name: name(), getTracks: () => tracks },
    entries: {},
  } : null));
  moreBtn.disabled = true;

  function renderHeader() {
    document.title = `${name()} · Fantastic`;
    if (!loaded) return;
    $('title').classList.remove('skeleton');
    $('title').textContent = name();
    $('creator').textContent = info?.creator || '';
    const updated = info?.updateTime ? new Date(Number(info.updateTime)) : null;
    const count = info?.trackCount || tracks.length;
    $('sub').textContent = [
      count ? t('album.songs', { n: count }) : '',
      updated && !Number.isNaN(updated.getTime()) ? t('qq.updated', { date: updated.toLocaleDateString(window.AmI18n.locale) }) : '',
    ].filter(Boolean).join(' · ');
    const notes = String(info?.description || '').trim();
    $('notes').hidden = !notes;
    $('notes-text').textContent = notes;
    $('notes').classList.toggle('expanded', notesExpanded);
    $('notes-more').textContent = t(notesExpanded ? 'album.less' : 'album.more');
    $('notes-more').hidden = true;
    if (notes) {
      requestAnimationFrame(() => {
        const text = $('notes-text');
        $('notes-more').hidden = !notesExpanded && text.scrollHeight <= text.clientHeight + 1;
      });
    }
    const cover = info?.cover || tracks.find((track) => track.artwork?.url)?.artwork || '';
    header({ cover, attrs: {}, alt: t('album.coverAlt', { title: name() }) });
    $('play-all').disabled = $('shuffle').disabled = !tracks.length;
    moreBtn.disabled = false;
  }

  function play(index) {
    const list = queue();
    if (!list.length) return;
    if (player.current && trackKey(player.current) === list[index]?.key) { player.toggle(); return; }
    player.playQueue(list, index);
  }

  function renderRows() {
    rows = tracks.map((track, index) => trackRow(track, { actions, onPlay: () => play(index) }));
    $('tracks').replaceChildren(...rows.map((row) => row.row));
    $('tracks').hidden = !rows.length;
    $('more').hidden = !more;
    syncTrackRows(rows, player);
    const minutes = Math.round(tracks.reduce((sum, track) => sum + (track.duration || 0), 0) / 60000);
    const line = [tracks.length && t('album.songs', { n: tracks.length }), minutes && t('album.minutes', { n: minutes })].filter(Boolean).join(' · ');
    $('footer').replaceChildren(el('p', { textContent: line }));
    $('footer').hidden = !line;
  }

  function render() {
    renderHeader();
    renderRows();
    const message = error || (loaded && !info ? t('qq.playlistPartial') : '');
    $('alert').hidden = !message;
    $('alert').textContent = message;
  }

  $('play-all').addEventListener('click', () => play(0));
  $('shuffle').addEventListener('click', () => {
    const list = queue();
    if (list.length) player.playQueue(list, Math.floor(Math.random() * list.length), { shuffle: true });
  });
  $('notes-more').addEventListener('click', () => { notesExpanded = !notesExpanded; renderHeader(); });
  $('more').addEventListener('click', async () => {
    $('more').disabled = true;
    try {
      const page = await qq.playlist(id, { offset, signal });
      offset += page.songs.length;
      const keys = new Set(tracks.map((track) => track.key));
      tracks.push(...page.songs.filter((track) => !keys.has(track.key)));
      more = page.more && page.songs.length > 0;
      render();
    } catch (err) {
      if (!signal.aborted && err.name !== 'AbortError') toast(t('qq.playlistFailed', { msg: err.message }));
    } finally { $('more').disabled = false; }
  });
  $('hero').querySelector('.detail-actions').append(moreBtn);
  player.onChange(() => syncTrackRows(rows, player));
  onLangChange(render);
  render();
  qq.playlist(id, { signal }).then((data) => {
    if (signal.aborted) return;
    info = data.playlist;
    tracks = data.songs;
    offset = data.songs.length;
    more = data.more;
    loaded = true;
    render();
  }).catch((err) => {
    if (signal.aborted) return;
    error = t('qq.playlistFailed', { msg: err.message });
    render();
  });
}
