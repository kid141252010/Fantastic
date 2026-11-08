// node scripts/build-cjdict.cjs /path/to/icu/icu4c/source/data/brkitr/dictionaries/cjdict.txt
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { gzipSync } = require('node:zlib');
const input = fs.readFileSync(process.argv[2]);
const lines = input.toString('utf8').replace(/^\uFEFF/, '').split(/\r?\n/);
const first = lines.findIndex(line => /^\S+\t\d+$/.test(line));
if (first < 0) throw new Error('No dictionary entries');
const entries = lines.slice(first).filter(line => line.trim());
for (const entry of entries) {
  if (!/^\S+\t\d+$/.test(entry) || Number(entry.split('\t')[1]) > 255) throw new Error('Invalid dictionary entry');
}
const output = path.join(__dirname, '../src/ui/lyrics');
const data = Buffer.from(entries.join('\n') + '\n');
const compressed = gzipSync(data, { level: 9 });
fs.writeFileSync(path.join(output, 'cjdict.tsv.gz'), compressed);
fs.writeFileSync(path.join(output, 'CJDICT-LICENSE.txt'), lines.slice(0, first).join('\n') + '\n');
console.log(`${entries.length} entries; ${data.length} raw bytes; ${compressed.length} gzip bytes; source SHA256 ${createHash('sha256').update(input).digest('hex')}`);
