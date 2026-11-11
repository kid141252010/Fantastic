// QQ MV 页（/qq-mv/{vid}），由 app.mjs 挂载：<video> 直接播放 /qq/mv/{vid}/play 重定向到的 QQ CDN MP4
import * as qq from '../sources/qq.mjs';
import { artUrl } from '../library.mjs';
import { createActions } from './actions.mjs';

export const bodyClass = 'qq-mv-body';
export const styles = ['/assets/views/qq-mv.css'];

export function mount({ root, url, signal, player, navigate, onLangChange, toast }) {
  const { t, lang } = window.AmI18n;
  const $ = (id) => root.querySelector(`#${id}`);
  const vid = url.pathname.match(/^\/qq-mv\/([0-9A-Za-z]{1,32})\/?$/)?.[1];
  const actions = createActions({ signal, player, navigate, toast });
  let mv = null;
  let error = '';
  const el = (tag, props = {}, ...children) => {
    const node = Object.assign(document.createElement(tag), props);
    node.append(...children);
    return node;
  };
  const video = $('video');
  video.src = qq.mvPlayUrl(vid);
  $('open').href = qq.mvPlayUrl(vid);
  $('open').hidden = false;
  // 播放 MV 时暂停音乐（与 Apple Music MV 页相同）
  video.addEventListener('play', () => player.pause());
  signal.addEventListener('abort', () => { video.pause(); video.removeAttribute('src'); video.load(); }, { once: true });
  const more = actions.headerMenu(() => (mv ? { target: { kind: 'music-video', source: 'qq', href: qq.mvPath(vid), name: mv.name }, entries: {} } : null));
  $('actions').append(more);

  function render() {
    document.title = `${mv?.name || t('qq.mvTitle')} · Fantastic`;
    $('alert').hidden = !error;
    $('alert').textContent = error;
    if (!mv) return;
    $('title').textContent = mv.name || t('qq.mvTitle');
    $('artist').replaceChildren(...(mv.artists || []).flatMap((artist, index) => [
      index ? ' / ' : '',
      artist.mid ? el('a', { href: qq.artistPath(artist.mid), textContent: artist.name }) : artist.name,
    ]));
    const minutes = mv.durationMs ? `${Math.floor(mv.durationMs / 60000)}:${String(Math.round(mv.durationMs / 1000) % 60).padStart(2, '0')}` : '';
    $('meta').textContent = [
      mv.publishDate,
      minutes,
      mv.playCount ? t('qq.playCount', { n: Number(mv.playCount).toLocaleString(lang === 'zh' ? 'zh-CN' : 'en') }) : '',
    ].filter(Boolean).join(' · ');
    const desc = String(mv.description || '').trim();
    $('desc').hidden = !desc || desc === `${(mv.artists || []).map((a) => a.name).join(' / ')} - ${mv.name}`;
    $('desc').textContent = desc;
    if (mv.cover && !video.poster) video.poster = artUrl(mv.cover, 1280, 720);
  }

  onLangChange(render);
  render();
  qq.mv(vid, signal).then((data) => {
    if (signal.aborted) return;
    mv = data;
    render();
  }).catch((err) => {
    if (signal.aborted) return;
    error = t('qq.mvFailed', { msg: err.message });
    render();
  });
}
