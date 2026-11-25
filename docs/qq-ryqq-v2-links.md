# QQ 官方链接双版本识别审计

## 结果与边界

五类页面 `songDetail`、`albumDetail`、`singer`、`playlist`、`mv/v` 均接受 `/n/ryqq/` 与 `/n/ryqq_v2/`。既有匹配器只将版本段改为 `ryqq(?:_v2)?`，其余协议斜杠、尾斜杠、查询参数、锚定规则保持各调用点原状。MID 仍为原有 1–32 位 ASCII 字母数字；Rust 继续使用 `qq::valid_mid`，没有修改该函数。

`songPath`、`albumPath`、`artistPath`、`playlistPath`、`mvPath` 及其他生成点继续输出旧版规范路径。新版输入可以保留在地址栏、webUrl 返回值和资料库官方 URL 字段中；不迁移已存数据，不改已有旧版链接。

首页原本没有 QQ 直接粘贴解析分支，旧版 QQ 链接同样无法直接提交打开。本次新增 `qq.parseWebLink`，复用 `webUrl` 的完整页面校验，并接入当前选择 QQ 音源时的首页输入框；识别五类提示标签，提交直接导航，识别成功时不再请求搜索联想。原有 Apple／NCM 分支保持原行为。

## 审计方式与逐点清单

在仓库根目录运行：

```sh
rg -n --hidden -i -g '!.git' -g '!target' -g '!node_modules' -g '!dist' 'ryqq' .
git grep -n -i ryqq
```

下列清单覆盖本报告外 83 个命中行、21 个文件（同一行多个 URL 合并列示）。行号以本次修改后的工作树为准。本报告自身的说明、路径和文件名不再递归列入审计。未发现其他旧版专用匹配点。

### 运行时代码与注释

| 位置（当前工作树） | 判定 | 处理结果 |
| --- | --- | --- |
| `serverless/core.mjs:81` | 匹配 | isPagePath 的五类 QQ 页面匹配改为双版本；其余校验不变。 |
| `src/links.rs:42` | 匹配 | is_qq_link 的版本段放宽；继续调用原有 qq::valid_mid。 |
| `vercel.json:28` | 匹配／重写 | QQ SPA 重写规则版本段放宽；仍重写到 /index.html，不收紧原有 .+ 范围。 |
| `web/app.mjs:96` | 匹配 | 歌曲路由支持双版本，返回 qq-song。 |
| `web/app.mjs:97` | 匹配 | 歌手路由支持双版本，返回 qq-artist。 |
| `web/app.mjs:98` | 匹配 | 专辑路由支持双版本，返回 qq-album。 |
| `web/app.mjs:99` | 匹配 | 歌单路由支持双版本，返回 qq-playlist。 |
| `web/app.mjs:100` | 匹配 | MV 路由支持双版本，返回 qq-mv。 |
| `web/library.mjs:59` | 匹配 | QQ_PATH 接受五类双版本 URL；保留原有 MID、尾斜杠规则。 |
| `web/library.mjs:72` | 生成 | 旧 /qq-* 别名导入时仍生成旧版规范 URL；不迁移已存官方 URL。 |
| `web/player/player.js:540` | 生成 | 恢复 QQ 队列信息时生成旧版歌曲链接；保持不变。 |
| `web/player/player.js:3335` | 生成 | 歌词／播放界面歌曲链接生成；保持旧版。 |
| `web/sources/qq.mjs:142` | 生成 | songPath 保持旧版规范歌曲路径。 |
| `web/sources/qq.mjs:150` | 生成 | albumPath 保持旧版规范专辑路径。 |
| `web/sources/qq.mjs:151` | 生成 | artistPath 保持旧版规范歌手路径。 |
| `web/sources/qq.mjs:152` | 生成 | playlistPath 保持旧版规范歌单路径。 |
| `web/sources/qq.mjs:153` | 生成 | mvPath 保持旧版规范 MV 路径。 |
| `web/sources/qq.mjs:157` | 匹配 | webUrl 校验五类双版本页面；返回输入版本的官网 URL，保留查询参数。 |
| `web/sources/qq.mjs:191` | 匹配 | toTrack 从 artist.href 回填歌手 MID 时接受双版本。 |
| `web/sources/qq.mjs:192` | 生成 | toTrack 生成歌手 href，保持旧版。 |
| `web/sources/qq.mjs:198` | 生成 | toTrack 生成专辑 href，保持旧版；歌曲仍调用 songPath。 |
| `web/views/actions.mjs:576` | 匹配 | 专辑／歌单操作菜单提取 ID 时接受双版本；保留原有非尾锚定规则。 |
| `web/views/home.mjs:113` | 匹配（新增） | QQ 粘贴识别提示接受双版本，并映射歌曲、专辑、歌手、歌单、MV 标签。 |
| `web/views/qq-album.mjs:13` | 匹配 | 专辑视图从双版本 pathname 提取 albumMid。 |
| `web/views/qq-album.mjs:42` | 生成 | 专辑页歌手链接保持旧版。 |
| `web/views/qq-artist.mjs:16` | 匹配 | 歌手视图从双版本 pathname 提取 singerMid。 |
| `web/views/qq-artist.mjs:126` | 生成 | 歌手页曲目中的专辑链接保持旧版。 |
| `web/views/qq-artist.mjs:137` | 生成 | 歌手页操作对象中的 albumHref 保持旧版。 |
| `web/views/qq-artist.mjs:204` | 生成 | 歌手页专辑卡片链接保持旧版。 |
| `web/views/qq-mv.mjs:1` | 说明（规范路径示例） | 保留旧版 MV 路径的注释示例。 |
| `web/views/qq-mv.mjs:12` | 匹配 | MV 视图从双版本 pathname 提取 vid。 |
| `web/views/qq-playlist.mjs:1` | 说明（规范路径示例） | 保留旧版歌单路径的注释示例。 |
| `web/views/qq-playlist.mjs:15` | 匹配 | 歌单视图从双版本 pathname 提取 id。 |
| `web/views/qq-song.mjs:31` | 匹配 | 歌曲视图从双版本 pathname 提取 mid。 |

