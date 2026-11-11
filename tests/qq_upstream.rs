//! QQ upstream contract tests. Each upstream is a local axum server bound to a
//! random loopback port, so these tests do not need network access or fixtures
//! outside the repository.

use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use fantastic::{router, state::{AppState, Config}};
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::{net::TcpListener, task::JoinHandle};
use tower::ServiceExt;

async fn serve(app: Router) -> Option<(String, JoinHandle<()>)> {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => {
            if std::env::var("FANTASTIC_TEST_SKIP_LOOPBACK").as_deref() == Ok("1") {
                eprintln!("skipping loopback QQ mock: {error}");
                return None;
            }
            panic!("loopback QQ mock bind failed: {error}; set FANTASTIC_TEST_SKIP_LOOPBACK=1 to skip");
        }
    };
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Some((url, task))
}

#[allow(dead_code)]
fn counting_upstream(routes: fn(&str) -> Response) -> (Router, Arc<Mutex<HashMap<String, usize>>>) {
    let counts = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let seen = counts.clone();
    let router = Router::new().fallback(any(move |request: Request<Body>| {
        let path = request.uri().path().to_owned();
        let seen = seen.clone();
        async move {
            *seen.lock().unwrap().entry(path.clone()).or_default() += 1;
            routes(&path)
        }
    }));
    (router, counts)
}

fn config(api: String, amll: String) -> Config {
    Config {
        wrapper_url: "http://127.0.0.1:1".into(),
        amll_api: amll,
        qq_api: Some(api),
        amp_keepalive: Duration::ZERO,
        amp_cache_mb: 0,
        wrapper_rate: 0,
        wrapper_concurrency: 0,
        wrapper_auth: None,
        wrapper_relay: true,
        wrapper_hint: None,
        client_rate: 0,
        client_concurrency: 0,
        client_ip_header: None,
    }
}

async fn get(state: Arc<AppState>, uri: &str) -> axum::response::Response {
    router(state)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap()
}

fn song(mid: &str) -> Value {
    json!({
        "mid": mid,
        "name": "Song",
        "ar": [{"name": "Artist"}],
        "al": {"mid": "album", "name": "Album"},
        "dt": 1000,
        "file": {}
    })
}

#[tokio::test]
async fn search_parses_result_and_forwards_limit_offset() {
    let queries = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = queries.clone();
    let upstream = Router::new()
        .route(
            "/cloudsearch",
            any(move |State(seen): State<Arc<Mutex<Vec<String>>>>, request: Request<Body>| async move {
                seen.lock().unwrap().push(request.uri().query().unwrap_or_default().into());
                let query = request.uri().query().unwrap_or_default();
                if query.contains("keywords=missing") {
                    return Json(json!({})).into_response();
                }
                Json(json!({
                    "result": {
                        "songCount": 7,
                        "songs": [song("mid")]
                    }
                }))
                .into_response()
            }),
        )
        .with_state(seen);
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));

    let response = get(state.clone(), "/qq/search?q=hello%20world&limit=2&offset=4").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["total"], 7);
    assert_eq!(body["songs"][0]["id"], "mid");
    let query = queries.lock().unwrap()[0].clone();
    assert!(query.contains("keywords=hello+world") || query.contains("keywords=hello%20world"));
    assert!(query.contains("limit=2"));
    assert!(query.contains("offset=4"));
    assert!(query.split('&').any(|pair| pair == "type=1"));

    let response = get(state, "/qq/search?q=missing").await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    task.abort();
}

