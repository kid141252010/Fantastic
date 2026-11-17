// 首页（/，搜索结果为 /?q=），由 app.mjs 挂载。与 music.apple.com/{cc}/search 相同的布局：
// 顶部是搜索框（也可粘贴链接直接打开）；没有搜索时显示最近搜索与「类别浏览」砖块（categories.mjs，默认折叠为一行），
// 搜索时换成结果区（search-results.mjs）。地区为主地区（AmI18n.storefront），切换后类别与结果一起刷新。
import { createActions } from './actions.mjs';
import { amp, artUrl } from '../amp.mjs';
import { parseAppleLink } from './links.mjs';
import { createResults, toItem, el, artNode, explicitBadge, KIND_LABELS } from './search-results.mjs';
import { loadCategories, brickNode, brickSkeletons } from './categories.mjs';
import * as qq from '../sources/qq.mjs';

const { AmI18n } = window;
const { t } = AmI18n;

export const bodyClass = 'home-page';
export const styles = ['/assets/views/library.css'];

/** 输入框识别出的链接类型 → 提示文字 */
const DETECT_LABELS = {
  song: 'home.detectSong', 'music-video': 'home.detectMv', post: 'home.detectPost', album: 'home.detectAlbum', playlist: 'home.detectPlaylist', artist: 'home.detectArtist',
  room: 'home.detectRoom', 'multi-room': 'home.detectRoom', grouping: 'home.detectGrouping', curator: 'home.detectCurator', new: 'home.detectNew',
  'new/top-charts': 'charts.title', station: 'station.pageTitle',
};

