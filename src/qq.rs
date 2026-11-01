//! Server-only QQ adapter. Only public QQ artwork URLs enter DTOs.
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use lru::LruCache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    num::NonZeroUsize,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

type ApiResult<T> = Result<T, QqError>;
pub struct QqError(StatusCode);
impl IntoResponse for QqError {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(json!({"error": match self.0 {
                StatusCode::BAD_REQUEST => "Invalid QQ request",
                StatusCode::NOT_FOUND => "QQ resource not found",
                _ => "QQ source temporarily unavailable",
            }})),
        )
            .into_response()
    }
}
fn bad() -> QqError {
    QqError(StatusCode::BAD_REQUEST)
}
fn missing() -> QqError {
    QqError(StatusCode::NOT_FOUND)
}
fn unavailable() -> QqError {
    QqError(StatusCode::BAD_GATEWAY)
}
fn network(error: reqwest::Error) -> QqError {
    // reqwest Display otherwise includes the full private API URL.
    tracing::warn!(error = %error.without_url(), "qq network request failed");
    unavailable()
}

enum RequestError {
    Upstream(StatusCode),
    Network,
    Decode,
}

impl RequestError {
    fn api(self) -> QqError {
        unavailable()
    }
}
pub fn valid_mid(mid: &str) -> bool {
    !mid.is_empty() && mid.len() <= 32 && mid.bytes().all(|b| b.is_ascii_alphanumeric())
}
fn check_mid(mid: &str) -> ApiResult<()> {
    if valid_mid(mid) {
        Ok(())
    } else {
        Err(bad())
    }
}
fn cover_url(album_mid: &str) -> ApiResult<String> {
    check_mid(album_mid)?;
    Ok(format!(
        "https://y.qq.com/music/photo_new/T002R500x500M000{album_mid}_5.jpg"
    ))
}
fn cdn_url(raw: &str) -> Option<reqwest::Url> {
    let url = reqwest::Url::parse(raw).ok()?;
    (url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url
            .host_str()
            .is_some_and(|h| h.ends_with(".qq.com") || h == "qq.com" || h.ends_with(".gtimg.cn")))
    .then_some(url)
}

pub(crate) fn outbound_proxy(proxy: Option<&str>) -> Result<Option<reqwest::Proxy>, String> {
    let Some(proxy) = proxy.map(str::trim).filter(|url| !url.is_empty()) else {
        return Ok(None);
    };
    let url = reqwest::Url::parse(proxy).map_err(|e| format!("Invalid QQ proxy: {e}"))?;
    // reqwest accepts unknown schemes here and silently ignores them when routing.
    if !url.has_host()
        || !matches!(
            url.scheme(),
            "http" | "https" | "socks4" | "socks4a" | "socks5" | "socks5h"
        )
    {
        return Err("Invalid QQ proxy: unsupported scheme or missing host".into());
    }
    reqwest::Proxy::all(url)
        .map(Some)
        .map_err(|e| format!("Invalid QQ proxy: {}", e.without_url()))
}

