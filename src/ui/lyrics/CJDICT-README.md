# ICU CJK dictionary

`cjdict.tsv.gz` is a gzip-compressed UTF-8 `word<TAB>cost` list from
[Unicode ICU cjdict.txt](https://github.com/unicode-org/icu/blob/main/icu4c/source/data/brkitr/dictionaries/cjdict.txt).
It retains every dictionary entry and cost; only the comment header is moved to
`CJDICT-LICENSE.txt` (Unicode, Google/BSD, Libtabe and IPADIC notices).
Input SHA256: `e73fd72048981d0cc13e9dc436a7eaba07ffb6eff58c8a59dc75c1df746663a0`.

Rebuild: `node scripts/build-cjdict.cjs /path/to/cjdict.txt`.
The runtime follows ICU `CjkBreakEngine` word mode: NFKC normalization,
20-code-point dictionary matches, minimum summed costs, unknown cost 255 and
Katakana run costs. Non-CJK runs use the native segmenter where available.

The asset is deliberately served as binary gzip bytes (not HTTP Content-Encoding),
then decompressed using `DecompressionStream` in the browser. Both Rust and
serverless deployments use the same content-hashed, immutable asset URL; Rust
embeds only the compressed bytes, like its existing wasm and HRTF assets.
No dictionary download occurs if the native segmenter recognizes `生命` as one
word. Fetch/decompression failures retain native behavior. Parsing yields between
batches to avoid blocking lyric playback. The injected segmenter is process-wide;
the current lyrics are rebuilt after it becomes ready.