另有不直接含版本字面量、但影响链接处理的入口：

| 位置 | 判定 | 处理结果 |
| --- | --- | --- |
| `src/handlers.rs:32` | 匹配分派 | fallback_handler 已调用 is_qq_link；无需再加一套正则，补了五类双版本外壳测试。 |
| `web/sources/qq.mjs:161` | 粘贴解析（新增） | parseWebLink 复用 webUrl 的校验与协议斜杠规范化，不新增另一套 MID 规则。 |
| `web/views/home.mjs:81` | 粘贴解析分派 | 当前 QQ 音源调用 qq.parseWebLink。 |
| `web/views/home.mjs:143` | 识别后的联想控制 | 已识别的 QQ 链接不发搜索联想请求。 |

### 测试中的字面量

| 位置（当前工作树） | 判定 | 处理结果 |
| --- | --- | --- |
| `src/handlers.rs:428` | 匹配测试（双版本） | 新增 fallback_handler 的五类双版本外壳返回及无效 MID 反例。 |
| `src/handlers.rs:449` | 匹配测试（旧反例） | 保留旧版非法路径返回 404 且不缓存的断言。 |
| `src/handlers.rs:455` | 匹配测试（旧正例） | 保留旧版五类页面返回外壳的断言。 |
| `src/links.rs:173` | 匹配测试（旧正例） | 保留五类旧版单斜杠、尾斜杠路径断言。 |
| `src/links.rs:175` | 匹配测试（旧反例） | 保留旧别名、空 MID、非法字符、双尾斜杠反例。 |
| `src/links.rs:198` | 匹配测试（双版本） | 新增五类页面 × 两个版本 × 两种协议斜杠形式及边界矩阵。 |
| `src/links.rs:213` | 匹配测试（反例） | 新增未知版本反例。 |
| `tests/unit/qq_source.mjs:45` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:46` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:61` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:62` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:916` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:917` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:918` | 生成测试（既有） | 保留 toTrack／资料库歌曲、专辑、歌手旧版规范链接断言。 |
| `tests/unit/qq_source.mjs:945` | 生成测试（既有） | 保留旧别名导入后生成五类旧版规范链接的断言。 |
| `tests/unit/qq_source.mjs:946` | 生成测试（既有） | 保留旧别名导入后生成五类旧版规范链接的断言。 |
| `tests/unit/qq_source.mjs:948` | 生成测试（既有） | 保留旧别名导入后生成五类旧版规范链接的断言。 |
| `tests/unit/qq_source.mjs:949` | 生成测试（既有） | 保留旧别名导入后生成五类旧版规范链接的断言。 |
| `tests/unit/qq_source.mjs:963` | 匹配测试（既有正例） | 保留旧版 webUrl 正例及返回原官网 URL 的断言。 |
| `tests/unit/qq_source.mjs:964` | 匹配测试（既有正例） | 保留旧版 webUrl 正例及返回原官网 URL 的断言。 |
| `tests/unit/qq_source.mjs:965` | 匹配测试（既有正例） | 保留旧版 webUrl 正例及返回原官网 URL 的断言。 |
| `tests/unit/qq_source.mjs:966` | 匹配测试（既有反例） | 保留旧版路径遍历反例。 |
| `tests/unit/qq_source.mjs:977` | 生成测试（新增） | 断言五个路径生成函数仍输出旧版规范路径。 |
| `tests/unit/qq_source.mjs:978` | 匹配测试（双版本） | 新增 webUrl／parseWebLink 的五类双版本、斜杠形式、查询参数、MID 边界矩阵。 |
| `tests/unit/qq_source.mjs:995` | 匹配测试（新增反例） | 拒绝 HTTP、错误域名、未知版本、fragment 和未知页面类型。 |
| `tests/unit/qq_source.mjs:996` | 匹配测试（新增反例） | 拒绝 HTTP、错误域名、未知版本、fragment 和未知页面类型。 |
| `tests/unit/qq_source.mjs:997` | 匹配测试（新增反例） | 拒绝 HTTP、错误域名、未知版本、fragment 和未知页面类型。 |
| `tests/unit/qq_source.mjs:1003` | 匹配测试（双版本） | 新增 toTrack 的歌手 href 回填及非法 MID 反例。 |
| `tests/unit/qq_source.mjs:1005` | 生成测试（新增） | 断言新版歌手 href 回填后仍生成旧版规范歌手路径。 |
| `tests/unit/qq_source.mjs:1013` | 匹配测试（双版本） | 新增资料库条目与歌单内曲目导入，五类链接均保留输入版本。 |
| `tests/unit/qq_ui.mjs:20` | 匹配测试（既有夹具） | 保留旧版歌曲详情默认路径；原有 UI 断言继续执行。 |
| `tests/unit/qq_ui.mjs:63` | 匹配测试（双版本） | 真实 route 函数覆盖五类双版本、协议斜杠、尾斜杠、查询参数与非法 MID。 |
| `tests/unit/qq_ui.mjs:77` | 匹配测试（新增反例） | 拒绝未知路径版本或其他站点 origin。 |
| `tests/unit/qq_ui.mjs:78` | 匹配测试（新增反例） | 拒绝未知路径版本或其他站点 origin。 |
| `tests/unit/qq_ui.mjs:81` | 匹配测试（双版本） | 运行五个真实视图主体与 QQ adapter，验证提取的 MID、请求地址、渲染及播放。 |
| `tests/unit/qq_ui.mjs:133` | 生成测试（新增） | 确认双版本视图后续生成的歌曲、歌手、专辑导航仍为旧版。 |
| `tests/unit/qq_ui.mjs:135` | 生成测试（新增） | 确认双版本视图后续生成的歌曲、歌手、专辑导航仍为旧版。 |
| `tests/unit/qq_ui.mjs:136` | 生成测试（新增） | 确认双版本视图后续生成的歌曲、歌手、专辑导航仍为旧版。 |
| `tests/unit/qq_ui.mjs:148` | 匹配测试（双版本） | 运行真实 qqCollectionEntries，验证专辑／歌单操作使用正确 ID。 |
| `tests/unit/qq_ui.mjs:181` | 匹配测试（双版本） | 运行真实首页主体，验证五类粘贴识别标签与提交导航，不触发搜索或联想。 |
| `tests/unit/serverless.mjs:89` | 匹配测试（既有正例） | 保留旧版五类服务端页面正例。 |
| `tests/unit/serverless.mjs:92` | 匹配测试（既有正例） | 保留旧版五类服务端页面正例。 |
| `tests/unit/serverless.mjs:100` | 匹配测试（既有反例） | 保留旧版无效页面／MID 反例。 |
| `tests/unit/serverless.mjs:101` | 匹配测试（既有反例） | 保留旧版无效页面／MID 反例。 |
| `tests/unit/serverless.mjs:116` | 匹配／重写测试（双版本） | 新增 isPagePath 与 Vercel 重写 source 的五类双版本矩阵，保留 MID 边界。 |
| `tests/unit/serverless.mjs:134` | 匹配／重写测试（新增反例） | 未知版本不应匹配页面或 SPA 重写。 |

