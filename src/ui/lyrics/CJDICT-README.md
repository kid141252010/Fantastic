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

## Chinese singleton-run refinement

`jieba-words.txt.gz` contains deduplicated UTF-8 `word<TAB>frequency` entries,
including single-character words, from GMPlayer's `jieba-dict.txt`, `cutword-words.txt` and
`lyric-words.txt` at commit `cd7d25593ad13ed2a5bc4b828288f33ffadb8f98`:
https://github.com/Burial0268/GMPlayer/tree/cd7d25593ad13ed2a5bc4b828288f33ffadb8f98/src-tauri/crates/lyric-segmenter/data

Original reference: https://github.com/Burial0268/GMPlayer/commit/c68dc42af21245d75674493d9a165c7e7f9bc040

- jieba's default dictionary (via jieba-rs 0.11.0): MIT, see `JIEBA-LICENSE.txt`.
- cutword additions selected by GMPlayer: Apache-2.0, see `LICENSE-cutword.txt`.
- GMPlayer's lyric words and the LEANS_BACK approach: AGPL-3.0, attributed to
  Burial0268/GMPlayer; see this project's root `LICENSE` for the same license text.

Rebuild: `node scripts/build-jieba-words.cjs /path/to/GMPlayer` at that revision.
Both gzip assets are fetched in parallel in the existing one-shot fallback loader.
If the optional Jieba asset fails, cjdict-only segmentation is still installed.
Native-capable browsers still fetch neither dictionary.

For lines with Han and no kana/Hangul (GMPlayer's exact ranges), only maximal runs
of two or more consecutive single-Han cjdict segments are re-cut. Existing
multi-character cjdict segments remain intact. The Jieba DAG route maximizes
`log(frequency) - log(total frequency) + best suffix score`, with longer candidates
winning exact ties. An unknown single character has frequency 1 and is offered
only when no dictionary candidate starts there. HMM is disabled. This follows
jieba-rs 0.11.0's `calc` / `cut(false)`, not GMPlayer's all-candidate span union.

The generator preserves all base frequencies and adds missing lyric/cutword words
in GMPlayer's order (lyric words first). Explicit frequencies are preserved;
missing ones use `suggest_freq`: max(floor(exp(sum(log(token frequency or 1) -
log(total)) + log(total))) + 1, existing frequency or 1), using the dictionary so
far. The loaded total includes all entries, including single-character words.
At the pinned revision there are 352,825 entries, total frequency 60,925,619;
the frequency-less lyric addition `杯壁` gets frequency 1.

Finally a segment beginning with GMPlayer's `LEANS_BACK` postpositions joins the
preceding segment when immediately preceded by Han. This explicit postposition
rule is the only exception to preserving multi-character cjdict boundaries.
Kana/Hangul lines retain the previous segmentation. AMLL still uses soft
boundary penalties: a protected word is not an absolute prohibition on wrapping.
