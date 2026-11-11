/*
 * 歌词同步测量（排查用）：地址带 ?lyricprobe=1 后在歌词界面左上角显示，?lyricprobe=0 关闭（记在浏览器里）。
 * 每 0.5 秒给出最近 3 秒的：
 *   帧间隔         requestAnimationFrame 的间隔，换行那一帧重不重；
 *   播放进度        transport.currentTime 每帧前进量与页面时钟之差、回退与停住的帧数，读数抖不抖；
 *   AMLL 判定跳转   DomLyricPlayer 自动跳转检测判为跳转的次数（会让当前行重新开始动画）；
 *   换行晚了多少    行出现高亮的那一帧，送进的时间比行开始时间晚多少；
 *   动画刚开始      逐词动画从挂起转为运行的那一帧（校准之前），动画位置减去按进度应在的位置，负数为动画落后；
 *   动画平时        其余帧的同一差值；
 *   手动偏移        歌词整体提前的毫秒数，用来对着声音调到刚好，读出实际差多少。
 */

const KEY = 'am-hook:lyric-probe';
const OFFSET_KEY = 'am-hook:lyric-probe-offset';
const WINDOW = 3000;

function enabled() {
  try {
    const value = new URLSearchParams(location.search).get('lyricprobe');
    if (value !== null) localStorage.setItem(KEY, value === '0' ? '0' : '1');
    return localStorage.getItem(KEY) === '1';
  } catch {
    return false;
  }
}

const median = (values) => {
  if (!values.length) return NaN;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
};
const fmt = (value) => (Number.isFinite(value) ? String(Math.round(value)) : '-');

