const CJK = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\u30fc\uff70\uff9e\uff9f\u3099\u309a]/u;
const KATAKANA_COST = [8192, 984, 408, 240, 204, 252, 300, 372, 480];
const katakana = ch => /[\u30a1-\u30fa\u30fc-\u30fe\uff66-\uff9f]/u.test(ch);

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

export function createWordSegmenter(dictionary, native = null) {
  return {
    *segment(text) {
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
        const response = await fetcher(new URL('./cjdict.tsv.gz', import.meta.url));
        if (!response.ok) throw new Error('cjdict unavailable');
        return createWordSegmenter(await parseDictionary(await decompress(response)), native);
      })();
      install(await pending);
      onReady();
      return true;
    } catch { return false; }
  };
}

export const ensureWordSegmenter = createDictionaryLoader();