### 既有文档中的字面量

| 位置（当前工作树） | 判定 | 处理结果 |
| --- | --- | --- |
| `README.md:24` | 说明（规范生成示例） | 保留旧版歌曲 URL 示例；不是匹配器。 |
| `README.zh-CN.md:24` | 说明（规范生成示例） | 保留旧版歌曲 URL 示例；不是匹配器。 |
| `docs/ncm-canonical-path.md:26` | 历史说明 | 保留 NCM 改动当时的范围记录；本次 QQ 粘贴入口的变化以本报告为准。 |

## 测试覆盖与实际结果

新增测试在 `tests/unit/qq_source.mjs`、`tests/unit/qq_ui.mjs`、`tests/unit/serverless.mjs` 和 `src/links.rs` 中均覆盖五类页面 × 两种版本正例，并保留原有旧版断言；另在 `src/handlers.rs` 补充服务端外壳返回测试。相关矩阵还覆盖协议单／双斜杠、尾斜杠、查询参数、32 字符边界、非法字符、33 字符 ID、未知版本和多余路径等适用情形。

UI 测试使用仓库现有 DOM 替身：歌曲直接挂载真实模块，其他视图及首页执行真实主体并注入浏览器控件依赖；QQ adapter 和发出的请求路径仍为真实实现。Vercel 测试读取实际配置 source 进行路径匹配验证，未进行线上部署验证。

