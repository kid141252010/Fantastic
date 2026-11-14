const CJK = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\u30fc\uff70\uff9e\uff9f\u3099\u309a]/u;
const KATAKANA_COST = [8192, 984, 408, 240, 204, 252, 300, 372, 480];
const katakana = ch => /[\u30a1-\u30fa\u30fc-\u30fe\uff66-\uff9f]/u.test(ch);
const HAN = /[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\u{20000}-\u{3134f}]/u;
const KANA_OR_HANGUL = /[\u3040-\u30ff\u31f0-\u31ff\uff66-\uff9f\u1100-\u11ff\u3130-\u318f\uac00-\ud7af]/u;
const LEANS_BACK = ['之外', '之内', '之中', '之间', '之后', '之前', '之上', '之下', '之一', '之余', '之际', '之初', '之时', '之久', '之多', '之类', '之处'];

export async function parseJiebaDictionary(text) {
  const frequencies = new Map();
  let maxLength = 0;
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i++) {
    if (lines[i]) {
      const [word, rawFrequency, extra] = lines[i].split('\t');
      const length = [...word].length;
      const frequency = Number(rawFrequency);
      if (!length || /\s/u.test(word) || !/^\d+$/.test(rawFrequency ?? '') || !Number.isSafeInteger(frequency) || extra !== undefined) throw new Error('Invalid jieba entry');
      frequencies.set(word, frequency);
      maxLength = Math.max(maxLength, length);
    }
    if (i && i % 4096 === 0) await new Promise(resolve => setTimeout(resolve, 0));
  }
  const total = [...frequencies.values()].reduce((sum, frequency) => sum + frequency, 0);
  if (!Number.isSafeInteger(total) || total <= 0) throw new Error('Empty jieba frequencies');
  return { frequencies, total, maxLength };
}

// jieba-rs cut(HMM=false) route for Han: unknown singletons have frequency 1;
// only positions with no dictionary edges get an unknown edge. Longer ties win.
export function cutJieba(text, { frequencies, total, maxLength }) {
  const chars = [...text];
  const scores = new Float64Array(chars.length + 1);
  const next = new Int32Array(chars.length);
  const logTotal = Math.log(total);
  for (let i = chars.length - 1; i >= 0; i--) {
    let word = '';
    let bestEnd = -1;
    let best = -Infinity;
    for (let end = i; end < Math.min(chars.length, i + maxLength); end++) {
      word += chars[end];
      const frequency = frequencies.get(word);
      if (frequency !== undefined) {
        const score = Math.log(frequency) - logTotal + scores[end + 1];
        if (bestEnd < 0 || score > best || (score === best && end + 1 > bestEnd)) {
          best = score;
          bestEnd = end + 1;
        }
      }
    }
    scores[i] = bestEnd < 0 ? -logTotal + scores[i + 1] : best;
    next[i] = bestEnd < 0 ? i + 1 : bestEnd;
  }
  const result = [];
  for (let i = 0; i < chars.length; i = next[i]) result.push(chars.slice(i, next[i]).join(''));
  return result;
}

function* recutSingletons(segments, jieba) {
  const items = [...segments];
  const singleHan = item => [...item.segment].length === 1 && HAN.test(item.segment);
  for (let i = 0; i < items.length;) {
    let end = i;
    while (end < items.length && singleHan(items[end])) end++;
    if (end - i >= 2) {
      let index = items[i].index;
      for (const segment of cutJieba(items.slice(i, end).map(item => item.segment).join(''), jieba)) {
        yield { segment, index, isWordLike: true };
        index += segment.length;
      }
      i = end;
    } else yield items[i++];
  }
}

function* leanBack(text, segments) {
  let previous;
  for (const original of segments) {
    const item = { ...original };
    const before = [...text.slice(0, item.index)].pop() ?? '';
    if (previous && HAN.test(before) && LEANS_BACK.some(prefix => item.segment.startsWith(prefix))) {
      previous.segment += item.segment;
      previous.isWordLike ||= item.isWordLike;
    } else {
      if (previous) yield previous;
      previous = item;
    }
  }
  if (previous) yield previous;
}

export async function parseDictionary(text) {
  const dictionary = new Map();
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line) {
      const tab = line.lastIndexOf('\t');
      const cost = Number(line.slice(tab + 1));
      if (tab <= 0 || !/^\d+$/.test(line.slice(tab + 1)) || cost > 255) throw new Error('Invalid cjdict entry');
      dictionary.set(line.slice(0, tab), cost);
    }
    if (i && i % 4096 === 0) await new Promise(resolve => setTimeout(resolve, 0));
  }
  if (!dictionary.size) throw new Error('Empty cjdict');
  return dictionary;
}

