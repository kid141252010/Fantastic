//! QQ upstream contract tests. Each upstream is a local axum server bound to a
//! random loopback port, so these tests do not need network access or fixtures
//! outside the repository.

use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use am_hook::{router, state::{AppState, Config}};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::{net::TcpListener, task::JoinHandle};
use tower::ServiceExt;

async fn serve(app: Router) -> Option<(String, JoinHandle<()>)> {
    // Some managed test sandboxes deny loopback binds. CI runs these tests
    // with sockets enabled; a denied bind is reported and skipped locally.
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("skipping loopback QQ mock: {error}");
            return None;
        }
    };
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Some((url, task))
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
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 100_000).await.unwrap(),
    )
    .unwrap();
    assert_eq!(body["total"], 7);
    assert_eq!(body["songs"][0]["id"], "mid");
    let query = queries.lock().unwrap()[0].clone();
    assert!(query.contains("limit=2"));
    assert!(query.contains("offset=4"));

    let response = get(state, "/qq/search?q=missing").await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
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
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 100_000).await.unwrap(),
    )
    .unwrap();
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
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), 100_000).await.unwrap(),
        )
        .unwrap();
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
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 100_000).await.unwrap(),
    )
    .unwrap();
    assert_eq!(body["candidates"].as_array().unwrap().len(), 1);
    assert_eq!(body["candidates"][0]["source"], "lrc");
    assert!(body["candidates"][0]["text"].as_str().unwrap().contains("keep"));
    assert_eq!(*info_calls.lock().unwrap(), 1);
    task.abort();
}

#[tokio::test]
async fn audio_url_non_2xx_is_hidden_as_502() {
    let upstream = Router::new().route(
        "/extra/music/url",
        any(|_: Request<Body>| async { StatusCode::SERVICE_UNAVAILABLE }),
    );
    let Some((base, task)) = serve(upstream).await else {
        return;
    };
    let state = Arc::new(AppState::with_config(config(base.clone(), base)));
    let response = get(state, "/qq/song/mid/play?quality=HQ").await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    task.abort();
}