struct Cache<T> {
    entries: Mutex<LruCache<String, (Instant, T)>>,
    ttl: Duration,
}
impl<T: Clone> Cache<T> {
    fn new(size: usize, seconds: u64) -> Self {
        Self {
            entries: Mutex::new(LruCache::new(NonZeroUsize::new(size).unwrap())),
            ttl: Duration::from_secs(seconds),
        }
    }
    fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.lock().unwrap();
        let valid = entries
            .get(key)
            .is_some_and(|(expires, _)| Instant::now() < *expires);
        if valid {
            entries.get(key).map(|(_, value)| value.clone())
        } else {
            entries.pop(key);
            None
        }
    }
    fn put(&self, key: String, value: T) {
        self.put_with_ttl(key, value, self.ttl);
    }
    fn put_with_ttl(&self, key: String, value: T, ttl: Duration) {
        self.entries
            .lock()
            .unwrap()
            .put(key, (Instant::now() + ttl, value));
    }
}
pub struct Qq {
    base: String,
    amll: String,
    api: reqwest::Client,
    media: reqwest::Client,
    cover: reqwest::Client,
    permits: tokio::sync::Semaphore,
    probe_permits: tokio::sync::Semaphore,
    searches: Cache<SearchResult>,
    songs: Cache<Song>,
    infos: Cache<Value>,
    audio: Cache<Audio>,
    audio_failures: Cache<()>,
    lyrics: Cache<Lyrics>,
}
impl Qq {
    pub fn new(base: &str, amll: &str, proxy: Option<&str>) -> Result<Self, String> {
        let proxy = outbound_proxy(proxy)?;
        let mut api = reqwest::Client::builder()
            .http1_only()
            .user_agent("am-hook")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15));
        let mut media = reqwest::Client::builder()
            .http1_only()
            .user_agent("am-hook")
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::custom(|a| {
                if a.previous().len() < 5 && cdn_url(a.url().as_str()).is_some() {
                    a.follow()
                } else {
                    a.stop()
                }
            }));
        // Artwork accepts exactly cover_url's template, including across redirects.
        let mut cover = reqwest::Client::builder()
            .http1_only()
            .user_agent("am-hook")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none());
        if let Some(proxy) = proxy {
            api = api.proxy(proxy.clone());
            media = media.proxy(proxy.clone());
            cover = cover.proxy(proxy);
        }
        Ok(Self {
            base: base.trim_end_matches('/').into(),
            amll: amll.trim_end_matches('/').into(),
            api: api
                .build()
                .map_err(|e| format!("QQ API client: {}", e.without_url()))?,
            media: media
                .build()
                .map_err(|e| format!("QQ CDN client: {}", e.without_url()))?,
            cover: cover
                .build()
                .map_err(|e| format!("QQ cover client: {}", e.without_url()))?,
            permits: tokio::sync::Semaphore::new(8),
            probe_permits: tokio::sync::Semaphore::new(3),
            searches: Cache::new(128, 600),
            songs: Cache::new(4096, 86400),
            infos: Cache::new(4096, 86400),
            audio: Cache::new(4096, 300),
            audio_failures: Cache::new(4096, 120),
            lyrics: Cache::new(1024, 604800),
        })
    }
    async fn request_with_status(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<Value, RequestError> {
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| RequestError::Network)?;
        let response = self
            .api
            .get(format!("{}{path}", self.base))
            .query(params)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(error = %error.without_url(), "qq network request failed");
                RequestError::Network
            })?;
        if !response.status().is_success() {
            let status = response.status();
            tracing::debug!(path, %status, "qq upstream returned non-success status");
            return Err(RequestError::Upstream(status));
        }
        response.json().await.map_err(|error| {
            tracing::debug!(error = %error.without_url(), "qq upstream response decode failed");
            RequestError::Decode
        })
    }

    async fn request(&self, path: &str, params: &[(&str, String)]) -> ApiResult<Value> {
        self.request_with_status(path, params)
            .await
            .map_err(RequestError::api)
    }
    fn song_dto(&self, raw: &Value) -> Option<Song> {
        let id = string(
            raw.get("mid")
                .or_else(|| raw.get("songmid"))
                .or_else(|| raw.get("id"))?,
        );
        if !valid_mid(&id) {
            return None;
        }
        let album = raw
            .get("al")
            .or_else(|| raw.get("album"))
            .unwrap_or(&Value::Null);
        let album_mid = ["mid", "pmid"]
            .iter()
            .filter_map(|k| album.get(k))
            .map(string)
            .find(|s| valid_mid(s))
            .or_else(|| {
                let pic = album.get("picUrl")?.as_str()?;
                cdn_url(pic)?;
                static RE: OnceLock<Regex> = OnceLock::new();
                RE.get_or_init(|| {
                    Regex::new(r"T002(?:R\d+x\d+)?M000([0-9A-Za-z]{1,32})(?:_\d+)?\.").unwrap()
                })
                .captures(pic)
                .map(|c| c[1].to_owned())
            })
            .or_else(|| album.get("id").map(string).filter(|s| valid_mid(s)))
            .unwrap_or_default();
        let cover = if album_mid.is_empty() {
            String::new()
        } else {
            album
                .get("picUrl")
                .and_then(Value::as_str)
                .filter(|url| cdn_url(url).is_some())
                .map(|url| {
                    static RE: OnceLock<Regex> = OnceLock::new();
                    RE.get_or_init(|| Regex::new(r"T002(?:R\d+x\d+)?M000").unwrap())
                        .replace(url, "T002R{w}x{h}M000")
                        .into_owned()
                })
                .unwrap_or_else(|| {
                    format!(
                        "https://y.gtimg.cn/music/photo_new/T002R{{w}}x{{h}}M000{album_mid}.jpg"
                    )
                })
        };
        let artists = raw
            .get("ar")
            .or_else(|| raw.get("singer"))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|a| a.get("name").and_then(Value::as_str))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let duration_ms = raw
            .get("dt")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| raw.get("interval").and_then(Value::as_u64).unwrap_or(0) * 1000);
        let song = Song {
            id: id.clone(),
            name: raw
                .get("name")
                .or_else(|| raw.get("title"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            artists,
            album: album
                .get("name")
                .or_else(|| album.get("title"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            album_mid: album_mid.clone(),
            duration_ms,
            cover,
            vip: raw.get("fee").and_then(Value::as_u64) == Some(1)
                || raw.pointer("/pay/pay_play").and_then(Value::as_u64) == Some(1),
            ttml: raw.get("ttml").is_some_and(|v| v == 1 || v == true),
        };
        self.songs.put(id, song.clone());
        Some(song)
    }
    async fn song(&self, mid: &str) -> ApiResult<Song> {
        if let Some(song) = self.songs.get(mid) {
            return Ok(song);
        }
        self.info_data(mid).await?;
        self.songs.get(mid).ok_or_else(missing)
    }
    async fn info_data(&self, mid: &str) -> ApiResult<Value> {
        if let Some(data) = self.infos.get(mid) {
            if self.songs.get(mid).is_none() {
                self.song_from_info(mid, &data)?;
            }
            return Ok(data);
        }
        let data = self
            .request(
                "/extra/music/info",
                &[("platform", "qq".into()), ("mids", mid.into())],
            )
            .await?;
        // Keep only the normalized track_info object. The outer response may
        // contain account, server and other upstream fields that no parser
        // needs after the DTO has been built.
        let track = track_info(mid, &data).clone();
        self.song_from_info(mid, &track)?;
        self.infos.put(mid.into(), track.clone());
        Ok(track)
    }
    fn song_from_info(&self, mid: &str, data: &Value) -> ApiResult<Song> {
        let song = self.song_dto(track_info(mid, data)).ok_or_else(missing)?;
        if song.id != mid {
            return Err(missing());
        }
        Ok(song)
    }
    async fn audio_url(&self, mid: &str, quality: Quality) -> ApiResult<Audio> {
        let key = quality.cache_key(mid);
        if let Some(audio) = self.audio.get(&key) {
            return Ok(audio);
        }
        let data = self
            .request(
                "/extra/music/url",
                &[
                    ("platform", "qq".into()),
                    ("mid", mid.into()),
                    ("music_quality", quality.name().into()),
                    ("audio_api", "2".into()),
                ],
            )
            .await?;
        let audio = Audio::from_response(&data)?;
        self.audio.put(key, audio.clone());
        Ok(audio)
    }
    async fn probe_audio_url(&self, mid: &str, quality: Quality) -> ApiResult<Audio> {
        let key = quality.cache_key(mid);
        if let Some(audio) = self.audio.get(&key) {
            return Ok(audio);
        }
        if self.audio_failures.get(&key).is_some() {
            return Err(unavailable());
        }
        let _permit = self
            .probe_permits
            .acquire()
            .await
            .map_err(|_| unavailable())?;
        // A concurrent probe may have populated either cache while this call
        // waited for the semaphore.
        if let Some(audio) = self.audio.get(&key) {
            return Ok(audio);
        }
        if self.audio_failures.get(&key).is_some() {
            return Err(unavailable());
        }
        let data = self
            .request_with_status(
                "/extra/music/url",
                &[
                    ("platform", "qq".into()),
                    ("mid", mid.into()),
                    ("music_quality", quality.name().into()),
                    ("audio_api", "2".into()),
                ],
            )
            .await
            .map_err(RequestError::api)?;
        let audio = match Audio::from_response(&data) {
            Ok(audio) if audio.size_bytes.is_some_and(|size| size > 0) => audio,
            Ok(_) | Err(_) => {
                // A successful upstream response with no usable URL/size is a
                // deterministic absence of this tier. Transient request and
                // upstream failures never enter this cache.
                self.audio_failures.put(key, ());
                return Err(unavailable());
            }
        };
        self.audio.put(key, audio.clone());
        Ok(audio)
    }
    async fn qualities(&self, mid: &str, basic: bool) -> ApiResult<Qualities> {
        // One info request supplies all size fields, shared with the song route.
        let data = self.info_data(mid).await?;
        let file = &track_info(mid, &data)["file"];
        let qualities = futures::future::join_all(
            QUALITY_LIST
                .into_iter()
                .filter(|quality| !basic || quality.size_field().is_some())
                .map(|quality| async move {
                    if basic {
                        return QualityRow::from_size(
                            quality,
                            quality.info_size(file),
                            quality.format(),
                        );
                    }
                    if !quality.needs_probe(file) {
                        // Verified info mappings avoid unnecessary URL requests. Cached
                        // probes still describe the actual payload when it differs.
                        match self.audio.get(&quality.cache_key(mid)) {
                            Some(audio) if audio.size_bytes.is_some() => {
                                QualityRow::from_audio(quality, file, Some(&audio))
                            }
                            _ => QualityRow::from_size(
                                quality,
                                quality.info_size(file),
                                quality.format(),
                            ),
                        }
                    } else {
                        // Unmapped tiers are probed separately, through the same short-lived
                        // (mid, quality) cache used by playback and download.
                        let audio = self.probe_audio_url(mid, quality).await.ok();
                        QualityRow::from_audio(quality, file, audio.as_ref())
                    }
                }),
        )
        .await;
        Ok(Qualities { qualities })
    }
}
fn track_info<'a>(mid: &str, data: &'a Value) -> &'a Value {
    // Both the normalized search dialect and QQ's native track_info dialect.
    let rows = data
        .as_array()
        .or_else(|| data.get("data").and_then(Value::as_array));
    let raw = rows
        .and_then(|a| a.first())
        .or_else(|| data.as_object().and_then(|map| map.get(mid)))
        .unwrap_or_else(|| data.get("data").unwrap_or(data));
    raw.get("track_info").unwrap_or(raw)
}
macro_rules! define_qualities {
    ($( $variant:ident => $name:literal ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
        enum Quality {
            $(#[serde(rename = $name)] $variant,)+
        }

        // The supplied 19+ tier table contains twenty distinct, verified API codes.
        const QUALITY_LIST: [Quality; 20] = [$(Quality::$variant),+];

        impl Quality {
            fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }
    };
}

// Keep the API spelling, serde spelling and request spelling in one declaration.
// This preserves the old `rename_all = "UPPERCASE"` JSON exactly.
define_qualities! {
    Mp3128 => "MP3128",
    Aac48 => "AAC48",
    Web => "WEB",
    Aac192 => "AAC192",
    Ogg96 => "OGG96",
    Ogg192 => "OGG192",
    Hq => "HQ",
    Nac => "NAC",
    Sq => "SQ",
    Rs => "RS",
    Qai => "QAI",
    Q360v1 => "Q360V1",
    Q360v2 => "Q360V2",
    Ra360 => "RA360",
    Dts => "DTS",
    Dtsx => "DTSX",
    Da => "DA",
    Daeac3 => "DAEAC3",
    Da448 => "DA448",
    Da1500 => "DA1500",
}
impl Quality {
    fn cache_key(self, mid: &str) -> String {
        format!("{mid}:{}", self.name())
    }
    fn size_field(self) -> Option<&'static str> {
        // Verified against /extra/music/url, not inferred from display names.
        // size_new slot indices and the DTSX field remain deliberately unmapped.
        match self {
            Self::Mp3128 => Some("size_128mp3"),
            Self::Aac48 => Some("size_48aac"),
            Self::Web => Some("size_96aac"),
            Self::Aac192 => Some("size_192aac"),
            Self::Ogg96 => Some("size_96ogg"),
            Self::Ogg192 => Some("size_192ogg"),
            Self::Hq => Some("size_320mp3"),
            Self::Sq => Some("size_flac"),
            Self::Rs => Some("size_hires"),
            Self::Ra360 => Some("size_360ra"),
            Self::Dts => Some("size_dts"),
            Self::Da | Self::Daeac3 => Some("size_dolby"),
            _ => None,
        }
    }
    fn info_size(self, file: &Value) -> Option<u64> {
        let value = file.get(self.size_field()?)?;
        if self == Self::Ra360 {
            // The array can describe multiple encodings. Only an unambiguous
            // positive size can identify a row without a separate URL probe.
            let mut sizes = value.as_array()?.iter().filter_map(byte_size);
            let size = sizes.next()?;
            sizes.all(|other| other == size).then_some(size)
        } else {
            byte_size(value)
        }
    }
    fn needs_probe(self, file: &Value) -> bool {
        self.size_field().is_none()
            || (self == Self::Ra360
                && self.info_size(file).is_none()
                && file["size_360ra"]
                    .as_array()
                    .is_some_and(|sizes| sizes.iter().any(|size| byte_size(size).is_some())))
    }
    fn default_codec(self) -> Codec {
        match self {
            Self::Mp3128 | Self::Hq => Codec::Mp3,
            Self::Aac48 | Self::Web | Self::Aac192 => Codec::Aac,
            Self::Ogg96 | Self::Ogg192 => Codec::Ogg,
            Self::Sq | Self::Rs | Self::Qai | Self::Q360v1 | Self::Q360v2 => Codec::Flac,
            Self::Nac => Codec::Nac,
            Self::Dts => Codec::Dts,
            Self::Dtsx => Codec::Dtsx,
            // WET (001ATFiY3pb3y6) stsd verification, 2026-10-31:
            // DA/DA448/DA1500 = ac-4/dac4; DAEAC3 = ec-3/dec3.
            // QQ's fixed Dolby pipelines determine the codec by tier.
            // AC-4 is reported as a native MIME candidate; only DAEAC3 uses
            // the E-AC-3 JOC path in the frontend.
            Self::Da | Self::Da448 | Self::Da1500 => Codec::Ac4,
            Self::Daeac3 => Codec::Ec3,
            Self::Ra360 => Codec::Unknown,
        }
    }
    fn is_dolby(self) -> bool {
        matches!(self, Self::Da | Self::Daeac3 | Self::Da448 | Self::Da1500)
    }
    fn playback(self, codec: Codec) -> Playback {
        if matches!(self, Self::Nac | Self::Dts | Self::Dtsx | Self::Ra360) {
            Playback::Download
        } else {
            codec.playback()
        }
    }
    fn playback_class(self) -> &'static str {
        // AC-4 is selected by browser capability probing. Keep that runtime
        // decision out of this static contract while retaining its codec/MIME.
        if matches!(self, Self::Da | Self::Da448 | Self::Da1500) {
            "ac-4-probe"
        } else {
            match self.playback(self.default_codec()) {
                Playback::Native => "native",
                Playback::Joc => "joc",
                Playback::Download => "download",
            }
        }
    }
    fn format(self) -> &'static str {
        if let Some(field) = self.size_field() {
            if let Some((_, format)) = FILE_FORMATS.iter().find(|(name, _)| *name == field) {
                return format;
            }
        }
        match self {
            Self::Ra360 => "360 Reality Audio",
            Self::Da => "AC-4 Dolby Atmos",
            Self::Da448 => "AC-4 A-JOC 448 kbps",
            Self::Da1500 => "AC-4 A-JOC 1500 kbps",
            _ => self.default_codec().format(),
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QualityCapability {
    quality: Quality,
    default_codec: &'static str,
    content_type: &'static str,
    playback_class: &'static str,
    size_field: Option<&'static str>,
    display_name: &'static str,
}
impl QualityCapability {
    fn from_quality(quality: Quality) -> Self {
        let codec = quality.default_codec();
        Self {
            quality,
            default_codec: codec.name(),
            content_type: codec.content_type(),
            playback_class: quality.playback_class(),
            size_field: quality.size_field(),
            display_name: quality.format(),
        }
    }
}
fn byte_size(value: &Value) -> Option<u64> {
    value.as_u64().filter(|size| *size > 0)
}
// These are file-field semantics, not fixed quality-to-field mappings. The
// positive /url file_size must match before using a field's codec description.
// size_dolby is shared by AC-4 and E-AC-3 and cannot identify a codec.
const FILE_FORMATS: &[(&str, &str)] = &[
    ("size_128mp3", "MP3 128 kbps"),
    ("size_320mp3", "MP3 320 kbps"),
    ("size_192aac", "AAC 192 kbps"),
    ("size_96aac", "AAC 96 kbps"),
    ("size_48aac", "AAC 48 kbps"),
    ("size_24aac", "AAC 24 kbps"),
    ("size_192ogg", "Ogg 192 kbps"),
    ("size_96ogg", "Ogg 96 kbps"),
    ("size_flac", "FLAC"),
    ("size_ape", "APE"),
    ("size_hires", "FLAC Hi-Res"),
    ("size_dts", "DTS"),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Playback {
    Native,
    Joc,
    Download,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Codec {
    Mp3,
    Aac,
    Flac,
    Ogg,
    Opus,
    Wav,
    Ape,
    Ec3,
    Ac3,
    Ac4,
    Dts,
    Dtsx,
    Nac,
    Unknown,
}
impl Codec {
    fn from_name(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "aac" | "mp4a.40.2" => Self::Aac,
            "flac" => Self::Flac,
            "ogg" | "vorbis" => Self::Ogg,
            "opus" => Self::Opus,
            "wav" | "pcm" => Self::Wav,
            "ape" => Self::Ape,
            "ec-3" | "ec3" | "eac3" | "e-ac-3" | "eac3-joc" | "joc" => Self::Ec3,
            "ac-3" | "ac3" => Self::Ac3,
            "ac-4" | "ac4" => Self::Ac4,
            "dts" | "dtsc" => Self::Dts,
            "dtsx" | "dts:x" | "dtsx.0" => Self::Dtsx,
            "nac" | "aicodec" => Self::Nac,
            _ => return None,
        })
    }
    fn name(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Flac => "flac",
            Self::Ogg => "vorbis",
            Self::Opus => "opus",
            Self::Wav => "pcm",
            Self::Ape => "ape",
            Self::Ec3 => "ec-3",
            Self::Ac3 => "ac-3",
            Self::Ac4 => "ac-4",
            Self::Dts => "dts",
            Self::Dtsx => "dtsx",
            Self::Nac => "nac",
            Self::Unknown => "unknown",
        }
    }
    fn format(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::Aac => "AAC",
            Self::Flac => "FLAC",
            Self::Ogg => "Ogg",
            Self::Opus => "Opus",
            Self::Wav => "PCM WAV",
            Self::Ape => "APE",
            Self::Ec3 => "E-AC-3 JOC",
            Self::Ac3 => "AC-3",
            Self::Ac4 => "AC-4",
            Self::Dts => "DTS",
            Self::Dtsx => "DTS:X",
            Self::Nac => "AICodec",
            Self::Unknown => "Unknown",
        }
    }
    fn playback(self) -> Playback {
        match self {
            Self::Mp3 | Self::Aac | Self::Flac | Self::Ogg | Self::Opus | Self::Wav | Self::Ac4 => {
                Playback::Native
            }
            Self::Ec3 => Playback::Joc,
            _ => Playback::Download,
        }
    }
    fn content_type(self) -> &'static str {
        extension_info_for_codec(self)
            .map(|info| info.content_type)
            .unwrap_or("application/octet-stream")
    }
}