export function mount({ root, signal, player, navigate, onLangChange, toast, loadStatus }) {
  const $ = (id) => root.querySelector(`#${id}`);
  const actions = createActions({ signal, player, navigate, toast });
  const input = $('input');
  const errorEl = $('error');
  const detectEl = $('detect');
  const submitLabel = $('submit-label');
  const suggestEl = $('suggest');
  const landing = $('landing');
  let errorKey = null;

  /* ---------- 搜索音源（Apple Music / QQ 音乐）：只影响搜索，播放来源只随 play / playQueue 改变，回到首页不会打断其他来源的歌曲 ---------- */
  let source = 'apple';
  try { source = qq.enabled() && (new URL(location.href).searchParams.get('source') || localStorage.getItem('am-hook:source')) === 'qq' ? 'qq' : 'apple'; } catch {}
  function applySource() {
    $('source-switch').hidden = !qq.enabled();
    $('source-select').value = source;
    $('storefront').hidden = source === 'qq';
    // 示例链接、类别浏览与音质说明都是 Apple Music 的
    for (const node of root.querySelectorAll('.examples, .formats-section')) node.hidden = source === 'qq';
    $('categories').hidden = source === 'qq' || categoriesEmpty;
    input.dataset.i18nAttr = source === 'qq' ? 'placeholder=qq.placeholder' : 'placeholder=home.placeholder';
    input.placeholder = t(source === 'qq' ? 'qq.placeholder' : 'home.placeholder');
    try { localStorage.setItem('am-hook:source', source); } catch {}
  }
  $('source-select').addEventListener('change', () => {
    source = $('source-select').value === 'qq' ? 'qq' : 'apple';
    closeSuggest();
    const url = new URL(location.href);
    url.searchParams.set('source', source);
    window.history.pushState(null, '', url);
    applySource(); renderDetect();
    if (source === 'apple' && !categoriesLoaded) renderCategories();
    if (input.value.trim()) search(input.value.trim(), 'replace'); else closeResults();
  });

  /** 搜索地区为主地区（导航里设置；结果与类别标题旁的按钮打开同一个选择面板）；l 为该地区选定的曲库语言或默认语言 */
  const storefront = () => AmI18n.storefront;
  const catalogLang = () => AmI18n.catalogLang(storefront());

  // wrapper-lite 状态由 app.mjs 维护（导航里的状态胶囊），进入首页时重新检查；没有选择过主地区时以账号所在地区为准，搜索与类别等它返回。
  // 搜索 QQ 音乐时不需要，第一次用到 Apple Music 时才检查
  let statusReady = source === 'qq' ? null : loadStatus();
  const ready = () => (statusReady ||= loadStatus());

  const results = createResults({
    els: { section: $('results'), title: $('results-title'), tabs: $('results-tabs'), body: $('results-body') },
    actions, storefront, catalogLang, ready, source: () => source, player, toast,
  });

  const isUrlLike = (value) => /^https?:\/\//i.test(value);

  /* ---------- 输入框：链接直接打开，其余按关键词搜索 ---------- */
  $('form').addEventListener('submit', (event) => {
    event.preventDefault();
    const value = input.value.trim();
    const link = source === 'apple' ? parseAppleLink(value) : null;
    if (link) { navigate('/' + link); return; }
    // 无法识别的链接与空输入给出提示
    errorKey = !value ? 'home.empty' : isUrlLike(value) ? 'home.invalid' : null;
    input.setAttribute('aria-invalid', String(!!errorKey));
    errorEl.hidden = !errorKey;
    if (errorKey) {
      errorEl.textContent = t(errorKey);
      input.focus();
      return;
    }
    closeSuggest();
    search(value, 'push');
  });
  input.addEventListener('input', () => { input.removeAttribute('aria-invalid'); errorEl.hidden = true; renderDetect(); scheduleSuggest(); });

  /** 输入时即时提示识别结果：歌曲 / MV 等类型与地区；按钮在“解析”与“搜索”之间切换 */
  function renderDetect() {
    const value = input.value.trim();
    const link = source === 'apple' && value && parseAppleLink(value);
    submitLabel.dataset.i18n = link ? 'home.submit' : 'home.search';
    submitLabel.textContent = t(submitLabel.dataset.i18n);
    detectEl.hidden = !link;
    if (!link) return;
    const [, cc, kind] = link.match(/^https:\/\/music\.apple\.com\/(?:([a-z]{2})\/)?(song|music-video|post|album|playlist|artist|station|room|multi-room|grouping|curator|new\/top-charts|new)\b/);
    detectEl.textContent = t(DETECT_LABELS[kind]) + (cc ? ` · ${cc.toUpperCase()}` : '');
  }

  // 示例链接：填入输入框，按“解析”即可打开
  root.querySelectorAll('[data-example]').forEach((btn) => btn.addEventListener('click', () => {
    input.value = btn.dataset.example;
    input.dispatchEvent(new Event('input'));
    input.focus();
  }));

  // 按 / 聚焦输入框
  document.addEventListener('keydown', (event) => {
    if (event.key !== '/' || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.target instanceof Element && event.target.closest('input, textarea, select, [contenteditable]')) return;
    event.preventDefault();
    input.focus();
  }, { signal });

  /* ---------- 建议下拉框：search/suggestions 的 terms（补全词）与 topResults（直达条目） ---------- */
  let suggestTimer = 0;
  let suggestController = null;
  let suggestItems = [];
  let activeIndex = -1;

  function scheduleSuggest() {
    clearTimeout(suggestTimer);
    if (suggestController) suggestController.abort();
    const term = input.value.trim();
    if (!term || (source === 'apple' && (parseAppleLink(term) || isUrlLike(term)))) { closeSuggest(); return; }
    suggestTimer = setTimeout(() => (source === 'qq' ? loadQQSuggest(term) : loadSuggest(term)), 180);
  }

  async function loadSuggest(term) {
    const controller = suggestController = new AbortController();
    try {
      await ready();
      // 参数与官网一致（types 相同，才能拿到与官网相同的直达结果与顺序）；本站打不开的类型在 renderSuggest 里跳过
      const data = await amp(`/v1/catalog/${storefront()}/search/suggestions`, {
        term, l: await catalogLang(), platform: 'web', kinds: 'terms,topResults',
        types: 'activities,artists,albums,editorial-items,music-movies,music-videos,playlists,record-labels,songs,stations,tv-episodes',
        'fields[albums]': 'artistName,artwork,contentRating,name,playParams,url', 'fields[artists]': 'url,name,artwork',
        'limit[results:terms]': '5', 'limit[results:topResults]': '10', 'omit[resource]': 'autos', 'art[url]': 'c,f', with: 'naturalLanguage',
      }, { signal: controller.signal });
      if (controller !== suggestController || document.activeElement !== input) return;
      renderSuggest(term, (data.results && data.results.suggestions) || []);
    } catch {
      if (controller === suggestController) closeSuggest();
    }
  }

  /** 补全词：点击后搜索 searchTerm */
  function termOption(term, display, searchTerm) {
    const option = el('button', 'suggest-item suggest-term');
    option.type = 'button';
    // 与官网一致：拆出与输入匹配的部分，补全部分加粗
    const at = display.toLowerCase().indexOf(term.toLowerCase());
    const text = el('span', 'suggest-text');
    if (at >= 0) text.append(el('b', null, display.slice(0, at)), el('span', null, display.slice(at, at + term.length)), el('b', null, display.slice(at + term.length)));
    else text.append(el('b', null, display));
    option.append(el('span', 'suggest-icon'), text);
    option.addEventListener('click', () => {
      input.value = searchTerm;
      renderDetect();
      closeSuggest();
      search(searchTerm, 'push');
    });
    return option;
  }

  /** 直达结果：链接到条目页面 */
  function resultOption({ href, title, sub, art, artClass = 'suggest-art', explicit = false }) {
    const option = el('a', 'suggest-item suggest-result');
    option.href = href;
    const titleNode = el('span', 'suggest-title', title);
    if (explicit) titleNode.append(explicitBadge());
    const text = el('span', 'suggest-text');
    text.append(titleNode, el('span', 'suggest-sub', sub));
    option.append(artNode(artClass, art), text);
    return option;
  }

  function renderSuggest(term, suggestions) {
    const options = [];
    for (const suggestion of suggestions) {
      if (suggestion.kind === 'terms' && suggestion.displayTerm) {
        options.push(termOption(term, suggestion.displayTerm, suggestion.searchTerm || suggestion.displayTerm));
      } else if (suggestion.kind === 'topResults') {
        const item = toItem(suggestion.content, storefront());
        if (!item) continue;
        // 与官网一致：接口给了 subtitle 就用它，否则为“类型 · 艺人 / 歌单作者”
        options.push(resultOption({
          href: item.href, title: item.title, explicit: item.explicit,
          sub: item.subtitle || [t(KIND_LABELS[item.kind]), item.artist].filter(Boolean).join(' · '),
          art: artUrl(item.artwork, 80, 80), artClass: item.mv ? 'suggest-art mv' : item.kind === 'artist' ? 'suggest-art round' : 'suggest-art',
        }));
      }
    }
    showSuggest(options);
  }

  /** QQ 音乐的输入联想：歌曲为补全词，艺人与专辑直达页面 */
  async function loadQQSuggest(term) {
    const controller = suggestController = new AbortController();
    try {
      const data = await qq.suggest(term, controller.signal);
      if (controller !== suggestController || document.activeElement !== input || source !== 'qq') return;
      showSuggest([
        ...data.songs.slice(0, 5).map((song) => termOption(term, [song.name, song.artist].filter(Boolean).join(' - '), [song.name, song.artist].filter(Boolean).join(' '))),
        ...data.artists.slice(0, 3).map((artist) => resultOption({
          href: qq.artistPath(artist.mid), title: artist.name, sub: t('home.detectArtist'),
          art: artUrl(artist.avatar, 80, 80), artClass: 'suggest-art round',
        })),
        ...data.albums.slice(0, 3).map((album) => resultOption({
          href: qq.albumPath(album.mid), title: album.name, sub: [t('home.detectAlbum'), album.artist].filter(Boolean).join(' · '),
          art: artUrl(album.cover, 80, 80),
        })),
      ]);
    } catch {
      if (controller === suggestController) closeSuggest();
    }
  }

  function showSuggest(options) {
    suggestItems = [];
    for (const option of options) {
      option.id = `suggest-${suggestItems.length}`;
      option.setAttribute('role', 'option');
      option.setAttribute('aria-selected', 'false');
      option.tabIndex = -1;
      // 保持输入框焦点，避免 blur 先于 click 关闭下拉框
      option.addEventListener('mousedown', (event) => event.preventDefault());
      suggestItems.push(option);
    }
    activeIndex = -1;
    input.removeAttribute('aria-activedescendant');
    suggestEl.replaceChildren(...suggestItems);
    suggestEl.hidden = suggestItems.length === 0;
    input.setAttribute('aria-expanded', String(!suggestEl.hidden));
  }

  function setActive(index) {
    activeIndex = index;
    suggestItems.forEach((option, i) => {
      option.classList.toggle('active', i === index);
      option.setAttribute('aria-selected', String(i === index));
    });
    if (index >= 0) {
      input.setAttribute('aria-activedescendant', suggestItems[index].id);
      suggestItems[index].scrollIntoView({ block: 'nearest' });
    } else input.removeAttribute('aria-activedescendant');
  }

  function closeSuggest() {
    clearTimeout(suggestTimer);
    if (suggestController) suggestController.abort();
    suggestController = null;
    suggestEl.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    setActive(-1);
  }

  input.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') { closeSuggest(); return; }
    if (suggestEl.hidden || !suggestItems.length) return;
    const n = suggestItems.length;
    if (event.key === 'ArrowDown') { event.preventDefault(); setActive(activeIndex + 1 >= n ? -1 : activeIndex + 1); }
    else if (event.key === 'ArrowUp') { event.preventDefault(); setActive(activeIndex - 1 < -1 ? n - 1 : activeIndex - 1); }
    else if (event.key === 'Enter' && activeIndex >= 0 && !event.isComposing) { event.preventDefault(); suggestItems[activeIndex].click(); }
  });
  input.addEventListener('blur', closeSuggest);
  input.addEventListener('focus', () => { if (input.value.trim()) scheduleSuggest(); });

  /* ---------- 搜索与地址栏 ?q= ---------- */
  /** history: 'push' / 'replace' 同步地址栏 ?q=，省略时不改动（切换地区、语言后重新搜索） */
  function search(term, history) {
    addRecent(term);
    if (history) {
      const url = new URL(location.href);
      url.searchParams.set('q', term);
      if (qq.enabled()) url.searchParams.set('source', source);
      if (url.href !== location.href) {
        if (history === 'push') window.history.pushState(null, '', url);
        else window.history.replaceState(window.history.state, '', url);
      }
    }
    landing.hidden = true;
    results.run(term);
  }

  function closeResults() {
    results.close();
    landing.hidden = false;
    const url = new URL(location.href);
    if (url.searchParams.has('q')) {
      url.searchParams.delete('q');
      history.replaceState(history.state, '', url);
    }
  }
  $('close-results').addEventListener('click', () => {
    closeResults();
    input.value = '';
    renderDetect();
  });

  /** 地址栏 ?q= 决定是否显示搜索结果（刷新、前进后退） */
  function syncFromUrl() {
    if (location.pathname !== '/') return; // 正在离开首页
    const params = new URL(location.href).searchParams;
    const nextSource = qq.enabled() && params.get('source') === 'qq' ? 'qq' : params.has('source') ? 'apple' : source;
    const changed = source !== nextSource;
    source = nextSource; applySource();
    if (changed && source === 'apple' && !categoriesLoaded) renderCategories();
    const q = (params.get('q') || '').trim();
    if (!q) {
      if (results.term || !$('results').hidden) closeResults();
      return;
    }
    input.value = q;
    renderDetect();
    if (changed || q !== results.term) search(q);
  }
  addEventListener('popstate', syncFromUrl, { signal });

  /* ---------- 最近搜索：关键词列表，新的在前，同词（不分大小写）去重 ---------- */
  const RECENT_KEY = 'am-hook:searches';
  const RECENT_LIMIT = 12;
  // 清理旧版“最近解析”留下的记录
  try { localStorage.removeItem('am-hook:recent'); } catch {}

  function readRecent() {
    try {
      const items = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]');
      return Array.isArray(items) ? items.filter((term) => typeof term === 'string' && term.trim()).slice(0, RECENT_LIMIT) : [];
    } catch { return []; }
  }

  function writeRecent(items) {
    try { localStorage.setItem(RECENT_KEY, JSON.stringify(items.slice(0, RECENT_LIMIT))); } catch {}
    renderRecent();
  }

  function addRecent(term) {
    const key = term.toLowerCase();
    writeRecent([term, ...readRecent().filter((item) => item.toLowerCase() !== key)]);
  }

  function renderRecent() {
    const items = readRecent();
    $('recent').hidden = items.length === 0;
    $('recent-list').replaceChildren(...items.map((term) => {
      const open = el('button', 'recent-term');
      open.type = 'button';
      open.append(el('span', 'suggest-icon'), el('span', 'recent-text', term));
      open.addEventListener('click', () => {
        input.value = term;
        input.removeAttribute('aria-invalid');
        errorEl.hidden = true;
        renderDetect();
        closeSuggest();
        search(term, 'push');
        $('results').scrollIntoView({ behavior: 'smooth', block: 'start' });
      });
      const remove = el('button', 'recent-remove');
      remove.type = 'button';
      remove.title = t('home.removeRecent', { term });
      remove.setAttribute('aria-label', remove.title);
      remove.addEventListener('click', () => writeRecent(readRecent().filter((item) => item !== term)));
      const li = el('li', 'recent-item');
      li.append(open, remove);
      return li;
    }));
  }

  $('clear-recent').addEventListener('click', () => {
    try { localStorage.removeItem(RECENT_KEY); } catch {}
    renderRecent();
  });

  /* ---------- 类别浏览（categories.mjs） ---------- */
  const grid = $('brick-grid');
  const categoriesEl = $('categories');
  const toggle = $('categories-toggle');
  let categoriesToken = 0;
  // 类别只属于 Apple Music：QQ 音源下不请求，切回 Apple Music 时再加载；没有类别时整块隐藏
  let categoriesLoaded = false;
  let categoriesEmpty = false;

  // 默认折叠：只显示第一行（手机两列时两行），按钮展开全部；展开状态按浏览器记住
  const EXPANDED_KEY = 'am-hook:categories-expanded';
  let expanded = false;
  try { expanded = localStorage.getItem(EXPANDED_KEY) === '1'; } catch {}

  /** 折叠时可见的砖块数；网格不可见（搜索中）时为 0，等 ResizeObserver 在重新显示时再算 */
  function collapsedCount() {
    const columns = getComputedStyle(grid).gridTemplateColumns;
    if (!columns || columns === 'none') return 0;
    const n = columns.split(' ').filter(Boolean).length;
    return n * (n <= 2 ? 2 : 1);
  }

  /** 按折叠状态隐藏多余的砖块（含加载占位），reveal 时给新露出的砖块加淡入动画 */
  function applyCollapse(reveal = false) {
    const limit = collapsedCount();
    if (!limit) return;
    const bricks = grid.querySelectorAll('a.brick').length;
    [...grid.children].forEach((node, i) => {
      const hide = !expanded && i >= limit;
      if (reveal && node.hidden && !hide) {
        node.style.setProperty('--i', String(i - limit));
        node.classList.add('reveal');
        node.addEventListener('animationend', () => node.classList.remove('reveal'), { once: true });
      }
      node.hidden = hide;
    });
    toggle.hidden = bricks <= limit;
    toggle.setAttribute('aria-expanded', String(expanded));
    $('categories-toggle-label').textContent = expanded ? t('home.showLess') : t('home.showAll', { n: bricks });
  }

  toggle.addEventListener('click', () => {
    expanded = !expanded;
    try { localStorage.setItem(EXPANDED_KEY, expanded ? '1' : '0'); } catch {}
    applyCollapse(expanded);
    // 收起时若标题已滚出视口，回到类别开头
    if (!expanded && categoriesEl.getBoundingClientRect().top < 0) categoriesEl.scrollIntoView({ block: 'start', behavior: 'smooth' });
  });

  // 列数随宽度变化（auto-fill），宽度改变或从隐藏恢复时重新计算
  const gridObserver = new ResizeObserver(() => applyCollapse());
  gridObserver.observe(grid);
  signal.addEventListener('abort', () => gridObserver.disconnect(), { once: true });

  async function renderCategories() {
    if (source === 'qq') return;
    const token = ++categoriesToken;
    categoriesLoaded = true;
    categoriesEmpty = false;
    categoriesEl.hidden = false;
    categoriesEl.setAttribute('aria-busy', 'true');
    // 已有砖块（切换语言 / 地区）时保留到新数据返回，避免整块闪烁
    if (!grid.querySelector('a.brick')) {
      grid.replaceChildren(...brickSkeletons());
      applyCollapse();
    }
    $('categories-error').hidden = true;
    try {
      await ready();
      const cc = storefront();
      const { title, bricks } = await loadCategories(cc, await catalogLang());
      if (token !== categoriesToken || signal.aborted) return;
      $('categories-title').textContent = title || t('home.categories');
      if (!bricks.length) { categoriesEmpty = true; categoriesEl.hidden = true; return; }
      grid.replaceChildren(...bricks.map(brickNode));
      applyCollapse();
    } catch (error) {
      if (token !== categoriesToken || signal.aborted) return;
      grid.replaceChildren();
      toggle.hidden = true;
      const failed = $('categories-error');
      failed.hidden = false;
      failed.querySelector('span').textContent = t('home.categoriesFailed', { msg: error.message });
    } finally {
      if (token === categoriesToken) categoriesEl.removeAttribute('aria-busy');
    }
  }
  $('categories-retry').addEventListener('click', renderCategories);

  /* ---------- 地区与语言（曲库语言变化经 onLangChange 通知） ---------- */
  signal.addEventListener('abort', AmI18n.onSettingsChange(({ kind }) => {
    if (kind !== 'storefront') return;
    closeSuggest();
    renderCategories();
    if (results.term) results.run(results.term);
  }), { once: true });

  /* ---------- 页脚版本：部署环境（host.js）给出，链到该版本的发布说明；直接打开静态文件时没有 ---------- */
  function renderVersion() {
    const version = String((window.FANTASTIC_HOST && window.FANTASTIC_HOST.version) || '');
    const link = $('footer-version');
    link.hidden = !/^\d+\.\d+\.\d+/.test(version);
    if (link.hidden) return;
    link.textContent = `v${version}`;
    link.href = 'https://github.com/kid141252010/Fantastic/blob/main/CHANGELOG.md';
    link.title = t('footer.release', { v: version });
  }

  onLangChange(() => {
    renderVersion();
    applyCollapse();
    renderDetect();
    closeSuggest();
    if (!errorEl.hidden && errorKey) errorEl.textContent = t(errorKey);
    applySource();
    if (results.term) results.run(results.term);
    renderRecent();
    renderCategories();
  });

  renderVersion();
  renderDetect();
  renderRecent();
  syncFromUrl();
  renderCategories();
}