#[tokio::test]
async fn album_and_artist_pages_normalize_upstream_shapes() {
    let upstream = Router::new().fallback(any(|request: Request<Body>| async move {
        let path = request.uri().path();
        match path {
            "/album" => Json(json!({
                "code": 200,
                "album": {
                    "id": "albummid", "name": "Album", "picUrl": "https://y.qq.com/music/photo_new/T002R500x500M000albummid_5.jpg",
                    "publishTime": 1704067200000u64, "size": 2, "subType": "录音室版", "description": "About",
                    "artists": [{"id": "artistmid", "name": "Artist"}]
                },
                "songs": [song("songa"), song("songb")]
            })).into_response(),
            "/artist/detail" => Json(json!({
                "code": 200,
                "data": {"artist": {"id": "artistmid", "name": "Artist", "avatar": "http://y.gtimg.cn/avatar.jpg", "cover": "https://y.gtimg.cn/cover.jpg", "briefDesc": "Bio", "musicSize": 2, "albumSize": 1}}
            })).into_response(),
            "/artist/songs" => Json(json!({"code": 200, "total": 101, "songs": [song("songa")] })).into_response(),
            "/artist/album" => Json(json!({"code": 200, "hotAlbums": [{"id": "albummid", "name": "Album", "picUrl": "https://y.qq.com/music/photo_new/T002R500x500M000albummid_5.jpg", "publishTime": 1704067200000u64, "size": 2}]})).into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }));
    let Some((base, task)) = serve(upstream).await else { return; };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));

    let response = get(state.clone(), "/qq/album/albummid").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["album"]["mid"], "albummid");
    assert_eq!(body["album"]["artists"][0], json!({"name": "Artist", "mid": "artistmid"}));
    assert_eq!(body["songs"].as_array().unwrap().len(), 2);

    let response = get(state.clone(), "/qq/artist/artistmid").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["artist"]["avatar"], "https://y.gtimg.cn/avatar.jpg");
    assert_eq!(body["totalSongs"], 101);
    assert_eq!(body["albums"][0]["mid"], "albummid");
    assert_eq!(body["songs"].as_array().unwrap().len(), 1);
    task.abort();
}

#[tokio::test]
async fn artist_page_degrades_when_songs_upstream_fails() {
    let upstream = Router::new().fallback(any(|request: Request<Body>| async move {
        match request.uri().path() {
            "/artist/detail" => Json(json!({"code": 200, "data": {"artist": {"id": "artistmid", "name": "Artist"}}})).into_response(),
            "/artist/songs" => StatusCode::BAD_GATEWAY.into_response(),
            "/artist/album" => Json(json!({"code": 200, "hotAlbums": []})).into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }));
    let Some((base, task)) = serve(upstream).await else { return; };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let response = get(state, "/qq/artist/artistmid").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["songs"], json!([]));
    assert_eq!(body["albums"], json!([]));
    task.abort();
}

fn lyric_info(mid: &str) -> Value {
    json!({"data": [song(mid)]})
}

fn valid_ttml(label: &str) -> String {
    format!("<tt><body><div><p begin=\"0\" end=\"1\">{label}</p></div></body></tt>")
}

#[tokio::test]
async fn lyrics_use_source_priority_and_cache_partial_failures() {
    let calls = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let counts = calls.clone();
    let upstream = Router::new()
        .fallback(any(move |State(counts): State<Arc<Mutex<HashMap<String, usize>>>>, request: Request<Body>| async move {
            let path = request.uri().path().to_owned();
            *counts.lock().unwrap().entry(path.clone()).or_default() += 1;
            match path.as_str() {
                "/v1/lyrics/get" => (
                    StatusCode::OK,
                    Json(json!({"data":{"lyrics":valid_ttml("amll")}})),
                )
                    .into_response(),
                "/lyric/ttml" => Json(json!({"content":valid_ttml("ttml")})).into_response(),
                "/lyric/new" => Json(json!({"data":{
                    "qrc":"[0,1000]qrc",
                    "lrc":"[00:01.00]lrc"
                }}))
                .into_response(),
                "/extra/music/info" => Json(lyric_info("priority")).into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }));
    let Some((base, task)) = serve(upstream.with_state(counts.clone())).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let response = get(state, "/qq/song/priority/lyrics").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let sources: Vec<_> = body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|candidate| candidate["source"].as_str().unwrap())
        .collect();
    assert_eq!(sources, ["amll", "ttml", "qrc", "lrc"]);
    assert_eq!(calls.lock().unwrap()["/v1/lyrics/get"], 1);
    task.abort();

    // A failed source receives the short failure-aware TTL. The second request
    // is served by that cache entry and does not re-hit any upstream.
    let calls = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let counts = calls.clone();
    let upstream = Router::new()
        .fallback(any(move |State(counts): State<Arc<Mutex<HashMap<String, usize>>>>, request: Request<Body>| async move {
            let path = request.uri().path().to_owned();
            *counts.lock().unwrap().entry(path.clone()).or_default() += 1;
            match path.as_str() {
                "/v1/lyrics/get" | "/lyric/new" => StatusCode::BAD_GATEWAY.into_response(),
                "/lyric/ttml" => Json(json!({"content":valid_ttml("fallback")})).into_response(),
                "/extra/music/info" => Json(lyric_info("partial")).into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }));
    let Some((base, task)) = serve(upstream.with_state(counts.clone())).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    for _ in 0..2 {
        let response = get(state.clone(), "/qq/song/partial/lyrics").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["candidates"][0]["source"], "ttml");
    }
    let calls = calls.lock().unwrap();
    assert_eq!(calls["/v1/lyrics/get"], 1);
    assert_eq!(calls["/lyric/ttml"], 1);
    assert_eq!(calls["/lyric/new"], 1);
    assert_eq!(calls["/extra/music/info"], 1);
    task.abort();
}

