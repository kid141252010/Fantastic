//! --local-wrapper-only: the server never relays wrapper-lite requests, and host.js tells the page
//! to use a browser-side (local) wrapper-lite with the operator's defaults and hint.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use fantastic::state::{AppState, Config};
use axum::Router;

/// serverless 部署交给页面的版本号（serverless/core.mjs 的 VERSION）与二进制相同；发布时一起改
#[test]
fn serverless_version_matches_cargo() {
    let expected = format!("export const VERSION = '{}';", env!("CARGO_PKG_VERSION"));
    assert!(include_str!("../serverless/core.mjs").contains(&expected), "update serverless/core.mjs: {expected}");
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

fn config(wrapper_url: String, relay: bool) -> Config {
    let fallback = fantastic::state::WrapperFallbackConfig {
        url: wrapper_url.clone(), auth: Some("Bearer fallback-secret".into()), rate: 1, concurrency: 1,
    };
    Config {
        wrapper_url,
        amll_api: "https://api.amll.dev".into(),
        qq_api: None,
        qq_cdn: fantastic::qq::CdnMode::Off,
        amp_keepalive: Duration::ZERO,
        amp_cache_mb: 0,
        wrapper_rate: 5,
        wrapper_concurrency: 3,
        wrapper_auth: Some("Bearer operator-secret".into()),
        wrapper_fallback: Some(fallback),
        wrapper_relay: relay,
        wrapper_hint: Some("Get a token at https://token.test".into()),
        client_rate: 0,
        client_concurrency: 0,
        client_ip_header: None,
    }
}

#[tokio::test]
async fn local_wrapper_only_never_relays() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let wrapper = serve(Router::new().fallback(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        async { axum::Json(serde_json::json!({ "code": 0, "data": { "regions": ["us"] } })) }
    }))
    .await;

    let base = serve(fantastic::router(Arc::new(AppState::with_config(config(wrapper.clone(), false))))).await;
    let client = reqwest::Client::new();

    let host = client.get(format!("{base}/assets/host.js")).send().await.unwrap();
    assert_eq!(host.headers()["content-type"], "text/javascript; charset=utf-8");
    let text = host.text().await.unwrap();
    let json = text.strip_prefix("window.FANTASTIC_HOST = ").and_then(|s| s.strip_suffix(";\n")).unwrap();
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "serverWrapper": false,
            "amllApi": "https://api.amll.dev",
            "qqEnabled": false,
            "ncmEnabled": true,
            "version": env!("CARGO_PKG_VERSION"),
            "wrapper": { "url": wrapper, "rate": 5, "concurrency": 3 },
            "hint": "Get a token at https://token.test",
        })
    );
    assert!(!text.contains("operator-secret"), "--wrapper-auth is never sent to pages");
    assert!(!text.contains("fallback-secret"), "fallback auth is never sent to pages");

    for path in ["/status", "/key?adamId=1&uri=skd://x", "/lyrics/1", "/parse/song/1", "/parse/mv/1", "/mv/webplayback/1"] {
        let response = client.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(response.status(), 501, "{path}");
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["code"], 1, "{path}");
    }
    let license = client.post(format!("{base}/mv/license")).json(&serde_json::json!({})).send().await.unwrap();
    assert_eq!(license.status(), 501);
    assert_eq!(calls.load(Ordering::SeqCst), 0, "wrapper-lite is never contacted");

    // 默认仍转发；host.js 不带本地模式默认值
    let relay = serve(fantastic::router(Arc::new(AppState::with_config(config(wrapper, true))))).await;
    let text = client.get(format!("{relay}/assets/host.js")).send().await.unwrap().text().await.unwrap();
    assert_eq!(
        text,
        format!("window.FANTASTIC_HOST = {{\"amllApi\":\"https://api.amll.dev\",\"hint\":\"Get a token at https://token.test\",\"ncmEnabled\":true,\"qqEnabled\":false,\"serverWrapper\":true,\"version\":\"{}\"}};\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(client.get(format!("{relay}/status")).send().await.unwrap().status(), 200);
    assert_eq!(calls.load(Ordering::SeqCst), 2, "primary and fallback are each checked once");
}

#[tokio::test]
async fn status_merges_regions_primary_first_and_caches_both() {
    use axum::http::{HeaderMap, StatusCode};
    use axum::response::IntoResponse;
    let mode = Arc::new(AtomicUsize::new(0));
    let calls = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
    let mut urls = Vec::new();
    for index in 0..2 {
        let (mode, calls) = (mode.clone(), calls.clone());
        urls.push(serve(Router::new().fallback(move |headers: HeaderMap| {
            let mode = mode.load(Ordering::SeqCst);
            calls[index].fetch_add(1, Ordering::SeqCst);
            async move {
                assert_eq!(headers["authorization"], if index == 0 { "Bearer operator-secret" } else { "Bearer fallback-secret" });
                if mode & (1 << index) != 0 {
                    // 即使 JSON 声称成功，也不能采信 HTTP 错误响应中的地区
                    return (StatusCode::BAD_GATEWAY, axum::Json(serde_json::json!({ "code": 0, "data": { "regions": ["gb"] } }))).into_response();
                }
                if index == 0 { tokio::time::sleep(Duration::from_millis(20)).await; }
                let regions = if index == 0 { serde_json::json!(["CN", "jp", "cn", "bad", 7]) } else { serde_json::json!(["US", "JP", "de"]) };
                axum::Json(serde_json::json!({ "code": 0, "data": { "regions": regions } })).into_response()
            }
        })).await);
    }
    let mut cfg = config(urls[0].clone(), true);
    let fallback = cfg.wrapper_fallback.as_mut().unwrap();
    fallback.url = urls[1].clone();
    fallback.rate = 0;
    cfg.wrapper_rate = 0;
    let client = reqwest::Client::new();
    let base = serve(fantastic::router(Arc::new(AppState::with_config(cfg.clone())))).await;
    for _ in 0..3 {
        let reply = client.get(format!("{base}/status")).send().await.unwrap();
        assert_eq!(reply.status(), 200);
        assert_eq!(reply.json::<serde_json::Value>().await.unwrap()["regions"], serde_json::json!(["cn", "jp", "us", "de"]));
    }
    assert_eq!(calls[0].load(Ordering::SeqCst), 1);
    assert_eq!(calls[1].load(Ordering::SeqCst), 1);
    for (failed, expected, status) in [
        (1, serde_json::json!(["us", "jp", "de"]), 200),
        (2, serde_json::json!(["cn", "jp"]), 200),
        (3, serde_json::json!([]), 502),
    ] {
        mode.store(failed, Ordering::SeqCst);
        let base = serve(fantastic::router(Arc::new(AppState::with_config(cfg.clone())))).await;
        let reply = client.get(format!("{base}/status")).send().await.unwrap();
        assert_eq!(reply.status(), status);
        assert_eq!(reply.json::<serde_json::Value>().await.unwrap()["regions"], expected);
    }
    assert_eq!(calls[0].load(Ordering::SeqCst), 4, "one probe per primary per cache window");
    assert_eq!(calls[1].load(Ordering::SeqCst), 4, "one probe per fallback per cache window");
}