// ICU CjkBreakEngine: NFKC, 20-code-point dictionary matches, unknown cost 255,
// and the Katakana run heuristic. Offsets remain UTF-16 positions in the original text.
function cjkSegments(text, dictionary, base) {
  const chars = [];
  const offsets = [];
  let offset = 0;
  for (const match of text.matchAll(/.[\p{M}\uff9e\uff9f]*/gu)) {
    for (const ch of match[0].normalize('NFKC')) {
      chars.push(ch);
      offsets.push(offset);
    }
    offset += match[0].length;
  }
  offsets.push(text.length);
  const costs = new Float64Array(chars.length + 1).fill(Infinity);
  const previous = new Int32Array(chars.length + 1).fill(-1);
  costs[0] = 0;
  const offer = (start, end, cost) => {
    const total = costs[start] + cost;
    if (total < costs[end]) { costs[end] = total; previous[end] = start; }
  };
  for (let i = 0; i < chars.length; i++) {
    let word = '';
    let single = false;
    for (let j = i; j < Math.min(chars.length, i + 20); j++) {
      word += chars[j];
      const cost = dictionary.get(word);
      if (cost !== undefined) {
        offer(i, j + 1, cost);
        if (j === i) single = true;
      }
    }
    if (!single) offer(i, i + 1, 255);
    if (katakana(chars[i]) && (i === 0 || !katakana(chars[i - 1]))) {
      let end = i + 1;
      while (end < chars.length && end - i < 20 && katakana(chars[end])) end++;
      if (end - i < 20) offer(i, end, KATAKANA_COST[end - i] ?? 8192);
    }
  }
  const boundaries = [text.length];
  for (let end = chars.length; end > 0;) {
    end = previous[end];
    if (end > 0 && offsets[end] !== boundaries[boundaries.length - 1]) boundaries.push(offsets[end]);
  }
  boundaries.push(0);
  boundaries.reverse();
  return boundaries.slice(1).map((end, i) => ({
    segment: text.slice(boundaries[i], end), index: base + boundaries[i], isWordLike: true,
  })).filter(item => item.segment);
}

export function createWordSegmenter(dictionary, native = null, jieba = null) {
  function* segmentRuns(text) {
    let start = 0;
    while (start < text.length) {
      const isCjk = CJK.test(String.fromCodePoint(text.codePointAt(start)));
      let end = start;
      while (end < text.length) {
        const ch = String.fromCodePoint(text.codePointAt(end));
        if (CJK.test(ch) !== isCjk) break;
        end += ch.length;
      }
      const run = text.slice(start, end);
      if (isCjk) yield* cjkSegments(run, dictionary, start);
      else if (native) {
        for (const item of native.segment(run)) yield { segment: item.segment, index: start + item.index, isWordLike: item.isWordLike };
      } else {
        for (const match of run.matchAll(/[\p{L}\p{N}\p{M}_]+|[^\p{L}\p{N}\p{M}_]/gu)) {
          yield { segment: match[0], index: start + match.index, isWordLike: /[\p{L}\p{N}]/u.test(match[0]) };
        }
      }
      start = end;
    }
  }
  return {
    *segment(text) {
      const segments = segmentRuns(text);
      if (jieba && HAN.test(text) && !KANA_OR_HANGUL.test(text)) yield* leanBack(text, recutSingletons(segments, jieba));
      else yield* segments;
    },
  };
}

export function needsDictionary(Segmenter = globalThis.Intl?.Segmenter) {
  try {
    const words = [...new Segmenter('zh', { granularity: 'word' }).segment('生命')];
    return words.length !== 1 || words[0].segment !== '生命' || !words[0].isWordLike;
  } catch { return true; }
}

async function inflate(response) {
  return new Response(response.body.pipeThrough(new DecompressionStream('gzip'))).text();
}

export function createDictionaryLoader({ Segmenter = globalThis.Intl?.Segmenter, fetcher = globalThis.fetch, decompress = inflate } = {}) {
  let pending;
  return async (install, onReady) => {
    if (!needsDictionary(Segmenter)) return false;
    try {
      pending ??= (async () => {
        let native = null;
        try { native = new Segmenter(undefined, { granularity: 'word' }); } catch {}
        const jieba = (async () => {
          const response = await fetcher(new URL('./jieba-words.txt.gz', import.meta.url));
          if (!response.ok) throw new Error('jieba words unavailable');
          return parseJiebaDictionary(await decompress(response));
        })().catch(() => null);
        const response = await fetcher(new URL('./cjdict.tsv.gz', import.meta.url));
        if (!response.ok) throw new Error('cjdict unavailable');
        return createWordSegmenter(await parseDictionary(await decompress(response)), native, await jieba);
      })();
      install(await pending);
      onReady();
      return true;
    } catch { return false; }
  };
}

export const ensureWordSegmenter = createDictionaryLoader();
