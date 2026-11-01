# QQ 音源规格

本文只描述当前实现的接口和运行约束。

## 配置与请求路径

QQ 是可选服务端音源。设置 `--qq-api` / `AM_HOOK_QQ_API` 后启用；可选的 `--qq-proxy` / `AM_HOOK_QQ_PROXY` 只用于服务端 QQ API、AMLL、封面和媒体请求。代理地址和凭据不会进入页面配置。未设置 QQ API 时所有 `/qq/*` 路由返回 404。

主要接口如下：

| 方法 | 路径 | 行为 |
| --- | --- | --- |
| GET | `/qq/search?q=&limit=&offset=` | 搜索；`limit` 归一化到 1–100，`offset` 最大 10000 |
| GET | `/qq/song/:mid` | 白名单歌曲 DTO |
| GET | `/qq/song/:mid/play?quality=` | 校验档位后返回 302；浏览器直连 QQ CDN |
| GET | `/qq/song/:mid/download?quality=` | 服务端流式转发，限制 CDN 主机和响应头 |
| GET | `/qq/song/:mid/qualities[?basic=1]` | 读取 info 尺寸；完整模式才探测未映射档位 |
| GET | `/qq/song/:mid/lyrics` | AMLL → TTML → QRC → LRC 候选 |
| GET | `/qq/song/:mid/lyrics/raw?kind=qrc\|qrctrans\|qrcroma` | 返回同一缓存中的原始 QRC 字段 |
| GET | `/qq/cover/:albumMid` | 只请求固定 QQ 封面模板，禁止重定向 |
| GET | `/qq/capabilities` | 返回 [`qq-quality-validation.md`](qq-quality-validation.md) 定义的静态 20 档表 |

QQ 搜索、音质查询和下载没有客户端限流；JOC 的 DAEAC3 播放仍使用同一个 `/download` 路径。

## 播放与安全边界

未知档位在请求上游前即被拒绝。播放响应只含 302 和白名单 CDN `Location`，媒体字节由浏览器直接获取；服务端 QQ 代理只影响服务端请求。下载客户端最多跟随 5 次重定向，且每一跳必须是 HTTPS、无端口/凭据、主机为 `*.qq.com`、`qq.com` 或 `*.gtimg.cn`。歌曲 mid、专辑 mid、封面 URL、音频 URL、歌词字段均经过白名单或清洗；DTO 不携带上游私有字段。

歌词 TTML 仅保留允许的标签、时间属性和 `x-bg` / `x-translation` / `x-roman` 角色，拒绝 DOCTYPE/ENTITY、外部资源和超大文档。QRC 输入支持明文字符串或 `LyricContent` 包装；没有解密逻辑。歌词来源成功、明确没有内容、失败三态分别处理：部分失败结果只缓存 5 分钟，全部失败返回 502，全部明确没有内容缓存 1 小时，正常成功缓存 7 天。

## 测试范围

`tests/qq_upstream.rs` 和 `tests/qq_adapter/mod.rs` 使用随机 loopback 端口的 axum mock 覆盖搜索解析、歌词来源与缓存、audio URL / download 上游错误及媒体重定向白名单；纯 DTO、解析和安全约束测试保留在 QQ 单元测试中。
