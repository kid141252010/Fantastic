// node scripts/build-jieba-words.cjs /path/to/GMPlayer
const fs = require('node:fs');
const path = require('node:path');
const { gzipSync } = require('node:zlib');
const source = path.join(process.argv[2], 'src-tauri/crates/lyric-segmenter/data');
const output = path.join(__dirname, '../src/ui/lyrics');
async function build() {
  const { cutJieba } = await import('../src/ui/lyrics/word-segmenter.mjs');
  const frequencies = new Map();
  let total = 0;
  let maxLength = 0;
  for (const file of ['jieba-dict.txt', 'lyric-words.txt', 'cutword-words.txt']) {
    for (const line of fs.readFileSync(path.join(source, file), 'utf8').split(/\r?\n/)) {
      const [word, rawFrequency] = line.trim().split(/\s+/);
      if (!word || word.startsWith('#') || (file !== 'jieba-dict.txt' && frequencies.has(word))) continue;
      let frequency;
      if (rawFrequency !== undefined) {
        if (!/^\d+$/.test(rawFrequency)) throw new Error('Invalid frequency');
        frequency = Number(rawFrequency);
      } else {
        // Jieba::add_word(None) calls suggest_freq against the dictionary so far.
        const logTotal = Math.log(total);
        const logFrequency = cutJieba(word, { frequencies, total, maxLength }).reduce((sum, token) => sum + Math.log(frequencies.get(token) ?? 1) - logTotal, 0);
        frequency = Math.max(Math.floor(Math.exp(logFrequency + logTotal)) + 1, frequencies.get(word) ?? 1);
      }
      total += frequency - (frequencies.get(word) ?? 0);
      frequencies.set(word, frequency);
      maxLength = Math.max(maxLength, [...word].length);
    }
  }
  const raw = Buffer.from([...frequencies].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([word, frequency]) => `${word}\t${frequency}`).join('\n') + '\n');
  const compressed = gzipSync(raw, { level: 9 });
  fs.writeFileSync(path.join(output, 'jieba-words.txt.gz'), compressed);
  fs.copyFileSync(path.join(source, 'LICENSE-cutword'), path.join(output, 'LICENSE-cutword.txt'));
  console.log(`${frequencies.size} words; total frequency ${total}; ${raw.length} raw bytes; ${compressed.length} gzip bytes; 杯壁 frequency ${frequencies.get('杯壁')}`);
}
build().catch(error => { console.error(error); process.exitCode = 1; });