#[tokio::test]
async fn cold_lyrics_fetch_uses_song_context_for_filtering() {
    let info_calls = Arc::new(Mutex::new(0usize));
    let seen_info_calls = info_calls.clone();
    let upstream = Router::new().fallback(any(move |request: Request<Body>| {
        let seen_info_calls = seen_info_calls.clone();
        async move {
            if request.uri().path() == "/extra/music/info" {
                *seen_info_calls.lock().unwrap() += 1;
            }
            match request.uri().path() {
                "/v1/lyrics/get" | "/lyric/ttml" => StatusCode::NO_CONTENT.into_response(),
                "/lyric/new" => Json(json!({"data":{
                    "qrc":"[0,1000]Song - Artist(0,1000)",
                    "lrc":"[00:01.00]keep"
                }}))
                .into_response(),
                "/extra/music/info" => Json(lyric_info("cold")).into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }));
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let response = get(state, "/qq/song/cold/lyrics").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["candidates"].as_array().unwrap().len(), 1);
    assert_eq!(body["candidates"][0]["source"], "lrc");
    assert!(body["candidates"][0]["text"].as_str().unwrap().contains("keep"));
    assert_eq!(*info_calls.lock().unwrap(), 1);
    task.abort();
}

#[tokio::test]
async fn audio_url_non_2xx_is_hidden_as_502() {
    let hits = Arc::new(Mutex::new(0usize));
    let queries = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = hits.clone();
    let seen_queries = queries.clone();
    let upstream = Router::new().route(
        "/extra/music/url",
        any(move |request: Request<Body>| {
            let seen = seen.clone();
            let seen_queries = seen_queries.clone();
            async move {
                *seen.lock().unwrap() += 1;
                seen_queries.lock().unwrap().push(request.uri().query().unwrap_or_default().into());
                StatusCode::FORBIDDEN
            }
        }),
    );
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let response = get(state, "/qq/song/mid/play?quality=HQ").await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(*hits.lock().unwrap(), 1);
    let query = queries.lock().unwrap()[0].clone();
    assert!(query.contains("platform=qq"));
    assert!(query.contains("mid=mid"));
    assert!(query.contains("music_quality=HQ"));
    assert!(query.contains("audio_api=2"));
    task.abort();
}

#[tokio::test]
async fn raw_lyrics_cold_path_caches_payload_and_preserves_field_names() {
    let calls = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let queries = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_calls = calls.clone();
    let seen_queries = queries.clone();
    let upstream = Router::new().fallback(any(move |request: Request<Body>| {
        let calls = seen_calls.clone();
        let queries = seen_queries.clone();
        async move {
            let path = request.uri().path().to_owned();
            *calls.lock().unwrap().entry(path.clone()).or_default() += 1;
            queries.lock().unwrap().push(request.uri().query().unwrap_or_default().to_owned());
            match path.as_str() {
                "/v1/lyrics/get" | "/lyric/ttml" => StatusCode::NOT_FOUND.into_response(),
                "/lyric/new" => Json(json!({"data": {
                    "qrc": "[0,1000]qrc",
                    "qrctrans": "[00:01.00]translation",
                    "qrcroma": "[00:01.00]roman"
                }})).into_response(),
                "/extra/music/info" => Json(lyric_info("rawcold")).into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }));
    let Some((base, task)) = serve(upstream).await else { return; };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let first = get(state.clone(), "/qq/song/rawcold/lyrics/raw?kind=qrc").await;
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(to_bytes(first.into_body(), 100_000).await.unwrap().as_ref(), b"[0,1000]qrc");
    let second = get(state.clone(), "/qq/song/rawcold/lyrics/raw?kind=qrctrans").await;
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(to_bytes(second.into_body(), 100_000).await.unwrap().as_ref(), b"[00:01.00]translation");
    let third = get(state.clone(), "/qq/song/rawcold/lyrics").await;
    assert_eq!(third.status(), StatusCode::OK);
    let body = json_body(third).await;
    assert_eq!(body["rawQrc"], true);
    assert_eq!(body["rawTrans"], true);
    assert_eq!(body["rawRoma"], true);
    let calls = calls.lock().unwrap();
    assert_eq!(calls["/lyric/new"], 1);
    assert_eq!(calls["/extra/music/info"], 1);
    let queries = queries.lock().unwrap();
    assert!(queries.iter().any(|query| query.contains("qqMusicId=rawcold")));
    assert!(queries.iter().any(|query| query.contains("mid=rawcold")));
    assert!(queries.iter().any(|query| query.contains("id=rawcold")));
    assert!(queries.iter().any(|query| query.contains("mids=rawcold")));
    task.abort();
}

#[tokio::test]
async fn search_types_suggest_playlist_and_mv_routes_normalize_upstream_shapes() {
    let upstream = Router::new().fallback(any(|request: Request<Body>| async move {
        let path = request.uri().path().to_owned();
        let query = request.uri().query().unwrap_or_default().to_owned();
        let body = match path.as_str() {
            "/cloudsearch" if query.contains("type=10&") || query.ends_with("type=10") => json!({"result": {
                "albumCount": 3,
                "albums": [{"id": "0041WVfh2vtlJE", "name": "Album", "picUrl": "http://y.gtimg.cn/music/photo_new/T002R180x180M0000041WVfh2vtlJE_1.jpg",
                    "publishTime": 1749139200, "artists": [{"id": "0025NhlN2yWrP4", "name": "Artist"}]}]
            }}),
            "/cloudsearch" if query.contains("type=100") && !query.contains("type=1000") && !query.contains("type=1004") => json!({"result": {
                "artistCount": 1,
                "artists": [{"id": "0025NhlN2yWrP4", "name": "Artist", "picUrl": "http://y.gtimg.cn/music/photo_new/T001R150x150M0000025NhlN2yWrP4_11.jpg", "albumSize": 43, "mvSize": 9}]
            }}),
            "/cloudsearch" if query.contains("type=1000") => json!({"result": {
                "playlistCount": 1,
                "playlists": [{"id": "7039749142", "name": "List", "coverImgUrl": "http://qpic.y.qq.com/music_cover/x/300?n=1",
                    "trackCount": 99, "playCount": 5, "creator": {"nickname": "Me"}, "description": "Desc"}]
            }}),
            "/cloudsearch" if query.contains("type=1004") => json!({"result": {
                "mvCount": 1,
                "mvs": [{"id": "w0026q7f01a", "name": "MV", "cover": "http://y.gtimg.cn/music/photo_new/T015R640x360M10300061J2t0b0PPW.jpg",
                    "duration": 317000, "playCount": 7, "artistId": "0025NhlN2yWrP4", "artistName": "Artist"}]
            }}),
            "/search/suggest" => json!({"result": {
                "songs": [{"id": "Song - Artist", "mid": "0039MnYb0qxYhV", "name": "Song - Artist", "relname": "Song", "artists": [{"id": 0, "name": "Artist"}]}],
                "artists": [{"id": "004Ya5GP1ckXTA", "name": "Artist", "picUrl": "http://y.gtimg.cn/music/photo_new/T001R150x150M000004Ya5GP1ckXTA_3.jpg"}],
                "albums": [{"id": "002mGsM62gHnX9", "name": "Album", "artist": {"id": 0, "name": "Artist"}}]
            }}),
            // The detail request fails for some playlists; tracks must still load.
            "/playlist/detail" => return (StatusCode::NOT_FOUND, Json(json!({"code": 404}))).into_response(),
            "/playlist/track/all" => json!({"code": 200, "songs": [song("mid1"), song("mid2")]}),
            "/mv/detail" => json!({"code": 200, "data": {"id": "w0026q7f01a", "name": "MV", "duration": 317, "playCount": 1,
                "artists": [{"id": "0025NhlN2yWrP4", "name": "Artist"}], "desc": "Desc", "publishTime": "2003-07-29"}}),
            "/mv/url" => json!({"code": 200, "data": {"id": "w0026q7f01a", "url": "http://mv6.music.tc.qq.com/abc/v.mp4?fname=v.mp4"}}),
            _ => return StatusCode::NOT_FOUND.into_response(),
        };
        Json(body).into_response()
    }));
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));

    let body = json_body(get(state.clone(), "/qq/search?q=x&type=album").await).await;
    assert_eq!(body["total"], 3);
    assert!(body.get("songs").is_none());
    assert_eq!(body["albums"][0]["mid"], "0041WVfh2vtlJE");
    assert_eq!(body["albums"][0]["cover"], "https://y.gtimg.cn/music/photo_new/T002R{w}x{h}M0000041WVfh2vtlJE_1.jpg");
    assert_eq!(body["albums"][0]["publishTime"], 1749139200000u64);
    assert_eq!(body["albums"][0]["artists"][0]["mid"], "0025NhlN2yWrP4");

    let body = json_body(get(state.clone(), "/qq/search?q=x&type=artist").await).await;
    assert_eq!(body["artists"][0]["avatar"], "https://y.gtimg.cn/music/photo_new/T001R{w}x{h}M0000025NhlN2yWrP4_11.jpg");
    assert_eq!(body["artists"][0]["albumSize"], 43);

    let body = json_body(get(state.clone(), "/qq/search?q=x&type=playlist").await).await;
    assert_eq!(body["playlists"][0]["id"], "7039749142");
    assert_eq!(body["playlists"][0]["cover"], "https://qpic.y.qq.com/music_cover/x/300?n=1");
    assert_eq!(body["playlists"][0]["creator"], "Me");

    let body = json_body(get(state.clone(), "/qq/search?q=x&type=mv").await).await;
    assert_eq!(body["mvs"][0]["vid"], "w0026q7f01a");
    assert_eq!(body["mvs"][0]["durationMs"], 317000);
    assert_eq!(body["mvs"][0]["artists"][0]["mid"], "0025NhlN2yWrP4");

    assert_eq!(get(state.clone(), "/qq/search?q=x&type=video").await.status(), StatusCode::BAD_REQUEST);

    let body = json_body(get(state.clone(), "/qq/suggest?q=x").await).await;
    assert_eq!(body["songs"][0], json!({"mid": "0039MnYb0qxYhV", "name": "Song", "artist": "Artist"}));
    assert_eq!(body["artists"][0]["mid"], "004Ya5GP1ckXTA");
    assert_eq!(body["albums"][0]["cover"], "https://y.gtimg.cn/music/photo_new/T002R{w}x{h}M000002mGsM62gHnX9.jpg");

    let body = json_body(get(state.clone(), "/qq/playlist/7039749142?limit=2").await).await;
    assert!(body["playlist"].is_null());
    assert_eq!(body["songs"].as_array().unwrap().len(), 2);
    assert_eq!(body["more"], true);

    let body = json_body(get(state.clone(), "/qq/mv/w0026q7f01a").await).await;
    assert_eq!(body["durationMs"], 317000);
    assert_eq!(body["publishDate"], "2003-07-29");
    assert_eq!(body["description"], "Desc");

    let response = get(state.clone(), "/qq/mv/w0026q7f01a/play").await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()["location"], "https://mv6.music.tc.qq.com/abc/v.mp4?fname=v.mp4");
    assert_eq!(get(state, "/qq/mv/..%2Fx/play").await.status(), StatusCode::BAD_REQUEST);
    task.abort();
}