| 命令 | 结果 |
| --- | --- |
| `node tests/run.mjs unit` | 已全量执行：25 个测试文件中 24 个通过；serverless 文件因下述 EPERM 失败，退出码 1。 |
| `node tests/unit/qq_source.mjs` | 51 项全部通过，含新矩阵与原有断言。 |
| `node tests/unit/qq_ui.mjs` | 22 项全部通过，含路由、五个视图、首页粘贴与操作菜单。 |
| `node tests/unit/serverless.mjs` | 16 项通过，1 项静态构建子进程测试受 EPERM 限制；新增双版本矩阵通过。 |
| `node scripts/build-static.mjs /tmp/qq-ryqq-v2-static` | 独立运行成功，生成 307 个文件。 |
| `cargo test --workspace` | 已执行，退出码 101；fantastic 库阶段 152 项通过、12 项回环绑定权限失败，随后停止。两项新增 QQ 测试与原有 prefixed_pages 均通过。 |
| `cargo test --workspace --no-fail-fast` | 继续执行全部 target：222 项通过、33 项因回环绑定权限失败、1 项原有忽略；退出码 101。 |
| `cargo test --workspace qq_official_page_versions` | 新增 is_qq_link 与 fallback_handler 两项测试全部通过，退出码 0。 |
| `git diff --check` | 通过。 |

### 沙箱限制

Node 失败项为 `tests/unit/serverless.mjs:560` 的 `build-static lays out web the way src/assets.rs serves it`：`execFileSync` 调用 `spawnSync /usr/bin/node` 返回 `EPERM`（errno -1）。相同构建脚本直接执行成功；未修改或绕过该测试断言。

Rust 所有失败均发生在回环 mock 的监听绑定，错误为 `Operation not permitted (os error 1)`／`PermissionDenied`。补跑 no-fail-fast 后的受限 target：

| target | 通过 | 失败 | 原因 |
| --- | ---: | ---: | --- |
| fantastic lib | 152 | 12 | NCM 1 项、QQ 7 项、wrapper 4 项无法绑定回环 mock。 |
| client_limit | 0 | 5 | 回环监听绑定受限。 |
| local_wrapper | 1 | 2 | 回环监听绑定受限。 |
| mv_api | 0 | 5 | 回环监听绑定受限。 |
| qq_upstream | 0 | 9 | 回环监听绑定受限。 |

本次使用默认测试配置，未设置 `FANTASTIC_TEST_SKIP_LOOPBACK`，回环受限的测试按失败记录。需在沙箱外复跑原定两条全量命令：

```sh
node tests/run.mjs unit
cargo test --workspace
```

工作分支保持 `feat/ncm-source`；未创建 commit。