/** 未开启时返回 null；开启时返回 { offset() → 歌词提前的毫秒数, frame(timestamp, transport, time, running) } */
export function createProbe(root, view, player) {
  if (!enabled()) return null;
  let offset = 0;
  try { offset = Number(localStorage.getItem(OFFSET_KEY)) || 0; } catch { /* 默认 0 */ }

  const frames = [];
  const clock = [];
  const seeks = [];
  const enters = [];
  const starts = [];
  const steady = [];
  const seen = new WeakSet();
  let previous = null;
  let highlightedBefore = new Set();
  let lastRender = 0;

  const detector = view.seekDetector;
  if (detector && typeof detector.detect === 'function') {
    const detect = detector.detect.bind(detector);
    detector.detect = (...args) => {
      const jumped = detect(...args);
      if (jumped) seeks.push([performance.now(), 1]);
      return jumped;
    };
  }

  const box = document.createElement('div');
  box.className = 'lyrics-probe';
  Object.assign(box.style, {
    position: 'fixed', zIndex: '2147483647', left: '4px', top: 'calc(env(safe-area-inset-top, 0px) + 4px)',
    maxWidth: 'calc(100vw - 8px)', padding: '6px 8px', borderRadius: '8px', background: 'rgba(0,0,0,.72)',
    color: '#fff', font: '11px/1.35 ui-monospace, monospace', pointerEvents: 'auto', whiteSpace: 'pre-wrap',
  });
  const text = document.createElement('div');
  const buttons = document.createElement('div');
  buttons.style.marginTop = '4px';
  box.append(text, buttons);
  const stop = (event) => event.stopPropagation();
  for (const type of ['pointerdown', 'pointerup', 'click', 'touchstart', 'touchend', 'wheel']) box.addEventListener(type, stop);

  const setOffset = (value) => {
    // 正在走的逐词动画只在开始时校准一次，这里按新的时间让当前行重新开始
    view.setCurrentTime(view.getCurrentTime() + value - offset, true);
    offset = value;
    try { localStorage.setItem(OFFSET_KEY, String(offset)); } catch { /* 只在本次会话生效 */ }
    render(true);
  };
  const button = (label, action) => {
    const el = document.createElement('button');
    el.type = 'button';
    el.textContent = label;
    Object.assign(el.style, { margin: '2px 4px 0 0', padding: '4px 7px', font: 'inherit', color: '#fff', background: 'rgba(255,255,255,.18)', border: '0', borderRadius: '5px' });
    el.addEventListener('click', (event) => { event.stopPropagation(); action(); });
    buttons.append(el);
    return el;
  };
  button('-50', () => setOffset(offset - 50));
  button('-10', () => setOffset(offset - 10));
  button('+10', () => setOffset(offset + 10));
  button('+50', () => setOffset(offset + 50));
  button('偏移归零', () => setOffset(0));
  const seekToggle = button('', () => { view.enableAutoSeekDetection = !view.enableAutoSeekDetection; render(true); });
  const copy = button('复制', async () => {
    try { await navigator.clipboard.writeText(text.textContent); copy.textContent = '已复制'; } catch { copy.textContent = '复制失败'; }
    setTimeout(() => { copy.textContent = '复制'; }, 1500);
  });
  root.append(box);

  const prune = (list, now) => { while (list.length && now - list[0][0] > WINDOW) list.shift(); };
  const values = (list) => list.map((item) => item[1]);

  function sampleAnimations(now, time) {
    const groups = view.currentLyricGroups;
    const highlighted = view.timelineController?.getSnapshot().highlightedGroups;
    if (!groups || !highlighted) return;
    const frameDrifts = [];
    for (const index of highlighted) {
      for (const line of [groups[index]?.mainLine, groups[index]?.bgLine]) {
        if (!line) continue;
        const expected = time - line.getLine().startTime;
        const check = (a) => {
          if (a.pending || a.playState !== 'running') { seen.delete(a); return; }
          const first = !seen.has(a);
          seen.add(a);
          if (!(expected >= 0) || a.playbackRate !== 1 || a.currentTime == null) return;
          const timing = a.effect?.getComputedTiming();
          const end = Number(timing?.delay ?? 0) + Number(timing?.duration ?? 0);
          if (expected >= end) return;
          const drift = a.currentTime - expected;
          if (first) starts.push([now, drift]);
          else frameDrifts.push(drift);
        };
        for (const word of line.splittedWords || []) word.elementAnimations?.forEach(check);
        line.maskAnimator?.animations?.forEach(check);
      }
    }
    if (frameDrifts.length) steady.push([now, median(frameDrifts)]);
  }

  function trackLines(now, time, running) {
    const snapshot = view.timelineController?.getSnapshot();
    const groups = view.currentLyricGroups;
    if (!snapshot || !groups) return;
    const highlighted = new Set(snapshot.highlightedGroups);
    if (running) {
      for (const index of highlighted) {
        const line = groups[index]?.mainLine;
        if (!highlightedBefore.has(index) && line) enters.push([now, time - line.getLine().startTime]);
      }
    }
    highlightedBefore = highlighted;
  }

  function render(force) {
    const now = performance.now();
    if (!force && now - lastRender < 500) return;
    lastRender = now;
    for (const list of [frames, clock, seeks]) prune(list, now);
    const intervals = values(frames);
    const steps = clock.map((item) => item[1]);
    const media = steps.reduce((sum, step) => sum + step.media, 0);
    const wall = steps.reduce((sum, step) => sum + step.wall, 0);
    const deltas = steps.map((step) => Math.abs(step.media - step.wall));
    const current = player.current;
    const transport = current ? player.transport() : null;
    const edge = navigator.userAgent.match(/(EdgA?|Edg|Chrome|Safari|Firefox)\/[\d.]+/)?.[0] || '';
    seekToggle.textContent = `AMLL 跳转检测：${view.enableAutoSeekDetection ? '开' : '关'}`;
    text.textContent = [
      `歌词同步测量 v1  ${current?.mode || '-'} ${current?.source || 'apple'}  ${edge}`,
      `readyState ${transport?.readyState ?? '-'}  ${transport?.paused ? '暂停' : '播放'}${transport?.seeking ? ' 跳转中' : ''}`,
      `帧间隔 平均 ${fmt(intervals.reduce((a, b) => a + b, 0) / (intervals.length || 1))} 最长 ${fmt(Math.max(0, ...intervals))} ms，超过 50ms ${intervals.filter((v) => v > 50).length} 次`,
      `播放进度/页面时钟 ${wall ? (media / wall).toFixed(3) : '-'}，每帧差 中位 ${fmt(median(deltas))} 最大 ${fmt(Math.max(0, ...deltas))} ms，回退 ${steps.filter((s) => s.media < 0).length} 帧，停住 ${steps.filter((s) => s.media === 0).length} 帧`,
      `AMLL 判定跳转 ${seeks.length} 次（3 秒内）`,
      `换行晚了 中位 ${fmt(median(values(enters.slice(-8))))} ms（最近 8 行）`,
      `动画刚开始 中位 ${fmt(median(values(starts.slice(-30))))} 最小 ${fmt(Math.min(...values(starts.slice(-30))))} ms（校准前，负数为落后）`,
      `动画平时 中位 ${fmt(median(values(steady.slice(-180))))} ms`,
      `手动偏移：歌词提前 ${offset} ms`,
    ].join('\n');
  }

  return {
    offset: () => offset,
    frame(timestamp, transport, time, running) {
      const media = transport ? transport.currentTime * 1000 : 0;
      if (previous) {
        frames.push([timestamp, timestamp - previous.timestamp]);
        if (running && previous.running) clock.push([timestamp, { media: media - previous.media, wall: timestamp - previous.timestamp }]);
      }
      previous = { timestamp, media, running };
      trackLines(timestamp, time, running);
      if (running) sampleAnimations(timestamp, time);
      for (const list of [enters, starts, steady]) prune(list, timestamp);
      render(false);
    },
  };
}
