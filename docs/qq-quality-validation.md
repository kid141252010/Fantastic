# QQ 音质与 capabilities 规格

本文是 QQ 音质接口的当前契约。档位、编码和播放分类由 Rust `Quality` / `Codec` 定义生成；前端启动时消费本接口的静态档位表。

## `GET /qq/capabilities`

启用 QQ 时返回：

```json
{
  "enabled": true,
  "qualities": [
    {
      "quality": "HQ",
      "defaultCodec": "mp3",
      "contentType": "audio/mpeg",
      "playbackClass": "native",
      "sizeField": "size_320mp3",
      "displayName": "MP3 320 kbps"
    }
  ]
}
```

`qualities` 固定按下表顺序返回 20 项。`sizeField: null` 表示 info 响应没有可直接确认该档位大小的字段，完整 `/qualities` 会按需探测；`basic=1` 只使用有已验证 size 字段的档位。`defaultCodec`、`contentType`、`displayName` 是静态默认值；完整 qualities 行仍会以实际 audio URL 的 allowlist 元数据补充编码和大小。

AC-4 的三个档位使用中性的 `playbackClass: "ac-4-probe"`。浏览器能力探测由后续前端消费批次叠加；`DAEAC3` 的 `joc` 是静态 E-AC-3 JOC 管线分类。

| quality | defaultCodec | contentType | playbackClass | sizeField | displayName |
| --- | --- | --- | --- | --- | --- |
| MP3128 | mp3 | audio/mpeg | native | size_128mp3 | MP3 128 kbps |
| AAC48 | aac | audio/mp4; codecs="mp4a.40.2" | native | size_48aac | AAC 48 kbps |
| WEB | aac | audio/mp4; codecs="mp4a.40.2" | native | size_96aac | AAC 96 kbps |
| AAC192 | aac | audio/mp4; codecs="mp4a.40.2" | native | size_192aac | AAC 192 kbps |
| OGG96 | vorbis | audio/ogg; codecs="vorbis" | native | size_96ogg | Ogg 96 kbps |
| OGG192 | vorbis | audio/ogg; codecs="vorbis" | native | size_192ogg | Ogg 192 kbps |
| HQ | mp3 | audio/mpeg | native | size_320mp3 | MP3 320 kbps |
| NAC | nac | audio/x-nac | download | null | AICodec |
| SQ | flac | audio/flac | native | size_flac | FLAC |
| RS | flac | audio/flac | native | size_hires | FLAC Hi-Res |
| QAI | flac | audio/flac | native | null | FLAC |
| Q360V1 | flac | audio/flac | native | null | FLAC |
| Q360V2 | flac | audio/flac | native | null | FLAC |
| RA360 | unknown | application/octet-stream | download | size_360ra | 360 Reality Audio |
| DTS | dts | audio/vnd.dts | download | size_dts | DTS |
| DTSX | dtsx | audio/vnd.dts.hd; codecs="dtsx" | download | null | DTS:X |
| DA | ac-4 | audio/mp4; codecs="ac-4" | ac-4-probe | size_dolby | AC-4 Dolby Atmos |
| DAEAC3 | ec-3 | audio/mp4; codecs="ec-3" | joc | size_dolby | E-AC-3 JOC |
| DA448 | ac-4 | audio/mp4; codecs="ac-4" | ac-4-probe | null | AC-4 A-JOC 448 kbps |
| DA1500 | ac-4 | audio/mp4; codecs="ac-4" | ac-4-probe | null | AC-4 A-JOC 1500 kbps |

## Quality and playback endpoints

浏览器加载 QQ source 模块时请求一次 `/qq/capabilities`。响应必须包含完整的 20 项以及 `defaultCodec`、`contentType`、`playbackClass`、`sizeField`；缺失、格式不完整或请求失败时使用前端内置的同形状回退表。`playbackInfo` 以这张表为基础，再叠加运行时能力：E-AC-3 始终进入 JOC 管线，AC-4 通过共享的 `supportsMime`（包含 `ManagedMediaSource`）探测，其他档位使用静态的 native/download 分类。完整 qualities 行仍可提供实际 URL 的 allowlist 编码和 MIME 覆盖；详情页继续使用完整 qualities，队列默认选择使用 `basic=1`。

`/qq/song/:mid/qualities?basic=1` uses the cached `/extra/music/info` `file` object and never probes unknown tiers. The full endpoint may query `/extra/music/url` for tiers without a verified size mapping; probes use a separate semaphore and short negative cache. `/play` returns a 302 for any available URL. Native rows use `/play`; DAEAC3 JOC uses `/download?quality=DAEAC3` and the existing browser JOC pipeline; download-only rows remain download actions.

## Validation

The Rust unit suite checks the generated JSON, every quality mapping, QRC/LRC conversion, TTML allowlisting, cache states and URL restrictions. `tests/qq_upstream.rs` adds local axum upstream integration coverage. Browser tests cover capabilities fallback/consumption, shared MIME detection, favorites caching, fixed HQ fallback, source navigation, and JOC download progress.