#[tokio::test]
async fn song_info_includes_whitelisted_meta() {
    let upstream = Router::new().fallback(any(|| async {
        Json(json!({"0039MnYb0qxYhV": {
            "extras": {"secret": "x"},
            "info": {
                "company": {"content": [{"value": "Label"}]},
                "genre": {"content": [{"value": "Pop"}]},
                "lan": {"content": [{"value": "国语"}]},
                "intro": {"content": [{"value": "Intro"}]},
                "pub_time": {"content": [{"value": "2003-07-31"}]}
            },
            "track_info": {"mid": "0039MnYb0qxYhV", "name": "晴天", "singer": [{"name": "周杰伦", "mid": "0025NhlN2yWrP4"}],
                "album": {"mid": "000MkMni19ClKG", "name": "叶惠美"}, "interval": 269, "index_album": 3, "index_cd": 0,
                "mv": {"vid": "w0026q7f01a"}}
        }}))
    }));
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let body = json_body(get(state, "/qq/song/0039MnYb0qxYhV").await).await;
    assert_eq!(body["mv"], "w0026q7f01a");
    assert_eq!(body["meta"], json!({"company": "Label", "genre": "Pop", "language": "国语", "intro": "Intro",
        "publishDate": "2003-07-31", "trackNumber": 3, "disc": 1}));
    assert!(!body.to_string().contains("secret"));
    task.abort();
}