#[derive(Clone, Copy)]
struct ExtensionInfo {
    extension: &'static str,
    codec: Codec,
    content_type: &'static str,
    download_content_type: &'static str,
}

// The playback MIME and the attachment MIME intentionally remain separate:
// browsers need codec parameters while downloaded files should retain their
// conventional raw-media type (for example .aac and .ac4).
const EXTENSION_INFO: &[ExtensionInfo] = &[
    ExtensionInfo {
        extension: "mp3",
        codec: Codec::Mp3,
        content_type: "audio/mpeg",
        download_content_type: "audio/mpeg",
    },
    ExtensionInfo {
        extension: "flac",
        codec: Codec::Flac,
        content_type: "audio/flac",
        download_content_type: "audio/flac",
    },
    ExtensionInfo {
        extension: "m4a",
        codec: Codec::Aac,
        content_type: "audio/mp4; codecs=\"mp4a.40.2\"",
        download_content_type: "audio/mp4",
    },
    ExtensionInfo {
        extension: "mp4",
        codec: Codec::Unknown,
        content_type: "application/octet-stream",
        download_content_type: "audio/mp4",
    },
    ExtensionInfo {
        extension: "aac",
        codec: Codec::Aac,
        content_type: "audio/aac",
        download_content_type: "audio/aac",
    },
    ExtensionInfo {
        extension: "ogg",
        codec: Codec::Ogg,
        content_type: "audio/ogg; codecs=\"vorbis\"",
        download_content_type: "audio/ogg",
    },
    ExtensionInfo {
        extension: "opus",
        codec: Codec::Opus,
        content_type: "audio/ogg; codecs=\"opus\"",
        download_content_type: "audio/ogg",
    },
    ExtensionInfo {
        extension: "wav",
        codec: Codec::Wav,
        content_type: "audio/wav",
        download_content_type: "audio/wav",
    },
    ExtensionInfo {
        extension: "ape",
        codec: Codec::Ape,
        content_type: "audio/x-ape",
        download_content_type: "audio/x-ape",
    },
    ExtensionInfo {
        extension: "eac3",
        codec: Codec::Ec3,
        content_type: "audio/mp4; codecs=\"ec-3\"",
        download_content_type: "audio/eac3",
    },
    ExtensionInfo {
        extension: "ec3",
        codec: Codec::Ec3,
        content_type: "audio/mp4; codecs=\"ec-3\"",
        download_content_type: "audio/eac3",
    },
    ExtensionInfo {
        extension: "joc",
        codec: Codec::Ec3,
        content_type: "audio/mp4; codecs=\"ec-3\"",
        download_content_type: "audio/eac3",
    },
    ExtensionInfo {
        extension: "ac3",
        codec: Codec::Ac3,
        content_type: "audio/mp4; codecs=\"ac-3\"",
        download_content_type: "audio/ac3",
    },
    ExtensionInfo {
        extension: "ac4",
        codec: Codec::Ac4,
        content_type: "audio/mp4; codecs=\"ac-4\"",
        download_content_type: "audio/ac4",
    },
    ExtensionInfo {
        extension: "dts",
        codec: Codec::Dts,
        content_type: "audio/vnd.dts",
        download_content_type: "audio/vnd.dts",
    },
    ExtensionInfo {
        extension: "dtsx",
        codec: Codec::Dtsx,
        content_type: "audio/vnd.dts.hd; codecs=\"dtsx\"",
        download_content_type: "audio/vnd.dts.hd",
    },
    ExtensionInfo {
        extension: "nac",
        codec: Codec::Nac,
        content_type: "audio/x-nac",
        download_content_type: "application/octet-stream",
    },
];

fn extension_info(extension: &str) -> Option<ExtensionInfo> {
    EXTENSION_INFO
        .iter()
        .copied()
        .find(|info| info.extension == extension)
}

