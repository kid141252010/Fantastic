# AMLL lyric player

The lyrics view uses [Apple Music-like Lyrics](https://github.com/kid141252010/applemusic-like-lyrics)
(`@applemusic-like-lyrics/core`, the source lock in `VERSION`, AGPL-3.0-only, see `LICENSE`):
`DomLyricPlayer` renders the lyrics, and `BackgroundRender` plus the Mesh, Isolation, and
Titanium renderers draw the selectable backgrounds. `PixiRenderer` is deliberately excluded.

The source is the user's fork, branch `master`: upstream `amll-dev/main` plus unmerged
changes for opt-in CJK–Latin text spacing and improved CJK / mixed-script line breaking.

`scripts/build-amll.cjs` bundles `entry.mjs` with esbuild into `src/ui/lyrics/amll-core.mjs`
and copies the package stylesheet to `src/ui/lyrics/amll.css`. Its dependencies are bundled too
(gl-matrix MIT, bezier-easing MIT, @ungap/structured-clone ISC, deep-freeze public domain).
`pixi-stub.mjs` replaces Pixi's imports, so nothing from Pixi ends up in the bundle.

`VERSION` records both the package version and the exact AMLL source revision as
`<package-version>+git.<short-commit>`, currently `0.6.0+git.f36b5ff`. The installed core's
`package.json` version must match the part before `+`; the source commit is recorded in the
generated JavaScript and CSS banners.

To rebuild from the pinned AMLL source:

    git clone --branch master https://github.com/kid141252010/applemusic-like-lyrics.git
    cd applemusic-like-lyrics
    git checkout f36b5ff6c421fb969afda56bd3c7bb169609299d
    rm -rf packages/core/dist
    rm -f packages/core/*.tgz
    pnpm install --filter @applemusic-like-lyrics/core
    pnpm --filter @applemusic-like-lyrics/core run build-only
    cd packages/core
    pnpm pack
    mkdir /tmp/amll-build-input
    cd /tmp/amll-build-input
    npm init -y
    npm install --legacy-peer-deps /path/to/applemusic-like-lyrics/packages/core/applemusic-like-lyrics-core-0.6.0.tgz esbuild
    node /path/to/fantastic/scripts/build-amll.cjs /tmp/amll-build-input/node_modules

The last command regenerates the two checked-in files in `src/ui/lyrics/`.

The full-screen player's layout, icons and animations follow AMLL's
[`react-full`](https://github.com/amll-dev/applemusic-like-lyrics/tree/main/packages/react-full) package
(same license). It is not bundled, since it needs React, framer-motion and jotai: `src/ui/lyrics/panel.mjs` carries its SVG
icons and reimplements the horizontal/vertical layouts, the spring transitions (sampled into CSS `linear()` easings),
the bouncing slider, media-button and thumb animations in plain DOM; the styles are the "全屏播放界面" section of `src/ui/app.css`.

The app does not need Node at runtime. Apple TTML is still parsed by `src/ui/lyrics/ttml.mjs`
(translations and pronunciations come from the iTunes metadata); `panel.mjs` converts that
model to AMLL `LyricLine`s.