fn extension_info_for_codec(codec: Codec) -> Option<ExtensionInfo> {
    EXTENSION_INFO
        .iter()
        .copied()
        .find(|info| info.codec == codec)
}
#[derive(Clone)]
struct Audio {
    url: reqwest::Url,
    size_bytes: Option<u64>,
    codec_hint: Option<Codec>,
}
impl Audio {
    fn from_response(data: &Value) -> ApiResult<Self> {
        let track = data.pointer("/0/track_info").ok_or_else(unavailable)?;
        Ok(Self {
            url: track["file_url"]
                .as_str()
                .and_then(cdn_url)
                .ok_or_else(unavailable)?,
            size_bytes: byte_size(&track["file_size"]),
            // Only allowlisted codec metadata enters DTOs; never pass upstream text through.
            codec_hint: ["codec", "codecs", "audio_codec"]
                .iter()
                .find_map(|key| track[*key].as_str().and_then(Codec::from_name)),
        })
    }
    fn matched_field(&self, file: &Value) -> Option<(&'static str, &'static str)> {
        let size = self.size_bytes?;
        let mut matching = FILE_FORMATS
            .iter()
            .filter(|(field, _)| byte_size(&file[*field]) == Some(size));
        let &(field, format) = matching.next()?;
        matching.next().is_none().then_some((field, format))
    }
    fn codec(&self, quality: Quality) -> Codec {
        if quality.is_dolby() {
            return quality.default_codec();
        }
        if let Some(codec) = self.codec_hint {
            return codec;
        }
        let extension = self
            .url
            .path()
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        extension_info(&extension)
            .map(|info| info.codec)
            .or_else(|| Codec::from_name(&extension))
            .unwrap_or(Codec::Unknown)
    }
    fn content_type(&self, quality: Quality) -> &'static str {
        let codec = self.codec(quality);
        let extension = self
            .url
            .path()
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        extension_info(&extension)
            .filter(|info| info.codec == codec)
            .map(|info| info.content_type)
            .unwrap_or_else(|| codec.content_type())
    }
    fn format(&self, quality: Quality, file: &Value) -> &'static str {
        if quality.is_dolby() {
            // A-JOC bitrates are nominal tier labels, not inferred from size.
            return quality.format();
        }
        let codec = self.codec(quality);
        if let Some((field, format)) = self.matched_field(file) {
            // Size equality cannot turn an AC-4/FLAC URL into a different codec.
            let compatible = match codec {
                Codec::Mp3 => field.ends_with("mp3"),
                Codec::Aac => field.ends_with("aac"),
                Codec::Ogg => field.ends_with("ogg"),
                Codec::Flac => matches!(field, "size_flac" | "size_hires"),
                Codec::Ape => field == "size_ape",
                Codec::Dts => field == "size_dts",
                _ => false,
            };
            if compatible {
                return format;
            }
        }
        // size_new has no verified per-slot codec semantics: even when its size
        // matches (NAC/Q360/QAI), describe the actual extension/codec only.
        codec.format()
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QualityRow {
    quality: Quality,
    available: bool,
    size_bytes: Option<u64>,
    format: &'static str,
    playback: Playback,
    codec: &'static str,
    content_type: &'static str,
}
impl QualityRow {
    fn from_size(quality: Quality, size_bytes: Option<u64>, format: &'static str) -> Self {
        let size_bytes = size_bytes.filter(|size| *size > 0);
        let codec = quality.default_codec();
        Self {
            quality,
            available: size_bytes.is_some(),
            size_bytes,
            format,
            playback: quality.playback(codec),
            codec: codec.name(),
            content_type: codec.content_type(),
        }
    }
    fn from_audio(quality: Quality, file: &Value, audio: Option<&Audio>) -> Self {
        let audio = audio.filter(|audio| audio.size_bytes.is_some());
        let mut row = Self::from_size(
            quality,
            audio.and_then(|a| a.size_bytes),
            audio.map_or(quality.format(), |a| a.format(quality, file)),
        );
        if let Some(audio) = audio {
            let codec = audio.codec(quality);
            row.playback = quality.playback(codec);
            row.codec = codec.name();
            row.content_type = audio.content_type(quality);
        }
        row
    }
}
#[derive(Serialize)]
struct Qualities {
    qualities: Vec<QualityRow>,
}
#[derive(Default, Deserialize)]
struct QualityQuery {
    quality: Option<Quality>,
    basic: Option<u8>,
    probe: Option<u8>,
}
impl QualityQuery {
    fn quality(&self) -> Quality {
        self.quality.unwrap_or(Quality::Hq)
    }
    fn basic(&self) -> bool {
        self.basic == Some(1) || self.probe == Some(0)
    }
}
fn string(v: &Value) -> String {
    v.as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| v.as_u64().map(|n| n.to_string()).unwrap_or_default())
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Song {
    id: String,
    name: String,
    artists: Vec<String>,
    album: String,
    album_mid: String,
    duration_ms: u64,
    cover: String,
    vip: bool,
    ttml: bool,
}
#[derive(Clone, Serialize)]
struct SearchResult {
    songs: Vec<Song>,
    total: u64,
}
#[derive(Clone, Serialize)]
struct Lyric {
    source: &'static str,
    text: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Lyrics {
    candidates: Vec<Lyric>,
    raw_qrc: bool,
    raw_trans: bool,
    raw_roma: bool,
    #[serde(skip_serializing)]
    raw: RawLyrics,
}
#[derive(Clone, Default)]
struct RawLyrics {
    qrc: Option<String>,
    trans: Option<String>,
    roma: Option<String>,
}
#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<usize>,
    offset: Option<usize>,
}
fn adapter(state: &AppState) -> ApiResult<&Qq> {
    state.qq.as_ref().ok_or_else(missing)
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/qq/search", get(search))
        .route("/qq/song/:mid/download", get(download))
        .route("/qq/song/:mid/qualities", get(qualities))
        .route("/qq/cover/:albumMid", get(cover))
        .route("/qq/song/:mid", get(info))
        .route("/qq/song/:mid/play", get(play))
        .route("/qq/song/:mid/lyrics", get(lyrics))
        .route("/qq/song/:mid/lyrics/raw", get(raw_lyrics))
        .route("/qq/capabilities", get(capabilities))
}
async fn cover(
    State(state): State<Arc<AppState>>,
    Path(album_mid): Path<String>,
) -> ApiResult<Response> {
    let qq = adapter(&state)?;
    let url = cover_url(&album_mid)?;
    cover_response(qq.cover.get(url).send().await.map_err(network)?)
}
fn cover_response(response: reqwest::Response) -> ApiResult<Response> {
    if response.status() == StatusCode::NOT_FOUND {
        return Err(missing());
    }
    if !response.status().is_success() {
        return Err(unavailable());
    }
    use futures::TryStreamExt;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from_stream(
            response
                .bytes_stream()
                .map_err(|e| std::io::Error::other(e.without_url())),
        ))
        .unwrap())
}
async fn capabilities(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    adapter(&state)?;
    let qualities = QUALITY_LIST
        .into_iter()
        .map(QualityCapability::from_quality)
        .collect::<Vec<_>>();
    Ok(Json(json!({"enabled": true, "qualities": qualities})))
}
async fn search(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<SearchResult>> {
    let qq = adapter(&state)?;
    let term = q.q.trim();
    if term.is_empty() || term.len() > 512 {
        return Err(bad());
    }
    let limit = q.limit.unwrap_or(30).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).min(10000);
    let key = format!("{limit}:{offset}:{term}");
    if let Some(data) = qq.searches.get(&key) {
        return Ok(Json(data));
    }
    let raw = qq
        .request(
            "/cloudsearch",
            &[
                ("platform", "qq".into()),
                ("keywords", term.into()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
                ("type", "1".into()),
            ],
        )
        .await?;
    let result = raw.get("result").ok_or_else(unavailable)?;
    let songs = result
        .get("songs")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|s| qq.song_dto(s)).collect())
        .unwrap_or_default();
    let data = SearchResult {
        songs,
        total: result.get("songCount").and_then(Value::as_u64).unwrap_or(0),
    };
    qq.searches.put(key, data.clone());
    Ok(Json(data))
}
async fn info(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
) -> ApiResult<Json<Song>> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    Ok(Json(qq.song(&mid).await?))
}
async fn qualities(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
    Query(query): Query<QualityQuery>,
) -> ApiResult<Json<Qualities>> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    Ok(Json(qq.qualities(&mid, query.basic()).await?))
}
async fn play(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
    Query(query): Query<QualityQuery>,
) -> ApiResult<Response> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    // Only this response can expose file_url; JSON DTOs have no audio URL field.
    Ok(play_redirect(
        &qq.audio_url(&mid, query.quality()).await?.url,
    ))
}
fn play_redirect(url: &reqwest::Url) -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, url.as_str())
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::empty())
        .unwrap()
}
fn download_disposition(song: &Song, url: &reqwest::Url) -> String {
    let ext = url
        .path()
        .rsplit('.')
        .next()
        .filter(|s| !s.is_empty() && s.len() <= 5 && s.bytes().all(|b| b.is_ascii_alphanumeric()))
        .unwrap_or("m4a");
    attachment(&safe_basename(song), ext)
}
fn download_content_type(url: &reqwest::Url) -> &'static str {
    let extension = url
        .path()
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    extension_info(&extension)
        .map(|info| info.download_content_type)
        .unwrap_or("application/octet-stream")
}
#[derive(Default, Deserialize)]
struct RawLyricsQuery {
    kind: Option<String>,
}
fn raw_lyrics_kind(kind: Option<&str>) -> ApiResult<&'static str> {
    match kind {
        Some("qrc") => Ok("qrc"),
        Some("qrctrans") => Ok("qrctrans"),
        Some("qrcroma") => Ok("qrcroma"),
        _ => Err(bad()),
    }
}
fn safe_basename(song: &Song) -> String {
    let artists = song.artists.join(", ");
    let raw = match (artists.is_empty(), song.name.is_empty()) {
        (false, false) => format!("{artists} - {}", song.name),
        (false, true) => artists,
        (true, false) => song.name.clone(),
        (true, true) => song.id.clone(),
    };
    let base: String = raw
        .chars()
        .filter(|c| !c.is_control() && !"/\\:*?\"<>|".contains(*c))
        .take(160)
        .collect();
    let mut base = base
        .trim_matches(|c: char| c.is_whitespace() || c == '.')
        .to_owned();
    if base.is_empty() {
        base = song.id.clone();
    }
    if matches!(
        base.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        base.push('_');
    }
    base
}
fn attachment(base: &str, ext: &str) -> String {
    let name = format!("{base}.{ext}");
    let encoded: String = name.bytes().map(|b| format!("%{b:02X}")).collect();
    format!("attachment; filename=\"download.{ext}\"; filename*=UTF-8''{encoded}")
}
fn lyrics_filename(song: &Song, kind: &str) -> (String, &'static str) {
    let (suffix, extension) = match kind {
        "qrc" => ("", "qrc"),
        "qrctrans" => (".trans", "lrc"),
        "qrcroma" => (".roma", "qrc"),
        _ => unreachable!("validated raw lyric kind"),
    };
    (format!("{}{suffix}", safe_basename(song)), extension)
}
fn lyrics_disposition(song: &Song, kind: &str) -> String {
    let (base, ext) = lyrics_filename(song, kind);
    attachment(&base, ext)
}
async fn download(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
    Query(query): Query<QualityQuery>,
) -> ApiResult<Response> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    let song = qq.song(&mid).await?;
    let url = qq.audio_url(&mid, query.quality()).await?.url;
    let response = qq.media.get(url.clone()).send().await.map_err(network)?;
    let status = response.status();
    if !status.is_success() {
        tracing::warn!("qq download upstream status: {status}");
        return Err(unavailable());
    }
    let mut out = Response::builder()
        .header(header::CONTENT_TYPE, download_content_type(&url))
        .header(
            header::CONTENT_DISPOSITION,
            download_disposition(&song, &url),
        )
        .header(header::CACHE_CONTROL, "no-store");
    if let Some(size) = response.headers().get(header::CONTENT_LENGTH) {
        out = out.header(header::CONTENT_LENGTH, size);
    }
    // Keep upstream errors (which carry URLs) out of streaming body diagnostics too.
    use futures::TryStreamExt;
    Ok(out
        .body(Body::from_stream(
            response
                .bytes_stream()
                .map_err(|e| std::io::Error::other(e.without_url())),
        ))
        .unwrap())
}
fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn clean_text(text: &str) -> String {
    static RE_URL: OnceLock<Regex> = OnceLock::new();
    RE_URL
        .get_or_init(|| Regex::new(r#"(?i)https?://[^\s<>"']+"#).unwrap())
        .replace_all(text, "")
        .into_owned()
}
fn wrap(body: &str, word: bool) -> String {
    format!("<tt xmlns=\"http://www.w3.org/ns/ttml\" xmlns:ttm=\"http://www.w3.org/ns/ttml#metadata\" xmlns:itunes=\"http://music.apple.com/lyric-ttml-internal\" itunes:timing=\"{}\">{body}</tt>", if word { "Word" } else { "Line" })
}
/// Rebuild the renderable body with a strict tag / attribute allowlist. No credit
/// comments, external entities, URLs, credit metadata, styles or external resources.
fn clean_ttml(text: &str) -> Option<String> {
    static UNSAFE: OnceLock<Regex> = OnceLock::new();
    if text.len() > 2_000_000
        || UNSAFE
            .get_or_init(|| Regex::new(r"(?i)<!DOCTYPE|<!ENTITY").unwrap())
            .is_match(text)
    {
        return None;
    }
    static COMMENTS: OnceLock<Regex> = OnceLock::new();
    let text = COMMENTS
        .get_or_init(|| Regex::new(r"(?s)<!--.*?-->").unwrap())
        .replace_all(text, "");
    static BODY: OnceLock<Regex> = OnceLock::new();
    let body = BODY
        .get_or_init(|| Regex::new(r"(?s)<body\b[^>]*>.*?</body>").unwrap())
        .find(&text)?
        .as_str();
    static TAGS: OnceLock<Regex> = OnceLock::new();
    let tags = TAGS.get_or_init(|| Regex::new(r"(?s)<[^>]*>|[^<]+").unwrap());
    static TAG: OnceLock<Regex> = OnceLock::new();
    let tag = TAG.get_or_init(|| Regex::new(r"^<(/?)(body|div|p|span|br)\b([^>]*)>$").unwrap());
    static ATTRS: OnceLock<Regex> = OnceLock::new();
    let attrs = ATTRS.get_or_init(|| Regex::new(r#"([\w:]+)\s*=\s*["']([^"']*)["']"#).unwrap());
    static TIME: OnceLock<Regex> = OnceLock::new();
    let time = TIME.get_or_init(|| Regex::new(r"^\d+(?::\d+){0,2}(?:\.\d+)?$").unwrap());
    let mut out = String::new();
    for token in tags.find_iter(body) {
        let token = token.as_str();
        if !token.starts_with('<') {
            out.push_str(&clean_text(token));
            continue;
        }
        let Some(c) = tag.captures(token) else {
            continue;
        };
        if &c[1] == "/" {
            out.push_str(&format!("</{}>", &c[2]));
            continue;
        }
        out.push('<');
        out.push_str(&c[2]);
        for a in attrs.captures_iter(&c[3]) {
            let (key, value) = (&a[1], &a[2]);
            let allowed = match key {
                "begin" | "end" | "dur" => time.is_match(value),
                "ttm:role" => matches!(value, "x-bg" | "x-translation" | "x-roman"),
                "ttm:agent" | "itunes:key" | "itunes:songPart" => valid_mid(value),
                _ => false,
            };
            if allowed {
                out.push_str(&format!(" {key}=\"{}\"", xml(value)));
            }
        }
        out.push_str(if token.ends_with("/>") { "/>" } else { ">" });
    }
    if !out.contains("<p ") {
        return None;
    }
    static WORD: OnceLock<Regex> = OnceLock::new();
    let word = WORD
        .get_or_init(|| Regex::new(r#"<span\b[^>]*\bbegin="#).unwrap())
        .is_match(&out);
    let mut result = wrap(&out, word);
    // Agent types affect duet layout; keep those declarations without names/credits.
    static AGENTS: OnceLock<Regex> = OnceLock::new();
    let agents = AGENTS.get_or_init(|| Regex::new(r#"<(?:ttm:)?agent\b[^>]*>"#).unwrap());
    let mut head = String::new();
    for agent in agents.find_iter(&text) {
        let mut id = None;
        let mut kind = None;
        for a in attrs.captures_iter(agent.as_str()) {
            if &a[1] == "xml:id" && valid_mid(&a[2]) {
                id = Some(a[2].to_owned());
            }
            if &a[1] == "type" && matches!(&a[2], "person" | "group" | "other") {
                kind = Some(a[2].to_owned());
            }
        }
        if let Some(id) = id {
            head.push_str(&format!(
                "<ttm:agent xml:id=\"{id}\" type=\"{}\"/>",
                kind.as_deref().unwrap_or("person")
            ));
        }
    }
    if !head.is_empty() {
        result = result.replacen(
            "<body",
            &format!("<head><metadata>{head}</metadata></head><body"),
            1,
        );
    }
    static LANG: OnceLock<Regex> = OnceLock::new();
    if let Some(lang) = LANG
        .get_or_init(|| Regex::new(r#"<tt\b[^>]*\bxml:lang=["']([A-Za-z0-9-]{1,35})["']"#).unwrap())
        .captures(&text)
    {
        result = result.replacen("<tt ", &format!("<tt xml:lang=\"{}\" ", &lang[1]), 1);
    }
    Some(result)
}
fn field_text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    let v = value.get(key)?;
    v.as_str()
        .or_else(|| v.get("lyric").and_then(Value::as_str))
}
struct LyricWord {
    start: u64,
    end: u64,
    text: String,
}
struct LyricLine {
    start: u64,
    end: u64,
    text: String,
    words: Vec<LyricWord>,
    is_bg: bool,
}
fn lrc_lines(text: &str) -> Vec<LyricLine> {
    static STAMP: OnceLock<Regex> = OnceLock::new();
    let stamp = STAMP.get_or_init(|| Regex::new(r"\[(\d+):(\d+(?:\.\d+)?)\]").unwrap());
    static OFFSET: OnceLock<Regex> = OnceLock::new();
    let offset = OFFSET
        .get_or_init(|| Regex::new(r"\[offset:([+-]?\d+)\]").unwrap())
        .captures(text)
        .and_then(|c| c[1].parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut lines = Vec::new();
    for line in text.lines() {
        let content = stamp.replace_all(line, "");
        let content = clean_text(content.trim());
        if content.is_empty() || content.starts_with('[') {
            continue;
        }
        for c in stamp.captures_iter(line) {
            let (Ok(minutes), Ok(seconds)) = (c[1].parse::<f64>(), c[2].parse::<f64>()) else {
                continue;
            };
            let start = ((minutes * 60.0 + seconds) * 1000.0 - offset)
                .round()
                .max(0.0) as u64;
            lines.push(LyricLine {
                start,
                end: start.saturating_add(8000),
                text: content.clone(),
                words: Vec::new(),
                is_bg: false,
            });
        }
    }
    lines.sort_by_key(|line| line.start);
    for i in 1..lines.len() {
        lines[i - 1].end = lines[i].start;
    }
    lines
}
fn qrc_content(text: &str) -> String {
    // QQ leaves quotes inside LyricContent unescaped. Only a matching quote
    // followed by optional whitespace and /> ends the value, across newlines.
    // Clipped responses may omit that delimiter; retain their available content.
    static ENVELOPE: OnceLock<Regex> = OnceLock::new();
    let envelope = ENVELOPE.get_or_init(|| {
        Regex::new(r#"(?s)\bLyricContent\s*=\s*(?:"(.*?)(?:"\s*/>|$)|'(.*?)(?:'\s*/>|$))"#).unwrap()
    });
    let content = envelope.captures(text);
    content
        .as_ref()
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map_or(text, |m| m.as_str())
        .replace("&#10;", "\n")
        .replace("&#13;", "\r")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
fn qrc_lines(text: &str) -> Vec<LyricLine> {
    // This follows AMLL's parseQrc implementation. QRC word captures are lazy:
    // each match consumes everything since the previous timing tuple, including
    // parentheses, commas, and spaces. That is important for nested translations
    // such as `you((123,4)字(127,4))`.
    let text = qrc_content(text);
    static LINE: OnceLock<Regex> = OnceLock::new();
    let line = LINE.get_or_init(|| Regex::new(r"^\[(\d+),(\d+)\]").unwrap());
    static WORD: OnceLock<Regex> = OnceLock::new();
    let word = WORD.get_or_init(|| Regex::new(r"(.*?)\((\d+),(\d+)\)").unwrap());
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let line_text = raw_line.trim();
        if line_text.is_empty() {
            continue;
        }
        let Some(line_match) = line.captures(line_text) else {
            continue;
        };
        let (Ok(start), Ok(duration)) =
            (line_match[1].parse::<u64>(), line_match[2].parse::<u64>())
        else {
            continue;
        };
        let line_content = line_text[line_match[0].len()..].trim();
        if line_content.is_empty() {
            continue;
        }
        let mut words = Vec::new();
        for w in word.captures_iter(line_content) {
            let (Ok(begin), Ok(duration)) = (w[2].parse::<u64>(), w[3].parse::<u64>()) else {
                continue;
            };
            words.push(LyricWord {
                start: begin,
                end: begin.saturating_add(duration),
                text: clean_text(&w[1]),
            });
        }

        let is_bg = !words.is_empty()
            && (words[0].text.starts_with('(') || words[0].text.starts_with('（'))
            && (words
                .last()
                .is_some_and(|word| word.text.ends_with(')') || word.text.ends_with('）')));
        if is_bg {
            if let Some(first) = words.first_mut() {
                if let Some(text) = first.text.strip_prefix('(') {
                    first.text = text.to_owned();
                } else if let Some(text) = first.text.strip_prefix('（') {
                    first.text = text.to_owned();
                }
            }
            if let Some(last) = words.last_mut() {
                if let Some(text) = last.text.strip_suffix(')') {
                    last.text = text.to_owned();
                } else if let Some(text) = last.text.strip_suffix('）') {
                    last.text = text.to_owned();
                }
            }
        }

        lines.push(LyricLine {
            start,
            end: start.saturating_add(duration),
            text: words.iter().map(|word| word.text.as_str()).collect(),
            words,
            is_bg,
        });
    }
    lines
}
const AUXILIARY_MAX_DISTANCE: u64 = 1000;
// A skipped auxiliary row costs 700 ms.  This makes every allowed pair at
// most 700 ms preferable to skipping it, while still allowing the DP to skip
// a badly timed row when doing so enables better later matches.
const AUXILIARY_SKIP_PENALTY: u64 = 700;
// A nearby ordinary row owns a competing candidate before a BG row.  This is
// only a BG reservation rule; ordinary rows still accept the full 1000ms DP
// matching limit.
const AUXILIARY_BG_PRIORITY_WINDOW: u64 = 500;

struct AuxiliaryLine {
    start: u64,
    text: String,
}

fn auxiliary_lines(lines: Vec<LyricLine>) -> Vec<AuxiliaryLine> {
    lines
        .into_iter()
        .filter_map(|line| {
            let text = if line.words.is_empty() {
                line.text
            } else {
                line.words.iter().map(|word| word.text.as_str()).collect()
            };
            (!matches!(text.trim(), "" | "//")).then_some(AuxiliaryLine {
                start: line.start,
                text,
            })
        })
        .collect()
}

fn align_auxiliary(lines: &[LyricLine], auxiliary: Vec<LyricLine>) -> Vec<Option<String>> {
    let mut auxiliary = auxiliary_lines(auxiliary);
    if lines.is_empty() || auxiliary.is_empty() {
        return vec![None; lines.len()];
    }
    auxiliary.sort_by_key(|line| line.start);

    // `state` is the number of the last matched main row (zero means that no
    // main row has been matched yet).  Main rows stay in their existing order;
    // only auxiliary rows are sorted by timestamp.  A match can therefore
    // only transition from a state <= the target row, which makes crossings
    // impossible while leaving unmatched main rows free of charge.
    #[derive(Clone, Copy)]
    struct DpValue {
        cost: u64,
        last_distance: u64,
        last_main: usize,
    }
    #[derive(Clone, Copy)]
    struct Parent {
        previous: usize,
        matched: bool,
    }
    #[derive(Clone, Copy)]
    struct Candidate {
        value: DpValue,
        parent: Parent,
        action_distance: Option<u64>,
    }

    const INF: u64 = u64::MAX / 4;
    let main_count = lines.len();
    let no_main = usize::MAX;
    let initial = DpValue {
        cost: 0,
        last_distance: INF,
        last_main: no_main,
    };

    // A path with a lower cost wins. For equal costs, compare the current
    // match's distance and main-row order first; otherwise retain the earlier
    // match already present in the path. This makes equal-cost choices stable.
    let better_value = |left: DpValue, right: DpValue| {
        (left.cost, left.last_distance, left.last_main)
            < (right.cost, right.last_distance, right.last_main)
    };
    let better_candidate = |left: Candidate, right: Candidate| {
        if left.value.cost != right.value.cost {
            return left.value.cost < right.value.cost;
        }
        match (left.action_distance, right.action_distance) {
            (Some(left_distance), Some(right_distance)) => {
                (left_distance, left.value.last_main, left.parent.previous)
                    < (right_distance, right.value.last_main, right.parent.previous)
            }
            (Some(left_distance), None) => {
                (left_distance, left.value.last_main)
                    < (right.value.last_distance, right.value.last_main)
            }
            (None, Some(right_distance)) => {
                (left.value.last_distance, left.value.last_main)
                    <= (right_distance, right.value.last_main)
            }
            (None, None) => better_value(left.value, right.value),
        }
    };

    let distance = |auxiliary_line: &AuxiliaryLine, line: &LyricLine| {
        // A timestamp inside a timed main line belongs to that line even when
        // its line-start anchors differ.  Outside the interval, use the
        // anchor difference required by the DP cost model.
        if auxiliary_line.start >= line.start && auxiliary_line.start <= line.end {
            0
        } else {
            auxiliary_line.start.abs_diff(line.start)
        }
    };

    let unreachable = DpValue {
        cost: INF,
        last_distance: INF,
        last_main: no_main,
    };
    let mut previous = vec![unreachable; main_count + 1];
    previous[0] = initial;
    let mut parents = Vec::with_capacity(auxiliary.len());
    for auxiliary_line in &auxiliary {
        // For a BG target, an available ordinary row gets the translation
        // first.  Once every ordinary row after the current path has been
        // consumed, the BG row can receive its own later auxiliary row.
        let mut ordinary_available = vec![false; main_count + 1];
        for index in (0..main_count).rev() {
            ordinary_available[index] = ordinary_available[index + 1]
                || (!lines[index].is_bg
                    && distance(auxiliary_line, &lines[index])
                        <= AUXILIARY_BG_PRIORITY_WINDOW);
        }

        let mut next = vec![DpValue {
            cost: INF,
            last_distance: INF,
            last_main: no_main,
        }; main_count + 1];
        let mut layer_parents = vec![
            Parent {
                previous: 0,
                matched: false,
            };
            main_count + 1
        ];
        let mut prefix_best = vec![0; main_count];
        let mut best = 0;
        for index in 0..main_count {
            if better_value(previous[index], previous[best]) {
                best = index;
            }
            prefix_best[index] = best;
        }
        let mut bg_prefix_best = vec![None; main_count];
        let mut best_bg = None;
        for index in 0..main_count {
            if !ordinary_available[index]
                && previous[index].cost != INF
                && best_bg.is_none_or(|candidate| {
                    better_value(previous[index], previous[candidate])
                })
            {
                best_bg = Some(index);
            }
            bg_prefix_best[index] = best_bg;
        }

        // Skipping the auxiliary row leaves the last matched main row alone.
        for state in 0..=main_count {
            if previous[state].cost == INF {
                continue;
            }
            let candidate = Candidate {
                value: DpValue {
                    cost: previous[state].cost.saturating_add(AUXILIARY_SKIP_PENALTY),
                    ..previous[state]
                },
                parent: Parent {
                    previous: state,
                    matched: false,
                },
                action_distance: None,
            };
            let current = Candidate {
                value: next[state],
                parent: layer_parents[state],
                action_distance: None,
            };
            if next[state].cost == INF || better_candidate(candidate, current) {
                next[state] = candidate.value;
                layer_parents[state] = candidate.parent;
            }
        }

        // Matching row `index` consumes a prefix state whose last main row is
        // before it. The prefix minimum makes this DP O(auxiliary * main).
        // The hard limit is checked against the row anchors even when an
        // auxiliary stamp falls inside a timed main interval.
        for index in 0..main_count {
            if auxiliary_line.start.abs_diff(lines[index].start) > AUXILIARY_MAX_DISTANCE {
                continue;
            }
            let match_distance = distance(auxiliary_line, &lines[index]);
            let source = if lines[index].is_bg {
                let Some(source) = bg_prefix_best[index] else {
                    continue;
                };
                source
            } else {
                prefix_best[index]
            };
            if previous[source].cost == INF {
                continue;
            }
            let candidate = Candidate {
                value: DpValue {
                    cost: previous[source].cost.saturating_add(match_distance),
                    last_distance: match_distance,
                    last_main: index,
                },
                parent: Parent {
                    previous: source,
                    matched: true,
                },
                action_distance: Some(match_distance),
            };
            let state = index + 1;
            let current = Candidate {
                value: next[state],
                parent: layer_parents[state],
                action_distance: if layer_parents[state].matched {
                    Some(next[state].last_distance)
                } else {
                    None
                },
            };
            if next[state].cost == INF || better_candidate(candidate, current) {
                next[state] = candidate.value;
                layer_parents[state] = candidate.parent;
            }
        }

        previous = next;
        parents.push(layer_parents);
    }

    let mut state = 0;
    for candidate in 1..=main_count {
        if better_value(previous[candidate], previous[state]) {
            state = candidate;
        }
    }
    let mut aligned = vec![None; main_count];
    for (auxiliary_index, layer) in parents.iter().enumerate().rev() {
        let parent = layer[state];
        if parent.matched {
            aligned[state - 1] = Some(auxiliary[auxiliary_index].text.clone());
        }
        state = parent.previous;
    }
    aligned
}

// MeT-Music UI's role vocabulary, copied from met-info-filter-reference.md.
// The source bundle lowercases these dot-separated words before matching.
const INFO_EN: &str = "作词.作曲.说唱作曲.原唱.中文填词.说唱填词.填词.改编词.词曲.作词作曲.填詞.中文填詞.词.曲.原.作詞.詞.总制作人.联合制作人.音乐制作.监制.音乐总监.项目总监.营销推广.编曲.編曲.制作.總製作人.聯合製作人.製作.音樂製作.監製.混音.母带.和声.和音.配唱.人声.录音勤务.录音.声乐指导.配乐.音频编辑.剪辑.音乐编辑.母帶.錄音.剪輯.和聲.器乐团队.戏曲.引用.乐队.吉他.贝斯.鼓.键盘.弦乐.管乐.打击乐.演唱.合唱.貝斯.鍵盤.弦樂.出品.特别鸣谢.歌曲营销.策划.策劃.运营.運營.推广.推廣.项目企划.企划.统筹.总监.總監.版权.版權.发行.商务.营销.宣传.發行.企劃.統籌.OP.SP.Lyrics.Lyricist.Lyrics by.Written by.Words by.Music.Music by.Composer.Composed by.Arranger.Arranged by.Arrangement.Producer.Produced by.Executive Producer.Co-Producer.Mixing.Mixed by.Mixing Engineer.Mastering.Mastered by.Mastering Engineer.Recording.Recorded by.Recording Engineer.Vocal Director.Backing Vocals.Vocal.Vocals.Backing Vocal.Crowd Vocals.Studio.Guitar.Bass.Drums.Keyboard.Piano.Strings.Programming.Publisher.Label.Coordinator";
const INFO_TN: &str = "音频.音頻.总.總.大.联合.聯合.共同.执行.執行.项目.項目.艺术.藝術.音乐.音樂.人声.人聲.弦乐.弦樂.乐队.樂隊.和声.和聲.歌曲.版权.版權.宣传.宣傳";
const INFO_NN: &str = "师.師.工程师.工程師.管理.后期.後期.室.棚.助理.统筹.統籌.处理.處理.录音.錄音.编写.編寫.指导.指導.协力.協力.总监.總監.监制.監製.编辑.編輯.策划.策劃.企划.企劃.人.方";

#[derive(Clone, Copy)]
struct LyricContext<'a> {
    name: &'a str,
    artists: &'a [String],
}

fn starts_with_role(text: &str, pos: usize, word: &str) -> bool {
    text.get(pos..)
        .and_then(|rest| rest.get(..word.len()))
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(word))
}

fn match_roles(text: &str, pos: usize, words: &str, out: &mut HashSet<usize>) {
    for word in words.split('.') {
        if starts_with_role(text, pos, word) {
            out.insert(pos + word.len());
        }
    }
}

fn skip_whitespace(text: &str, mut pos: usize) -> usize {
    while let Some(ch) = text.get(pos..).and_then(|rest| rest.chars().next()) {
        if !ch.is_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

/// Match a complete role label before the first colon. This is deliberately a
/// small parser rather than a prefix list: labels such as "音乐总监:",
/// "制作人/编曲:" and "总监制:" are composed from the three vocabularies.
fn is_role_label(text: &str) -> bool {
    let Some(colon) = text
        .char_indices()
        .find_map(|(pos, ch)| matches!(ch, ':' | '：').then_some(pos))
    else {
        return false;
    };
    let label = text[..colon].trim_end().to_lowercase();
    if label.is_empty() {
        return false;
    }
    let end = label.len();
    let mut segment_starts = HashSet::from([0]);
    let mut after_prefix = HashSet::new();
    let mut role_ends = HashSet::new();
    let mut positions: Vec<usize> = label.char_indices().map(|(pos, _)| pos).collect();
    positions.push(end);

    for pos in positions {
        if role_ends.contains(&pos) {
            if pos == end {
                return true;
            }
            match_roles(&label, pos, INFO_NN, &mut role_ends);
            let mut next = skip_whitespace(&label, pos);
            if let Some(ch) = label.get(next..).and_then(|rest| rest.chars().next()) {
                if matches!(ch, '/' | '／' | '&') {
                    next += ch.len_utf8();
                    next = skip_whitespace(&label, next);
                }
            }
            segment_starts.insert(pos);
            segment_starts.insert(next);
        }
        if segment_starts.contains(&pos) {
            match_roles(&label, pos, INFO_TN, &mut after_prefix);
        }
        if segment_starts.contains(&pos) || after_prefix.contains(&pos) {
            match_roles(&label, pos, INFO_EN, &mut role_ends);
        }
    }
    false
}

fn copyright_info(text: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"未[经經].*[许許]可.*(?:不得|禁止)").unwrap())
        .is_match(text)
}

fn is_prefix_info(text: &str) -> bool {
    let text = text.trim_start();
    ["器乐团队", "说唱作曲", "戏曲", "引用"]
        .iter()
        .any(|prefix| {
            text.strip_prefix(prefix).is_some_and(|rest| {
                rest.trim_start()
                    .chars()
                    .next()
                    .is_some_and(|ch| matches!(ch, ':' | '：'))
            })
        })
}

fn has_artist_header(text: &str, context: Option<LyricContext<'_>>) -> bool {
    let trimmed = text.trim();
    (trimmed.ends_with(':') || trimmed.ends_with('：'))
        && context.is_some_and(|ctx| {
            ctx.artists
                .iter()
                .filter(|artist| !artist.is_empty())
                .any(|artist| text.contains(artist))
        })
}

fn title_header(line: &LyricLine, first_nonempty: bool, context: Option<LyricContext<'_>>) -> bool {
    if !first_nonempty {
        return false;
    }
    let has_context = context.is_some_and(|ctx| {
        (!ctx.name.is_empty() && line.text.contains(ctx.name))
            || ctx
                .artists
                .iter()
                .filter(|artist| !artist.is_empty())
                .any(|artist| line.text.contains(artist))
    });
    (line.text.contains('-') && has_context) || (line.start == 0 && line.text.contains(" - "))
}

fn is_info_line(line: &LyricLine, first_nonempty: bool, context: Option<LyricContext<'_>>) -> bool {
    let text = &line.text;
    let lower = text.to_ascii_lowercase();
    is_role_label(text)
        || copyright_info(text)
        || (text.contains("计划")
            && ["酷狗", "音乐人", "QQ音乐"]
                .iter()
                .any(|term| text.contains(term)))
        || lower.contains("by:")
        || lower.contains("by：")
        || has_artist_header(text, context)
        || is_prefix_info(text)
        || title_header(line, first_nonempty, context)
        || (line.start == 0
            && line.end == 0
            && line.words.iter().all(|word| word.start == word.end))
}

fn pure_music_line(text: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"纯音乐[，,]?\s*请(您)?欣赏").unwrap())
        .is_match(text)
}

fn filter_info_lines(lines: Vec<LyricLine>, context: Option<LyricContext<'_>>) -> Vec<LyricLine> {
    let mut first_seen = false;
    lines
        .into_iter()
        .filter_map(|line| {
            let nonempty = !line.text.trim().is_empty();
            let first = nonempty && !first_seen;
            if nonempty {
                first_seen = true;
            }
            (!is_info_line(&line, first, context)).then_some(line)
        })
        .collect()
}

fn all_pure_music(lines: &[LyricLine]) -> bool {
    !lines.is_empty() && lines.iter().all(|line| pure_music_line(&line.text))
}

fn render_words(line: &LyricLine, out: &mut String) {
    if line.words.is_empty() {
        out.push_str(&xml(&line.text));
    } else {
        for word in &line.words {
            out.push_str(&format!(
                "<span begin=\"{:.3}\" end=\"{:.3}\">{}</span>",
                word.start as f64 / 1000.0,
                word.end as f64 / 1000.0,
                xml(&word.text)
            ));
        }
    }
}

fn render_role(out: &mut String, role: &str, text: Option<&str>) {
    if let Some(text) = text {
        out.push_str(&format!("<span ttm:role=\"{role}\">{}</span>", xml(text)));
    }
}

fn bg_parent(lines: &[LyricLine], bg_index: usize) -> Option<usize> {
    let bg = &lines[bg_index];
    let mut overlap: Option<usize> = None;
    for (index, line) in lines.iter().enumerate() {
        if line.is_bg || index == bg_index || line.start > bg.start || line.end <= bg.start {
            continue;
        }
        // Main lyric intervals normally do not overlap. If they do, the one
        // with the latest begin is the line whose active interval contains the
        // background begin, matching the nearest temporal reading.
        if overlap.is_none_or(|previous| line.start > lines[previous].start) {
            overlap = Some(index);
        }
    }
    if overlap.is_some() {
        return overlap;
    }
    // A background row with no containing interval follows the most recent
    // main row. Rows before the first main lyric stay standalone so content is
    // never discarded.
    lines
        .iter()
        .enumerate()
        .filter(|(index, line)| !line.is_bg && *index != bg_index && line.start <= bg.start)
        .max_by_key(|(_, line)| line.start)
        .map(|(index, _)| index)
}

fn render_lyrics(
    lines: Vec<LyricLine>,
    word: bool,
    translations: Vec<LyricLine>,
    pronunciations: Vec<LyricLine>,
) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let mut attached_bg = vec![Vec::new(); lines.len()];
    let mut standalone_bg = vec![false; lines.len()];
    for (index, _line) in lines.iter().enumerate().filter(|(_, line)| line.is_bg) {
        if let Some(parent) = bg_parent(&lines, index) {
            attached_bg[parent].push(index);
        } else {
            standalone_bg[index] = true;
        }
    }
    let translations = align_auxiliary(&lines, translations);
    let pronunciations = align_auxiliary(&lines, pronunciations);
    let mut out = String::from("<body><div>");
    for (index, line) in lines.iter().enumerate() {
        if line.is_bg {
            if !standalone_bg[index] {
                continue;
            }
            out.push_str(&format!(
                "<p begin=\"{:.3}\" end=\"{:.3}\">",
                line.start as f64 / 1000.0,
                line.end as f64 / 1000.0
            ));
            render_words(line, &mut out);
            render_role(&mut out, "x-translation", translations[index].as_deref());
            render_role(&mut out, "x-roman", pronunciations[index].as_deref());
            out.push_str("</p>");
            continue;
        }

        out.push_str(&format!(
            "<p begin=\"{:.3}\" end=\"{:.3}\">",
            line.start as f64 / 1000.0,
            line.end as f64 / 1000.0
        ));
        render_words(line, &mut out);
        render_role(&mut out, "x-translation", translations[index].as_deref());
        render_role(&mut out, "x-roman", pronunciations[index].as_deref());
        for &bg_index in &attached_bg[index] {
            let bg = &lines[bg_index];
            out.push_str(&format!(
                "<span ttm:role=\"x-bg\" begin=\"{:.3}\" end=\"{:.3}\">",
                bg.start as f64 / 1000.0,
                bg.end as f64 / 1000.0
            ));
            render_words(bg, &mut out);
            render_role(&mut out, "x-translation", translations[bg_index].as_deref());
            render_role(&mut out, "x-roman", pronunciations[bg_index].as_deref());
            out.push_str("</span>");
        }
        out.push_str("</p>");
    }
    out.push_str("</div></body>");
    Some(wrap(&out, word))
}
fn lrc_ttml_filtered(
    text: &str,
    translation: Option<&str>,
    context: Option<LyricContext<'_>>,
) -> Option<String> {
    let lines = lrc_lines(text);
    if all_pure_music(&lines) {
        return None;
    }
    render_lyrics(
        filter_info_lines(lines, context),
        false,
        translation.map(lrc_lines).unwrap_or_default(),
        Vec::new(),
    )
}
fn qrc_ttml_filtered(
    text: &str,
    translation: Option<&str>,
    pronunciation: Option<&str>,
    context: Option<LyricContext<'_>>,
) -> Option<String> {
    let lines = qrc_lines(text);
    if all_pure_music(&lines) {
        return None;
    }
    render_lyrics(
        filter_info_lines(lines, context),
        true,
        translation.map(lrc_lines).unwrap_or_default(),
        pronunciation.map(qrc_lines).unwrap_or_default(),
    )
}
fn converted_lyrics_with_context(raw: &Value, context: Option<LyricContext<'_>>) -> Vec<Lyric> {
    let raw = raw.get("data").unwrap_or(raw);
    let mut candidates = Vec::new();
    if let Some(text) = field_text(raw, "qrc").and_then(|text| {
        qrc_ttml_filtered(
            text,
            field_text(raw, "qrctrans"),
            field_text(raw, "qrcroma"),
            context,
        )
    }) {
        candidates.push(Lyric {
            source: "qrc",
            text,
        });
    }
    if let Some(text) = field_text(raw, "lrc")
        .and_then(|text| lrc_ttml_filtered(text, field_text(raw, "lrctrans"), context))
    {
        candidates.push(Lyric {
            source: "lrc",
            text,
        });
    }
    candidates
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LyricsSourceState {
    Success,
    None,
    Failed,
}
struct LyricsSource<T> {
    state: LyricsSourceState,
    value: Option<T>,
}

async fn fetch_amll_lyrics(qq: &Qq, mid: &str) -> LyricsSource<String> {
    let response = match qq
        .api
        .get(format!("{}/v1/lyrics/get", qq.amll))
        .query(&[("qqMusicId", &mid)])
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::debug!(error = %error.without_url(), "qq AMLL lyrics source failed");
            return LyricsSource {
                state: LyricsSourceState::Failed,
                value: None,
            };
        }
    };
    if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT {
        return LyricsSource {
            state: LyricsSourceState::None,
            value: None,
        };
    }
    if !response.status().is_success() {
        tracing::debug!(status = %response.status(), "qq AMLL lyrics source failed");
        return LyricsSource {
            state: LyricsSourceState::Failed,
            value: None,
        };
    }
    let raw = match response.json::<Value>().await {
        Ok(raw) => raw,
        Err(error) => {
            tracing::debug!(error = %error, "qq AMLL lyrics response decode failed");
            return LyricsSource {
                state: LyricsSourceState::Failed,
                value: None,
            };
        }
    };
    let Some(raw) = raw.pointer("/data/lyrics").and_then(Value::as_str) else {
        return LyricsSource {
            state: LyricsSourceState::None,
            value: None,
        };
    };
    let Some(text) = clean_ttml(raw) else {
        return LyricsSource {
            state: LyricsSourceState::None,
            value: None,
        };
    };
    LyricsSource {
        state: LyricsSourceState::Success,
        value: Some(text),
    }
}

async fn fetch_ttml_lyrics(qq: &Qq, mid: &str) -> LyricsSource<String> {
    let raw = match qq
        .request_with_status(
            "/lyric/ttml",
            &[("platform", "qq".into()), ("mid", mid.to_owned())],
        )
        .await
    {
        Ok(raw) => raw,
        Err(RequestError::Upstream(StatusCode::NOT_FOUND)) => {
            return LyricsSource {
                state: LyricsSourceState::None,
                value: None,
            };
        }
        Err(_) => {
            tracing::debug!("qq TTML lyrics source failed");
            return LyricsSource {
                state: LyricsSourceState::Failed,
                value: None,
            };
        }
    };
    let Some(content) = raw.get("content").and_then(Value::as_str) else {
        return LyricsSource {
            state: LyricsSourceState::None,
            value: None,
        };
    };
    let Some(text) = clean_ttml(content) else {
        return LyricsSource {
            state: LyricsSourceState::None,
            value: None,
        };
    };
    LyricsSource {
        state: LyricsSourceState::Success,
        value: Some(text),
    }
}

async fn fetch_new_lyrics(qq: &Qq, mid: &str) -> LyricsSource<Value> {
    let raw = match qq
        .request_with_status(
            "/lyric/new",
            &[("platform", "qq".into()), ("id", mid.to_owned())],
        )
        .await
    {
        Ok(raw) => raw,
        Err(RequestError::Upstream(StatusCode::NOT_FOUND)) => {
            return LyricsSource {
                state: LyricsSourceState::None,
                value: None,
            };
        }
        Err(_) => {
            tracing::debug!("qq QRC lyrics source failed");
            return LyricsSource {
                state: LyricsSourceState::Failed,
                value: None,
            };
        }
    };
    let raw = raw.get("data").cloned().unwrap_or(raw);
    let has_lyrics = ["qrc", "qrctrans", "qrcroma", "lrc", "lrctrans"]
        .iter()
        .any(|key| field_text(&raw, key).is_some_and(|text| !text.trim().is_empty()));
    LyricsSource {
        state: if has_lyrics {
            LyricsSourceState::Success
        } else {
            LyricsSourceState::None
        },
        value: has_lyrics.then_some(raw),
    }
}

async fn fetch_lyrics(qq: &Qq, mid: &str) -> ApiResult<(Lyrics, Duration)> {
    // Fetch all upstreams and the cold song context together. The context is only
    // needed to filter QRC/LRC metadata, so a context failure does not fail lyrics.
    let context = async {
        if let Some(song) = qq.songs.get(mid) {
            (Some(song), false)
        } else {
            match qq.song(mid).await {
                Ok(song) => (Some(song), false),
                Err(_) => {
                    tracing::debug!("qq lyrics song context unavailable");
                    (None, true)
                }
            }
        }
    };
    let (amll, ttml, new, (song, context_failed)) = tokio::join!(
        fetch_amll_lyrics(qq, mid),
        fetch_ttml_lyrics(qq, mid),
        fetch_new_lyrics(qq, mid),
        context,
    );
    let mut candidates = Vec::new();
    let mut raw_lyrics = RawLyrics::default();
    let mut context_failed_with_candidates = false;
    if let Some(text) = amll.value {
        candidates.push(Lyric {
            source: "amll",
            text,
        });
    }
    if let Some(text) = ttml.value {
        candidates.push(Lyric {
            source: "ttml",
            text,
        });
    }
    let mut new_state = new.state;
    if let Some(raw) = new.value {
        // Lyrics can be requested before the song details page has populated the
        // song cache. Reuse the normal song/info path on a cold cache so the
        // metadata-aware filters see the same title and artists. A failed info
        // request only drops the context; the lyric candidates remain usable.
        let context = song.as_ref().map(|song| LyricContext {
            name: &song.name,
            artists: &song.artists,
        });
        raw_lyrics.qrc = field_text(&raw, "qrc")
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned);
        raw_lyrics.trans = field_text(&raw, "qrctrans")
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned);
        raw_lyrics.roma = field_text(&raw, "qrcroma")
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned);
        let candidate_count = candidates.len();
        candidates.extend(converted_lyrics_with_context(&raw, context));
        context_failed_with_candidates = context_failed && candidates.len() > candidate_count;
        if candidates.len() == candidate_count {
            new_state = LyricsSourceState::None;
        }
    }
    let states = [amll.state, ttml.state, new_state];
    if states
        .iter()
        .all(|state| *state == LyricsSourceState::Failed)
    {
        return Err(unavailable());
    }
    let ttl = if states.contains(&LyricsSourceState::Failed) || context_failed_with_candidates {
        Duration::from_secs(300)
    } else if states.iter().all(|state| *state == LyricsSourceState::None) {
        Duration::from_secs(3600)
    } else {
        Duration::from_secs(604800)
    };
    Ok((
        Lyrics {
            candidates,
            raw_qrc: raw_lyrics.qrc.is_some(),
            raw_trans: raw_lyrics.trans.is_some(),
            raw_roma: raw_lyrics.roma.is_some(),
            raw: raw_lyrics,
        },
        ttl,
    ))
}
async fn lyrics(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
) -> ApiResult<Json<Lyrics>> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    if let Some(data) = qq.lyrics.get(&mid) {
        return Ok(Json(data));
    }
    let (data, ttl) = fetch_lyrics(qq, &mid).await?;
    qq.lyrics.put_with_ttl(mid, data.clone(), ttl);
    Ok(Json(data))
}
async fn raw_lyrics(
    State(state): State<Arc<AppState>>,
    Path(mid): Path<String>,
    Query(query): Query<RawLyricsQuery>,
) -> ApiResult<Response> {
    let qq = adapter(&state)?;
    check_mid(&mid)?;
    let kind = raw_lyrics_kind(query.kind.as_deref())?;
    let data = if let Some(data) = qq.lyrics.get(&mid) {
        data
    } else {
        let (data, ttl) = fetch_lyrics(qq, &mid).await?;
        qq.lyrics.put_with_ttl(mid.clone(), data.clone(), ttl);
        data
    };
    let text = match kind {
        "qrc" => data.raw.qrc,
        "qrctrans" => data.raw.trans,
        "qrcroma" => data.raw.roma,
        _ => unreachable!("validated raw lyric kind"),
    }
    .ok_or_else(missing)?;
    let song = qq.song(&mid).await?;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CONTENT_DISPOSITION, lyrics_disposition(&song, kind))
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(text))
        .unwrap())
}

#[cfg(test)]
#[path = "../tests/qq_adapter/mod.rs"]
mod tests;
