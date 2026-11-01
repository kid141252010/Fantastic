//! QQ adapter unit tests, including local axum upstream contract checks.
use super::*;
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use axum::{body::to_bytes, http::Request, routing::any, Router};
use tokio::net::TcpListener;
use tower::ServiceExt;

fn cli(args: &[&str]) -> crate::cli::Cli {
    use clap::{CommandFactory, FromArgMatches};
    let matches = crate::cli::Cli::command()
        .mut_arg("qq_api", |arg| arg.env(None::<&str>))
        .mut_arg("qq_proxy", |arg| arg.env(None::<&str>))
        .try_get_matches_from(args)
        .unwrap();
    crate::cli::Cli::from_arg_matches(&matches).unwrap()
}

#[test]
fn proxy_environment_is_parsed_and_cli_takes_precedence() {
    use clap::{CommandFactory, Parser};
    const PROXY: &str = "socks5://127.0.0.1:11080";
    const CHILD: &str = "FANTASTIC_QQ_PROXY_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let cli = crate::cli::Cli::try_parse_from(["fantastic"]).unwrap();
        assert_eq!(cli.qq_proxy.as_deref(), Some(PROXY));
        cli.config().unwrap();
        let cli =
            crate::cli::Cli::try_parse_from(["fantastic", "--qq-proxy", "http://127.0.0.1:11081"])
                .unwrap();
        assert_eq!(cli.qq_proxy.as_deref(), Some("http://127.0.0.1:11081"));
        let help = crate::cli::Cli::command().render_long_help().to_string();
        assert!(help.contains("FANTASTIC_QQ_PROXY"));
        assert!(!help.contains(PROXY));
        return;
    }
    // Use a subprocess so other tests never observe a mutated environment.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "qq::tests::proxy_environment_is_parsed_and_cli_takes_precedence",
        ])
        .env(CHILD, "1")
        .env("FANTASTIC_QQ_PROXY", PROXY)
        .env_remove("FANTASTIC_QQ_API")
        .env_remove("FANTASTIC_WRAPPER_AUTH")
        .env_remove("FANTASTIC_WRAPPER_HINT")
        .env_remove("FANTASTIC_AMLL_API")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn proxy_cli_parameter_reaches_all_qq_clients_without_changing_other_clients() {
    let default = AppState::new("http://127.0.0.1:12340".into());
    for proxy in ["socks5://127.0.0.1:11080", "http://127.0.0.1:11080"] {
        let cli = cli(&[
            "fantastic",
            "--qq-api",
            "https://example.invalid/api/web",
            "--qq-proxy",
            proxy,
        ]);
        assert_eq!(cli.qq_proxy.as_deref(), Some(proxy));
        let state =
            AppState::with_qq_proxy(cli.config().unwrap(), cli.qq_proxy.as_deref()).unwrap();
        let qq = state.qq.as_ref().unwrap();
        for client in [&qq.api, &qq.media, &qq.cover] {
            assert!(format!("{client:?}").contains("127.0.0.1:11080"));
        }
        // reqwest 0.12's Debug output is the only stable observable for proxy wiring here.
        assert_eq!(
            format!("{:?}", state.http_client),
            format!("{:?}", default.http_client)
        );
        assert_eq!(
            format!("{:?}", state.apple_client),
            format!("{:?}", default.apple_client)
        );
    }
}

#[test]
fn absent_or_blank_proxy_preserves_default_clients() {
    let cli = cli(&["fantastic", "--qq-api", "https://example.invalid/api/web"]);
    assert!(cli.qq_proxy.is_none());
    let default = AppState::with_config(cli.config().unwrap());
    let default = default.qq.as_ref().unwrap();
    for proxy in [None, Some(""), Some("  ")] {
        let state = AppState::with_qq_proxy(cli.config().unwrap(), proxy).unwrap();
        let qq = state.qq.as_ref().unwrap();
        assert_eq!(format!("{:?}", qq.api), format!("{:?}", default.api));
        assert_eq!(format!("{:?}", qq.media), format!("{:?}", default.media));
        assert_eq!(format!("{:?}", qq.cover), format!("{:?}", default.cover));
    }
}

#[test]
fn invalid_proxy_fails_at_startup_without_upstream_urls() {
    for proxy in ["http://[", "unsupported://127.0.0.1:11080"] {
        let error = Qq::new(
            "https://example.invalid/private-api",
            "https://example.invalid/private-amll",
            Some(proxy),
        )
        .err()
        .expect("invalid proxy must fail before making any request");
        assert!(error.starts_with("Invalid QQ proxy:"));
        let cli = cli(&[
            "fantastic",
            "--qq-api",
            "https://example.invalid/private-api",
            "--qq-proxy",
            proxy,
        ]);
        let message = cli.config().err().unwrap();
        assert!(message.starts_with("Invalid QQ proxy:"));
        for message in [message, error.to_string(), format!("{error:?}")] {
            assert!(!message.contains("example.invalid"));
            assert!(!message.contains("private-api"));
            assert!(!message.contains("private-amll"));
        }
        // Invalid proxy configuration is rejected even when QQ is disabled.
        assert!(self::cli(&["fantastic", "--qq-proxy", proxy])
            .config()
            .is_err());
    }
}

fn qq() -> Qq {
    Qq::new(
        "https://example.invalid/api/web",
        "https://example.invalid/amll",
        None,
    )
    .unwrap()
}
fn raw_song() -> Value {
    json!({"id":"0039MnYb0qxYhV", "name":"晴天", "ar":[{"id":1,"name":"周杰伦"}],
        "al":{"id":"003DFRzD192KKD","name":"叶惠美","picUrl":"https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg?version=5"},
        "dt":269000,"fee":1,"ttml":1,
        "file_url":"https://example.invalid/private", "server":"private", "account":{"token":"secret"}})
}
#[test]
fn mids_and_cdn_urls_are_restricted() {
    for mid in ["1", "abc123", "0039MnYb0qxYhV"] {
        assert!(valid_mid(mid));
    }
    for mid in ["", "../x", "abc-def", "x?y", "中文", &"a".repeat(33)] {
        assert!(!valid_mid(mid));
    }
    assert!(cdn_url("https://isure.stream.qqmusic.qq.com/song.m4a?vkey=test").is_some());
    for url in [
        "http://y.qq.com/x",
        "https://qq.com.evil.invalid/x",
        "https://example.invalid/qq.com/x",
        "https://user:secret@y.qq.com/x",
        "https://y.qq.com:444/x",
        "file:///tmp/x",
    ] {
        assert!(cdn_url(url).is_none());
    }
}
#[test]
fn cover_url_only_accepts_album_mids_and_uses_one_fixed_template() {
    for mid in ["1", "003DFRzD192KKD", &"a".repeat(32)] {
        assert_eq!(
            cover_url(mid).ok().unwrap(),
            format!("https://y.qq.com/music/photo_new/T002R500x500M000{mid}_5.jpg")
        );
    }
    for mid in [
        "",
        "../x",
        "a/b",
        "a\\b",
        "a?url=x",
        "a#x",
        "a.jpg",
        "a_5",
        "中文",
        "https://example.invalid/x",
        "%2F",
        &"a".repeat(33),
    ] {
        assert_eq!(cover_url(mid).err().unwrap().0, StatusCode::BAD_REQUEST);
    }
    // Never follow upstream redirects to another URL, even on the QQ CDN.
    assert!(format!("{:?}", qq().cover).contains("Policy(None)"));
}
#[tokio::test]
async fn cover_streams_jpeg_with_cache_headers_and_hides_upstream_failures() {
    let chunks = futures::stream::iter([
        Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"\xff\xd8")),
        Ok(bytes::Bytes::from_static(b"jpeg\xff\xd9")),
    ]);
    let upstream = axum::http::Response::builder()
        .header(header::SET_COOKIE, "upstream-private=value")
        .body(reqwest::Body::wrap_stream(chunks))
        .unwrap();
    let response = cover_response(upstream.into()).unwrap_or_else(|e| panic!("{}", e.0));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/jpeg");
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "public, max-age=86400"
    );
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    assert_eq!(
        to_bytes(response.into_body(), 100).await.unwrap(),
        b"\xff\xd8jpeg\xff\xd9"[..]
    );
    for (upstream, expected) in [
        (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND),
        (StatusCode::FOUND, StatusCode::BAD_GATEWAY),
        (StatusCode::FORBIDDEN, StatusCode::BAD_GATEWAY),
        (StatusCode::INTERNAL_SERVER_ERROR, StatusCode::BAD_GATEWAY),
    ] {
        let response = axum::http::Response::builder()
            .status(upstream)
            .header(header::LOCATION, "https://example.invalid/private")
            .body("private upstream error")
            .unwrap();
        let response = cover_response(response.into())
            .err()
            .unwrap()
            .into_response();
        assert_eq!(response.status(), expected);
        assert!(!response.headers().contains_key(header::CACHE_CONTROL));
        let body = to_bytes(response.into_body(), 1000).await.unwrap();
        assert!(!String::from_utf8_lossy(&body).contains("private"));
    }
}
#[test]
fn dto_is_a_whitelist_with_direct_cover_and_vip_flags() {
    let qq = qq();
    let song = qq.song_dto(&raw_song()).unwrap();
    let dto = serde_json::to_value(&song).unwrap();
    assert_eq!(dto["id"], "0039MnYb0qxYhV");
    assert_eq!(dto["artists"], json!([{ "name": "周杰伦", "mid": "1" }]));
    assert_eq!(dto["durationMs"], 269000);
    assert_eq!(
        dto["cover"],
        "https://y.qq.com/music/photo_new/T002R{w}x{h}M000003DFRzD192KKD_5.jpg?version=5"
    );
    assert_eq!(dto["vip"], true);
    assert_eq!(dto["ttml"], true);
    assert_eq!(dto.as_object().unwrap().len(), 9);
    let text = dto.to_string();
    for secret in [
        "file_url",
        "example.invalid",
        "private",
        "account",
        "picUrl",
    ] {
        assert!(!text.contains(secret), "{secret}");
    }
    assert!(qq.songs.get("0039MnYb0qxYhV").is_some());
    assert!(qq.song_dto(&json!({"id":"../x"})).is_none());
}
#[test]
fn dto_keeps_artist_names_and_valid_mids_in_order() {
    let song = qq().song_dto(&json!({
        "id": "song",
        "name": "Duet",
        "ar": [{"id": "artistA", "name": "A"}, {"mid": "artistB", "name": "B"}],
        "al": {"id": "album", "name": "Album"},
    })).unwrap();
    assert_eq!(serde_json::to_value(song).unwrap()["artists"], json!([
        {"name": "A", "mid": "artistA"}, {"name": "B", "mid": "artistB"}
    ]));
}
#[test]
fn dto_uses_an_empty_artist_mid_when_upstream_omits_or_rejects_it() {
    let song = qq().song_dto(&json!({
        "id": "song",
        "name": "Song",
        "ar": [{"name": "A"}, {"id": "bad-mid!", "name": "B"}],
        "al": {"id": "album", "name": "Album"},
    })).unwrap();
    assert_eq!(serde_json::to_value(song).unwrap()["artists"], json!([
        {"name": "A", "mid": ""}, {"name": "B", "mid": ""}
    ]));
}
#[test]
fn native_qq_info_dialect_is_supported() {
    let song = qq()
        .song_dto(
            &json!({"mid":"abc123","title":"Song","singer":[{"name":"Artist"}],
        "album":{"mid":"album1","title":"Album"},"interval":180,"pay":{"pay_play":1}}),
        )
        .unwrap();
    assert_eq!(song.name, "Song");
    assert_eq!(song.duration_ms, 180000);
    assert_eq!(song.album, "Album");
    assert!(song.vip);
}
#[test]
fn song_info_accepts_mid_keyed_map_on_cache_miss() {
    let qq = qq();
    let mid = "0039MnYb0qxYhV";
    let data = json!({
        "other123": {"track_info": {"mid": "other123"}},
        (mid): {"track_info": {"mid": mid, "title": "晴天"}}
    });
    assert!(qq.songs.get(mid).is_none());
    let song = qq.song_from_info(mid, &data).ok().unwrap();
    assert_eq!(song.id, mid);
    assert_eq!(song.name, "晴天");
    assert_eq!(qq.songs.get(mid).unwrap().id, mid);
}
#[test]
fn song_info_returns_missing_when_map_key_does_not_match() {
    let qq = qq();
    let mid = "0039MnYb0qxYhV";
    // Even matching track metadata must not be selected under another key.
    let data = json!({"other123": {"track_info": {"mid": mid, "title": "晴天"}}});
    let error = qq.song_from_info(mid, &data).err().unwrap();
    assert_eq!(error.0, StatusCode::NOT_FOUND);
    assert!(qq.songs.get(mid).is_none());
}
#[test]
fn song_info_checks_track_id_after_matching_map_key() {
    let mid = "0039MnYb0qxYhV";
    let data = json!({(mid): {"track_info": {"mid": "other123"}}});
    let error = qq().song_from_info(mid, &data).err().unwrap();
    assert_eq!(error.0, StatusCode::NOT_FOUND);
}
#[test]
fn song_info_preserves_existing_response_shapes() {
    let mid = "0039MnYb0qxYhV";
    let raw = raw_song();
    for data in [
        json!([raw]),
        json!({"data": [raw]}),
        json!({"data": raw}),
        raw,
    ] {
        let song = qq().song_from_info(mid, &data).ok().unwrap();
        assert_eq!(song.id, mid);
        assert_eq!(song.name, "晴天");
    }
}
#[test]
fn caches_expire_and_are_bounded() {
    let cache = Cache::new(2, 60);
    let before = Instant::now();
    cache.put("a".into(), 1);
    assert_eq!(cache.get("a"), Some(1));
    let expires = cache.entries.lock().unwrap().peek("a").unwrap().0;
    assert!(expires >= before + Duration::from_secs(59));
    assert!(expires <= Instant::now() + Duration::from_secs(60));
    cache.put("b".into(), 2);
    cache.put("c".into(), 3);
    assert_eq!(cache.get("a"), None);
    cache.entries.lock().unwrap().put(
        "expired".into(),
        (Instant::now() - Duration::from_secs(61), 4),
    );
    assert_eq!(cache.get("expired"), None);
    assert_eq!(qq().searches.ttl, Duration::from_secs(600));
    assert_eq!(qq().songs.ttl, Duration::from_secs(86400));
}
#[test]
fn ttml_keeps_timing_translation_and_agents_but_removes_credits_and_urls() {
    let text = r#"<tt><head><metadata><ttm:agent xml:id="v1" type="group"><ttm:name>private</ttm:name></ttm:agent><amll:meta key="author" value="https://example.invalid/private"/></metadata></head><!-- private --><body dur="3.000"><div><p begin="1.000" end="3.000" ttm:agent="v1" data-secret="private"><span begin="1.000" end="2.000">Hello</span><span ttm:role="x-translation">你好</span> https://example.invalid/private</p></div></body></tt>"#;
    let cleaned = clean_ttml(text).unwrap();
    for secret in [
        "private",
        "example.invalid",
        "<!--",
        "amll:meta",
        "ttm:name",
        "data-secret",
    ] {
        assert!(!cleaned.contains(secret), "{secret}");
    }
    assert!(cleaned.contains("itunes:timing=\"Word\""));
    assert!(cleaned.contains("ttm:role=\"x-translation\""));
    assert!(cleaned.contains("type=\"group\""));
    assert!(cleaned.contains("begin=\"1.000\""));
    assert!(
        clean_ttml("<!DOCTYPE tt><tt><body><div><p begin=\"0\">x</p></div></body></tt>").is_none()
    );
    assert!(
        clean_ttml("<!dOcTyPe tt><tt><body><div><p begin=\"0\">x</p></div></body></tt>").is_none()
    );
    let child_lang =
        clean_ttml("<tt><head xml:lang=\"fr\"/><body><div><p begin=\"0\">x</p></div></body></tt>")
            .unwrap();
    assert!(!child_lang.contains("xml:lang=\"fr\""));
    let root_lang =
        clean_ttml("<tt xml:lang=\"en\"><body><div><p begin=\"0\">x</p></div></body></tt>")
            .unwrap();
    assert!(root_lang.contains("<tt xml:lang=\"en\""));
    assert!(clean_ttml("not XML").is_none());
    assert!(clean_ttml("<!ENTITY secret SYSTEM \"file:///etc/passwd\"><tt><body><p begin=\"0\">x</p></body></tt>").is_none());
}
#[test]
fn lrc_fallback_handles_headers_offsets_repeated_stamps_and_xml_escaping() {
    let text = "[by:https://example.invalid/private]\n[offset:100]\n[00:01.00][00:03.00]A & B\n[00:05.0]Next";
    let cleaned = lrc_ttml_filtered(text, None, None).unwrap();
    assert!(!cleaned.contains("example.invalid"));
    assert!(cleaned.contains("begin=\"0.900\" end=\"2.900\">A &amp; B"));
    assert!(cleaned.contains("begin=\"2.900\" end=\"4.900\">A &amp; B"));
    assert!(lrc_ttml_filtered("[by:someone]\nno timestamps", None, None).is_none());
}
#[test]
fn qrc_word_timing_and_optional_fields_fall_through_safely() {
    let cleaned = qrc_ttml_filtered(
        "[1000,500]你(1000,200)好(1200,300)\n[2000,1000]World(2000,1000)",
        None,
        None,
        None,
    )
    .unwrap();
    assert!(cleaned.contains("itunes:timing=\"Word\""));
    assert!(cleaned.contains("<p begin=\"1.000\" end=\"1.500\">"));
    assert!(cleaned.contains("<span begin=\"1.200\" end=\"1.500\">好</span>"));
    assert!(qrc_ttml_filtered("encrypted-unavailable", None, None, None).is_none());
    assert_eq!(
        field_text(&json!({"lrc":{"lyric":"[00:01]text"}}), "lrc"),
        Some("[00:01]text")
    );
    assert_eq!(field_text(&json!({"lrc":"[00:01]text"}), "qrc"), None);
}

#[test]
fn qrc_parser_matches_amll_word_and_background_semantics() {
    let source = concat!(
        "[94530,1440]I’ve (94530,30)been (94560,180)thinking (94740,570)of (95310,180)you((95490,30)滋(95520,60)生(95580,60)着(95640,60)那(95700,60)些(95760,60)疯(95820,60)狂(95880,60))(95940,30)\n",
        "[41310,1740]You’re (41310,60)the (41370,120)masterpiece((41490,30)你(41520,210)会(41730,90)是(41820,120)落(41940,60)笔(42030,330)的(42360,60)杰(42420,60)作(42480,60))(42540,510)\n",
        "[27270,1500]Like (27270,330)that (27600,390)I’m (27990,60)back (28050,390)again (28440,300)(again)(28740,30)\n",
        "[2385,2385](你(2385,216)我(2601,216), (2817,216)却(3033,216)不(3249,216)止(3465,216)你(3681,216)我(3897,216))(4113,216)"
    );
    let lines = qrc_lines(source);
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].words[4].text, "you(");
    assert_eq!(lines[0].words[4].start, 95490);
    assert_eq!(lines[1].words[2].text, "masterpiece(");
    assert_eq!(lines[1].words[2].start, 41490);
    assert_eq!(lines[2].words[5].text, "(again)");
    assert_eq!(lines[2].words[5].start, 28740);

    let background = &lines[3];
    assert!(background.is_bg);
    assert_eq!(background.words.first().unwrap().text, "你");
    assert_eq!(background.words.last().unwrap().text, "");
    assert_eq!(background.text, "你我, 却不止你我");
}

#[test]
fn qrc_parser_drops_invalid_rows_and_keeps_parenthesized_fragments() {
    let lines = qrc_lines(
        "no timestamp\r\n[invalid,100]Bad(0,100)\r\n[-1,100]Bad(0,100)\n[0,20]   \n[0,20]\n[0,20]A(x)B(0,10) C(10,10)",
    );
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0]
            .words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["A(x)B", " C"]
    );
    assert_eq!(lines[0].words[0].start, 0);
    assert_eq!(lines[0].words[1].end, 20);
}

#[test]
fn qrc_background_rows_attach_with_nested_roles_or_stand_alone() {
    let rendered = qrc_ttml_filtered(
        "[0,100]main(0,100)\n[1000,1000](bg(1000,500))(1500,500)",
        Some("[00:01.000]bg translation"),
        Some("[1000,100]roman(1000,100)"),
        None,
    )
    .unwrap();
    assert_eq!(rendered.matches("<p ").count(), 1);
    assert!(rendered.contains(
        "<span ttm:role=\"x-bg\" begin=\"1.000\" end=\"2.000\"><span begin=\"1.000\" end=\"1.500\">bg</span><span begin=\"1.500\" end=\"2.000\"></span><span ttm:role=\"x-translation\">bg translation</span><span ttm:role=\"x-roman\">roman</span></span>"
    ));

    let standalone = qrc_ttml_filtered(
        "[1000,500](bg(1000,200))(1200,300)",
        Some("[00:01.000]standalone translation"),
        Some("[1000,100]standalone roman(1000,100)"),
        None,
    )
    .unwrap();
    assert_eq!(standalone.matches("<p ").count(), 1);
    assert!(!standalone.contains("ttm:role=\"x-bg\""));
    assert!(standalone.contains("<p begin=\"1.000\" end=\"1.500\">"));
    assert!(standalone.contains(
        "<span ttm:role=\"x-translation\">standalone translation</span>"
    ));
    assert!(standalone.contains("<span ttm:role=\"x-roman\">standalone roman</span>"));

    let fullwidth = qrc_ttml_filtered(
        "[2000,1000]（bg(2000,500)）（2500,500）",
        Some("[00:02.000]translation"),
        Some("[2000,100]roman(2000,100)"),
        None,
    )
    .unwrap();
    assert!(fullwidth.contains("x-translation"));
    assert!(fullwidth.contains("x-roman"));
}

#[test]
fn qrc_content_preserves_unescaped_quotes_until_the_element_delimiter() {
    let content =
        r#"[1113,753]词：J.Y. (1113,236)Park "(1349,180)The (1529,169)Asiansoul"(1698,168)"#;
    let envelope = format!(
        r#"<QrcInfos><Lyric_1 LyricType="1" LyricContent="{content}"/><Lyric_2 LyricContent="unused"/></QrcInfos>"#
    );
    assert_eq!(qrc_content(&envelope), content);
}

#[test]
fn qrc_content_handles_multiline_values_whitespace_and_entities() {
    let envelope = "<QrcInfos>\n<Lyric_1 LyricContent \t= \"[1000,500]A &quot;B&quot;(1000,500)\r\n[2000,500]C(2000,500)&#13;&#10;[3000,500]&lt;&gt;&apos;&amp;&amp;quot;(3000,500)\" \t\n/>\n</QrcInfos>";
    assert_eq!(
        qrc_content(envelope),
        "[1000,500]A \"B\"(1000,500)\r\n[2000,500]C(2000,500)\r\n[3000,500]<>'&&quot;(3000,500)"
    );
}

#[test]
fn qrc_content_handles_single_quoted_envelopes_with_unescaped_apostrophes() {
    let content = "[1000,500]Don't '(1000,500)\n[2000,500]stop'(2000,500)";
    for delimiter in ["'/>", "' \n/>"] {
        let envelope = format!("<Lyric_1 LyricContent='{content}{delimiter}");
        assert_eq!(qrc_content(&envelope), content);
    }
}

#[test]
fn qrc_content_keeps_plain_text_compatible() {
    let content = "[1000,500]Park \"(1000,500)&#10;[2000,500]&amp;(2000,500)";
    assert_eq!(
        qrc_content(content),
        "[1000,500]Park \"(1000,500)\n[2000,500]&(2000,500)"
    );
    assert_eq!(
        qrc_content("encrypted-unavailable"),
        "encrypted-unavailable"
    );
}

#[test]
fn qrc_content_keeps_available_content_when_the_envelope_is_clipped() {
    let content = "[1000,500]Park \"(1000,500)\n[2000,500]Don't(2000,500)unfinished";
    for quote in ['"', '\''] {
        let envelope = format!("<Lyric_1 LyricContent={quote}{content}");
        assert_eq!(qrc_content(&envelope), content);
    }
}

fn synthetic_wet_lyrics() -> Value {
    // Reduced /lyric/new response for WET (001ATFiY3pb3y6): Korean QRC,
    // Chinese LRC translation and romanized QRC XML. Text/times are synthetic.
    serde_json::from_str(include_str!("../fixtures/qq_lyric_new_wet.json")).unwrap()
}

#[test]
fn auxiliary_lines_prefers_word_text_and_filters_empty_or_placeholder_content() {
    let lines = [
        (1000, "", vec!["oneul ", "bam & <neo>"]),
        (2000, "fallback & text", vec![]),
        (3000, "stale text", vec![" ", "\t"]),
        (4000, "stale text", vec![" /", "/ "]),
        (5000, " // ", vec![]),
        (6000, "\t", vec![]),
        (7000, "stale text", vec!["actual ", "words"]),
    ]
    .into_iter()
    .map(|(start, text, words)| LyricLine {
        start,
        end: start + 500,
        text: text.into(),
        is_bg: false,
        words: words
            .into_iter()
            .map(|text| LyricWord {
                start,
                end: start + 500,
                text: text.into(),
            })
            .collect(),
    })
    .collect();
    assert_eq!(
        auxiliary_lines(lines)
            .into_iter()
            .map(|line| (line.start, line.text))
            .collect::<Vec<_>>(),
        vec![
            (1000, "oneul bam & <neo>".to_owned()),
            (2000, "fallback & text".to_owned()),
            (7000, "actual words".to_owned()),
        ]
    );
}

#[test]
fn auxiliary_matching_keeps_competing_rows_in_order() {
    let main = vec![
        test_line(1000, 1200, "first"),
        test_line(2000, 2200, "second"),
    ];
    let aligned = align_auxiliary(
        &main,
        vec![
            test_line(1050, 1100, "near first"),
            test_line(1100, 1150, "far first"),
        ],
    );
    assert_eq!(aligned[0].as_deref(), Some("near first"));
    assert!(aligned[1].is_none());
}

#[test]
fn auxiliary_matching_preserves_order_under_uniform_offset() {
    let main = vec![test_line(0, 500, "first"), test_line(600, 1100, "second")];
    let aligned = align_auxiliary(
        &main,
        vec![
            test_line(350, 500, "A"),
            test_line(950, 1100, "B"),
        ],
    );
    assert_eq!(aligned[0].as_deref(), Some("A"));
    assert_eq!(aligned[1].as_deref(), Some("B"));
}

#[test]
fn auxiliary_matching_does_not_move_a_translation_to_a_missing_row() {
    let main = vec![
        test_line(0, 500, "first"),
        test_line(600, 1100, "missing"),
        test_line(1200, 1700, "third"),
    ];
    let aligned = align_auxiliary(
        &main,
        vec![test_line(350, 500, "only first"), test_line(1550, 1700, "third")],
    );
    assert_eq!(aligned[0].as_deref(), Some("only first"));
    assert!(aligned[1].is_none());
    assert_eq!(aligned[2].as_deref(), Some("third"));
}

#[test]
fn auxiliary_matching_dp_enforces_distance_and_skip_boundaries() {
    // The hard limit uses the line-start anchors. At exactly 1000 ms the
    // pair is allowed, and the auxiliary timestamp at the main line's end
    // has zero match cost because it falls inside that timed interval.
    let at_max_distance = align_auxiliary(
        &[test_line(1000, 2000, "main")],
        vec![test_line(2000, 2100, "at max distance")],
    );
    assert_eq!(at_max_distance[0].as_deref(), Some("at max distance"));

    let over_max_distance = align_auxiliary(
        &[test_line(1000, 2000, "main")],
        vec![test_line(2001, 2101, "over max distance")],
    );
    assert!(over_max_distance[0].is_none());

    // With one isolated pair, matching wins while its distance is below the
    // 700 ms skip penalty and loses once the distance is above it.
    let below_skip_penalty = align_auxiliary(
        &[test_line(1000, 1000, "main")],
        vec![test_line(1699, 1799, "below skip penalty")],
    );
    assert_eq!(below_skip_penalty[0].as_deref(), Some("below skip penalty"));

    let above_skip_penalty = align_auxiliary(
        &[test_line(1000, 1000, "main")],
        vec![test_line(1701, 1801, "above skip penalty")],
    );
    assert!(above_skip_penalty[0].is_none());
}

#[test]
fn auxiliary_matching_breaks_equidistant_ties_by_earlier_main_row() {
    let main = vec![test_line(1000, 1200, "first"), test_line(2000, 2200, "second")];
    let aligned = align_auxiliary(&main, vec![test_line(1500, 1600, "tie")]);
    assert_eq!(aligned[0].as_deref(), Some("tie"));
    assert!(aligned[1].is_none());
}

#[test]
fn auxiliary_matching_prioritizes_a_main_row_over_a_bg_row() {
    let mut bg = test_line(1100, 1300, "background");
    bg.is_bg = true;
    let main = vec![test_line(1000, 1200, "main"), bg];
    let aligned = align_auxiliary(&main, vec![test_line(1100, 1200, "translation")]);
    assert_eq!(aligned[0].as_deref(), Some("translation"));
    assert!(aligned[1].is_none());
}

#[test]
fn auxiliary_roles_each_attach_to_the_nearest_available_line() {
    let text = qrc_ttml_filtered(
        "[1090,100]first(1090,100)\n[1000,100]closer(1000,100)\n[1090,100]duplicate(1090,100)",
        Some("[00:01.001]translation & <text>"),
        Some("[1001,100]ro(1001,50)man & <text>(1051,50)"),
        None,
    )
    .unwrap();
    let rows = lyric_rows(&text);
    assert_eq!(rows.len(), 3);
    assert!(rows[0].starts_with("<p begin=\"1.090\""));
    assert!(!rows[0].contains("ttm:role="));
    assert!(rows[1].starts_with("<p begin=\"1.000\""));
    assert!(rows[1].ends_with(
        "</span><span ttm:role=\"x-translation\">translation &amp; &lt;text&gt;</span><span ttm:role=\"x-roman\">roman &amp; &lt;text&gt;</span></p>"
    ));
    assert!(!rows[2].contains("ttm:role="));
}

#[test]
fn synthetic_wet_lyric_new_aligns_translation_and_romanization_in_amll_inline_format() {
    let raw = synthetic_wet_lyrics();
    for raw in [&raw, &raw["data"]] {
        let candidates = converted_lyrics_with_context(raw, None);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].source, "qrc");
        assert_eq!(candidates[1].source, "lrc");
        let text = &candidates[0].text;
        assert!(text.contains("xmlns:ttm=\"http://www.w3.org/ns/ttml#metadata\""));
        assert!(text.contains("itunes:timing=\"Word\""));
        let rows = lyric_rows(text);
        assert_eq!(rows.len(), 4, "auxiliary rows must not create lyric lines");
        // These roles must be direct <p> children, after the timed main spans,
        // per src/ui/lyrics/ttml.mjs readVoice/inline (also tests/lyrics_ui.cjs).
        for (row, start, translation, roman) in [
            (rows[0], "1.000", "今晚", "oneul bam"),
            (
                rows[2],
                "3.000",
                "光 &amp; &lt;你&gt;",
                "bit &amp; &lt;neo&gt;",
            ),
            (rows[3], "4.500", "一起", "hamkke"),
        ] {
            assert!(row.starts_with(&format!("<p begin=\"{start}\"")));
            assert!(row.ends_with(&format!(
                "</span><span ttm:role=\"x-translation\">{translation}</span><span ttm:role=\"x-roman\">{roman}</span></p>"
            )), "{row}");
        }
        assert_eq!(
            rows[1],
            "<p begin=\"2.000\" end=\"2.500\"><span begin=\"2.000\" end=\"2.500\">WET</span></p>"
        );
        assert!(rows[0].contains("<span begin=\"1.300\" end=\"1.600\">밤</span>"));
        assert_eq!(text.matches("ttm:role=\"x-translation\"").count(), 3);
        assert_eq!(text.matches("ttm:role=\"x-roman\"").count(), 3);
        for unused in [
            "> // </span>",
            "unmatched",
            "&amp;amp;",
            "QrcInfos",
            "LyricContent",
        ] {
            assert!(!text.contains(unused), "{unused}");
        }
    }
}

#[test]
fn real_wet_lyric_new_preserves_lines_and_auxiliary_spans_after_unescaped_quotes() {
    let raw: Value =
        serde_json::from_str(include_str!("../fixtures/qq_lyric_new_wet_real.json")).unwrap();
    let candidates = converted_lyrics_with_context(&raw, None);
    assert_eq!(candidates.len(), 2);
    let text = &candidates
        .iter()
        .find(|lyric| lyric.source == "qrc")
        .unwrap()
        .text;
    let rows = lyric_rows(text);
    // The real response fields were clipped at 2600 characters, without closing
    // the XML envelopes. Three leading production rows are filtered; the title
    // remains because this test deliberately omits song metadata context.
    assert_eq!(rows.len(), 25);
    assert!(text.contains("itunes:timing=\"Word\""));
    assert!(rows[0].starts_with("<p begin=\"0.391\""));
    assert!(rows[0].contains(">WET - </span>"));
    assert!(rows[0].contains("J.Y."));
    assert!(rows[24].starts_with("<p begin=\"57.987\" end=\"60.792\">"));
    assert!(rows[24].contains("<span begin=\"59.650\" end=\"60.202\">wet </span>"));
    assert!(!rows[24].contains(">we<"));
    // The tolerance now recovers all 23 translations for these clipped main rows;
    // the eight nonblank romanization rows and all T6 timing checks remain.
    assert_eq!(
        text.matches("<span ttm:role=\"x-translation\">").count(),
        23
    );
    assert_eq!(text.matches("<span ttm:role=\"x-roman\">").count(), 8);
    assert!(rows[5]
        .ends_with("</span><span ttm:role=\"x-translation\">如汽水那般令人充满活力</span></p>"));
    assert!(rows[2].ends_with(
        "</span><span ttm:role=\"x-roman\">ri deu mi  neol  um ji gi myeon </span></p>"
    ));
    assert!(!text.contains("LyricContent"));
    assert!(!text.contains("QrcInfos"));
}

#[test]
fn full_wet_lyric_new_aligns_available_auxiliary_text_without_changing_lrc() {
    let raw: Value =
        serde_json::from_str(include_str!("../fixtures/qq_lyric_new_wet_full.json")).unwrap();
    let artists = vec!["J.Y. Park".to_owned()];
    let context = LyricContext {
        name: "WET",
        artists: &artists,
    };
    let candidates = converted_lyrics_with_context(&raw, Some(context));
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].source, "qrc");
    assert_eq!(candidates[1].source, "lrc");
    let text = &candidates[0].text;
    let rows = lyric_rows(text);
    // The original QRC has 59 main rows. Ht removes the title head and ln
    // removes the three 词/曲/编曲 rows; the 54 translated rows remain because
    // those four production rows carry only // placeholders.
    assert_eq!(rows.len(), 55);
    assert!(text.contains("itunes:timing=\"Word\""));
    assert_eq!(
        text.matches("<span ttm:role=\"x-translation\">").count(),
        54
    );
    for filtered_start in ["0.391", "1.113", "1.866", "3.705"] {
        assert!(
            !text.contains(&format!("<p begin=\"{filtered_start}")),
            "filtered QRC production/title row at {filtered_start}"
        );
    }
    // The supplied full fixture has 16 nonblank qrcroma lines; the other 43
    // contain only whitespace words and must not produce empty role spans.
    assert_eq!(text.matches("<span ttm:role=\"x-roman\">").count(), 16);
    assert!(rows[1].starts_with("<p begin=\"9.329\" end=\"11.289\">"));
    assert!(rows[1].ends_with(
        "</span><span ttm:role=\"x-translation\">若节奏让你舞动起来</span><span ttm:role=\"x-roman\">ri deu mi  neol  um ji gi myeon </span></p>"
    ));
    assert!(rows[54].starts_with("<p begin=\"150.233\" end=\"151.740\">"));
    assert!(rows[54].ends_with(
        "<span begin=\"151.309\" end=\"151.740\">sweat</span><span ttm:role=\"x-translation\">亲爱的 尽管让我挥洒汗水吧</span></p>"
    ));
    assert!(rows[0].contains(">Yeah</span>"));
    for row in &rows[..1] {
        assert!(!row.contains("ttm:role="), "{row}");
    }

    let lrc = &candidates[1].text;
    assert!(lrc.contains("itunes:timing=\"Line\""));
    assert_eq!(lrc.matches("<p ").count(), 55);
    assert_eq!(lrc.matches("<span ttm:role=\"x-translation\">").count(), 54);
    for filtered_start in ["0.390", "1.110", "1.860", "3.700"] {
        assert!(
            !lrc.contains(&format!("<p begin=\"{filtered_start}")),
            "filtered LRC production/title row at {filtered_start}"
        );
    }
    assert!(!lrc.contains("x-roman"));
    assert!(lrc.contains(
        "<p begin=\"9.320\" end=\"11.280\">리듬이 널 움직이면<span ttm:role=\"x-translation\">若节奏让你舞动起来</span></p>"
    ));
    assert!(lrc.contains(
        "<p begin=\"150.230\" end=\"158.230\">Baby come and make me sweat<span ttm:role=\"x-translation\">亲爱的 尽管让我挥洒汗水吧</span></p>"
    ));
    assert!(!text.contains("LyricContent"));
    assert!(!text.contains("QrcInfos"));
}

#[test]
fn puyi_lyric_new_attaches_all_translations_without_crossing_rows() {
    let raw: Value =
        serde_json::from_str(include_str!("../fixtures/qq_lyric_new_puyi.json")).unwrap();
    let candidate = converted_lyrics_with_context(&raw, None)
        .into_iter()
        .find(|lyric| lyric.source == "qrc")
        .unwrap();
    assert_eq!(candidate.text.matches("ttm:role=\"x-translation\"").count(), 36);
    for (start, translation) in [
        ("94.530", "滋生着那些疯狂"),
        ("29.790", "回到你带我进入的梦"),
        ("92.010", "所有的想象"),
    ] {
        let row = candidate
            .text
            .split("<p ")
            .find(|row| row.starts_with(&format!("begin=\"{start}\"")))
            .unwrap_or_else(|| panic!("missing row {start}"));
        assert!(row.contains(&format!("x-translation\">{translation}</span>")), "{row}");
    }
}

fn test_line(start: u64, end: u64, text: &str) -> LyricLine {
    LyricLine {
        start,
        end,
        text: text.to_owned(),
        words: Vec::new(),
        is_bg: false,
    }
}

#[test]
fn info_filter_uses_role_segmentation_and_preserves_normal_colons() {
    for label in [
        "音乐总监：",    // en + nn
        "制作人/编曲：", // en + nn, then a second segment
        "总监制：",      // tn + en
        "Lyricist & Producer:",
    ] {
        assert!(is_role_label(label), "{label}");
    }
    assert!(!is_role_label("我说：这是正文"));

    let artists = vec!["J.Y. Park".to_owned()];
    let context = LyricContext {
        name: "WET",
        artists: &artists,
    };
    let kept = filter_info_lines(
        vec![
            test_line(100, 200, "WET - J.Y. Park"),
            test_line(300, 400, "词：J.Y. Park"),
            test_line(500, 600, "我说：今天天气很好"),
            test_line(700, 800, "正文 - 仍然是歌词"),
        ],
        Some(context),
    );
    assert_eq!(
        kept.iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>(),
        ["我说：今天天气很好", "正文 - 仍然是歌词"]
    );
}

#[test]
fn info_filter_applies_other_rules_and_pure_music_tracks() {
    let artists = vec!["歌手".to_owned()];
    let context = LyricContext {
        name: "歌曲",
        artists: &artists,
    };
    let zero = LyricLine {
        start: 0,
        end: 0,
        text: "占位".into(),
        words: vec![LyricWord {
            start: 0,
            end: 0,
            text: "占位".into(),
        }],
        is_bg: false,
    };
    let kept = filter_info_lines(
        vec![
            test_line(100, 200, "未经许可不得使用"),
            test_line(300, 400, "酷狗音乐人计划"),
            test_line(500, 600, "Written by: someone"),
            test_line(700, 800, "歌手："),
            test_line(900, 1000, "器乐团队：某某"),
            zero,
            test_line(1100, 1200, "我说：正文"),
        ],
        Some(context),
    );
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].text, "我说：正文");

    let raw = json!({
        "qrc": "[0,100]纯音乐，请欣赏(0,0)",
        "lrc": "[00:00.00]纯音乐，请欣赏",
    });
    assert!(converted_lyrics_with_context(&raw, None).is_empty());
}

#[test]
fn lyric_new_handles_each_missing_or_invalid_auxiliary_track_independently() {
    for translation in [false, true] {
        for roman in [false, true] {
            let mut raw = synthetic_wet_lyrics();
            let data = raw["data"].as_object_mut().unwrap();
            if !translation {
                data.remove("qrctrans");
            }
            if !roman {
                data.remove("qrcroma");
            }
            let text = &converted_lyrics_with_context(&raw, None)[0].text;
            assert_eq!(text.matches("<p ").count(), 4);
            assert_eq!(
                text.matches("ttm:role=\"x-translation\"").count(),
                if translation { 3 } else { 0 }
            );
            assert_eq!(
                text.matches("ttm:role=\"x-roman\"").count(),
                if roman { 3 } else { 0 }
            );
        }
    }
    for invalid in [Value::Null, json!(""), json!({}), json!("unavailable")] {
        let mut raw = synthetic_wet_lyrics();
        for field in ["qrctrans", "qrcroma", "lrctrans"] {
            raw["data"][field] = invalid.clone();
        }
        let candidates = converted_lyrics_with_context(&raw, None);
        assert_eq!(candidates.len(), 2);
        for candidate in candidates {
            assert!(candidate.text.contains("<p "));
            assert!(!candidate.text.contains("ttm:role="));
        }
    }
    // Auxiliary tracks alone cannot manufacture a main candidate.
    let mut raw = synthetic_wet_lyrics();
    raw["data"]["qrc"] = json!("encrypted-unavailable");
    raw["data"]["lrc"] = Value::Null;
    assert!(converted_lyrics_with_context(&raw, None).is_empty());
}

#[test]
fn lrc_candidate_uses_lrctrans_with_offsets_and_repeated_timestamps() {
    let candidates = converted_lyrics_with_context(&synthetic_wet_lyrics(), None);
    let text = &candidates[1].text;
    assert!(text.contains("itunes:timing=\"Line\""));
    assert_eq!(text.matches("<p ").count(), 2);
    for (start, end) in [("1.000", "3.000"), ("3.000", "11.000")] {
        assert!(text.contains(&format!(
            "<p begin=\"{start}\" end=\"{end}\">오늘 밤<span ttm:role=\"x-translation\">LRC 翻译</span></p>"
        )));
    }
    assert!(!text.contains("x-roman"));
    assert!(!text.contains("unmatched"));
}

fn state(enabled: bool) -> Arc<AppState> {
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = enabled.then(|| "https://example.invalid/api/web".into());
    Arc::new(AppState::with_config(config))
}

async fn serve_local(app: Router) -> Option<(String, tokio::task::JoinHandle<()>)> {
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
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Some((url, task))
}
async fn get(state: Arc<AppState>, uri: &str) -> Response {
    crate::router(state)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap()
}

fn lyric_rows(text: &str) -> Vec<&str> {
    Regex::new(r"<p .*?</p>").unwrap().find_iter(text).map(|row| row.as_str()).collect()
}

#[tokio::test]
async fn proxy_is_not_exposed_in_capabilities_or_host_config() {
    let cli = cli(&[
        "fantastic",
        "--qq-api",
        "https://example.invalid/api/web",
        "--qq-proxy",
        "socks5://127.0.0.1:11080",
    ]);
    let state =
        Arc::new(AppState::with_qq_proxy(cli.config().unwrap(), cli.qq_proxy.as_deref()).unwrap());
    for path in ["/qq/capabilities", "/assets/host.js"] {
        let response = get(state.clone(), path).await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(!body.contains("11080"));
        assert!(!body.to_ascii_lowercase().contains("qqproxy"));
        if path == "/qq/capabilities" {
            let value: Value = serde_json::from_str(&body).unwrap();
            assert_eq!(value["enabled"], true);
            let fixture: Value = serde_json::from_str(include_str!("../fixtures/qq_capabilities.json")).unwrap();
            assert_eq!(value, fixture, "Rust capabilities must match the shared fixture");
            let qualities = value["qualities"].as_array().unwrap();
            assert_eq!(qualities.len(), 20);
            assert_eq!(qualities[0]["quality"], "MP3128");
            assert_eq!(qualities[0]["defaultCodec"], "mp3");
            assert_eq!(qualities[0]["contentType"], "audio/mpeg");
            assert_eq!(qualities[0]["playbackClass"], "native");
            assert_eq!(qualities[0]["sizeField"], "size_128mp3");
            assert_eq!(qualities[0]["displayName"], "MP3 128 kbps");
            let ac4 = qualities.iter().find(|row| row["quality"] == "DA").unwrap();
            assert_eq!(ac4["defaultCodec"], "ac-4");
            assert_eq!(ac4["contentType"], "audio/mp4; codecs=\"ac-4\"");
            assert_eq!(ac4["playbackClass"], "ac-4-probe");
            assert_eq!(ac4["sizeField"], "size_dolby");
            let joc = qualities.iter().find(|row| row["quality"] == "DAEAC3").unwrap();
            assert_eq!(joc["defaultCodec"], "ec-3");
            assert_eq!(joc["playbackClass"], "joc");
            assert_eq!(joc["sizeField"], "size_dolby");
        }
    }
}
#[tokio::test]
async fn unconfigured_routes_are_404_even_with_missing_or_invalid_parameters() {
    let state = state(false);
    for path in [
        "/qq/capabilities",
        "/qq/search",
        "/qq/search?q=x",
        "/qq/song/a/play",
        "/qq/song/a/download",
        "/qq/song/a/lyrics",
        "/qq/song/a/lyrics/raw",
        "/qq/song/a/qualities",
        "/qq/cover/003DFRzD192KKD",
        "/qq/cover/a%21",
        "/qq/unknown",
    ] {
        assert_eq!(
            get(state.clone(), path).await.status(),
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
}
#[tokio::test]
async fn cover_route_rejects_invalid_album_mids_before_outbound_requests() {
    let state = state(true);
    for mid in [
        "a%21",
        "%2E%2E%2Fx",
        "a%3Fx=1",
        "https%3A%2F%2Fexample.invalid",
        "%E4%B8%AD",
        &"a".repeat(33),
    ] {
        let path = format!("/qq/cover/{mid}");
        assert_eq!(
            get(state.clone(), &path).await.status(),
            StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
    assert_eq!(
        get(state, "/qq/cover?url=https://example.invalid")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn configured_routes_validate_mid_and_serve_only_cached_dtos() {
    let state = state(true);
    for path in [
        "/qq/song/a%21",
        "/qq/song/a%21/play",
        "/qq/song/a%21/download",
        "/qq/song/a%21/lyrics",
        "/qq/song/a%21/lyrics/raw",
        "/qq/song/a%21/qualities",
    ] {
        assert_eq!(
            get(state.clone(), path).await.status(),
            StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
    let qq = state.qq.as_ref().unwrap();
    let song = qq.song_dto(&raw_song()).unwrap();
    qq.searches.put(
        "30:0:test".into(),
        SearchResult {
            songs: vec![song],
            total: 1,
        },
    );
    qq.lyrics.put(
        "abc".into(),
        Lyrics {
            candidates: vec![Lyric {
                source: "lrc",
                text: lrc_ttml_filtered("[00:01]Text", None, None).unwrap(),
            }],
            raw_qrc: false,
            raw_trans: false,
            raw_roma: false,
            raw: RawLyrics::default(),
        },
    );
    for path in [
        "/qq/search?q=test",
        "/qq/song/0039MnYb0qxYhV",
        "/qq/song/abc/lyrics",
        "/qq/capabilities",
        "/assets/host.js",
    ] {
        let response = get(state.clone(), path).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(!body.contains("example.invalid"), "{path}");
        assert!(!body.contains("file_url"), "{path}");
    }
}

#[tokio::test]
async fn raw_lyric_route_validates_kind_reports_missing_data_and_uses_utf8_filenames() {
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    let song = qq.song_dto(&raw_song()).unwrap();
    let mid = song.id.clone();
    qq.songs.put(mid.clone(), song);
    qq.lyrics.put(
        mid.clone(),
        Lyrics {
            candidates: Vec::new(),
            raw_qrc: true,
            raw_trans: true,
            raw_roma: false,
            raw: RawLyrics {
                qrc: Some("<LyricContent>qrc raw</LyricContent>".into()),
                trans: Some("[00:01.00]translation raw".into()),
                roma: None,
            },
        },
    );
    // filename* is the percent-encoded "周杰伦 - 晴天" plus the kind's suffix.
    const BASE: &str = "%E5%91%A8%E6%9D%B0%E4%BC%A6%20%2D%20%E6%99%B4%E5%A4%A9";
    for (kind, file, suffix, body) in [
        ("qrc", "download.qrc", "%2E%71%72%63", "<LyricContent>qrc raw</LyricContent>"),
        ("qrctrans", "download.lrc", "%2E%74%72%61%6E%73%2E%6C%72%63", "[00:01.00]translation raw"),
    ] {
        let response = get(state.clone(), &format!("/qq/song/{mid}/lyrics/raw?kind={kind}")).await;
        assert_eq!(response.status(), StatusCode::OK, "{kind}");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/plain; charset=utf-8");
        assert_eq!(
            response.headers()[header::CONTENT_DISPOSITION],
            format!("attachment; filename=\"{file}\"; filename*=UTF-8''{BASE}{suffix}").as_str()
        );
        let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
        assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), body, "{kind}");
    }
    let response = get(state.clone(), &format!("/qq/song/{mid}/lyrics/raw?kind=qrcroma")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND, "qrcroma without raw data");
    // Unknown and missing kinds are rejected before serving any data.
    for query in ["", "?kind=bad", "?kind=qrc%2F"] {
        let path = format!("/qq/song/{mid}/lyrics/raw{query}");
        assert_eq!(get(state.clone(), &path).await.status(), StatusCode::BAD_REQUEST, "{query}");
    }
}

#[tokio::test]
async fn raw_lyric_bad_kind_rejects_before_cold_upstream_fetch() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let upstream = Router::new().fallback(any(move || {
        seen.fetch_add(1, Ordering::SeqCst);
        async { StatusCode::NOT_FOUND }
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = Some(base);
    let state = Arc::new(AppState::with_config(config));
    let response = get(state, "/qq/song/cold/lyrics/raw?kind=bad").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(hits.load(Ordering::SeqCst), 0);
    task.abort();
}

#[tokio::test]
async fn playback_redirect_has_no_audio_or_json_body() {
    let url =
        reqwest::Url::parse("https://isure.stream.qqmusic.qq.com/song.m4a?vkey=test").unwrap();
    let response = play_redirect(&url);
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], url.as_str());
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(to_bytes(response.into_body(), 1000)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn download_upstream_non_2xx_returns_502() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let upstream = Router::new().fallback(any(move || {
        seen.fetch_add(1, Ordering::SeqCst);
        async { StatusCode::FORBIDDEN }
    }));
    let Some((base, task)) = serve_local(upstream).await else {
        return;
    };
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    let mid = "downloadtest";
    qq.songs.put(
        mid.into(),
        Song {
            id: mid.into(),
            name: "Song".into(),
            artists: vec!["Artist".into()],
            album: "Album".into(),
            album_mid: "album".into(),
            duration_ms: 1000,
            cover: String::new(),
            vip: false,
            ttml: false,
        },
    );
    qq.audio.put(
        Quality::Hq.cache_key(mid),
        Audio {
            url: reqwest::Url::parse(&format!("{base}/track.mp3")).unwrap(),
            size_bytes: Some(4),
            codec_hint: None,
        },
    );
    let response = get(state, &format!("/qq/song/{mid}/download?quality=HQ")).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    task.abort();
}

#[tokio::test]
async fn download_success_transfers_headers_and_body() {
    let upstream = Router::new().fallback(any(|| async {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "audio/mpeg")
            .header(header::CONTENT_LENGTH, "4")
            .body(Body::from("data"))
            .unwrap()
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    let mid = "downloadok";
    qq.songs.put(mid.into(), Song {
        id: mid.into(), name: "Song".into(), artists: vec!["Artist".into()], album: "Album".into(),
        album_mid: "album".into(), duration_ms: 1000, cover: String::new(), vip: false, ttml: false,
    });
    qq.audio.put(Quality::Hq.cache_key(mid), Audio {
        url: reqwest::Url::parse(&format!("{base}/track.mp3")).unwrap(), size_bytes: Some(4), codec_hint: None,
    });
    let response = get(state, &format!("/qq/song/{mid}/download?quality=HQ")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "audio/mpeg");
    assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "attachment; filename=\"download.mp3\"; filename*=UTF-8''%41%72%74%69%73%74%20%2D%20%53%6F%6E%67%2E%6D%70%33");
    assert_eq!(response.headers()[header::CONTENT_LENGTH], "4");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(to_bytes(response.into_body(), 100).await.unwrap().as_ref(), b"data");
    task.abort();
}

#[tokio::test]
async fn download_redirect_from_upstream_is_hidden_as_502() {
    let upstream = Router::new().fallback(any(|| async {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "https://evil.example.invalid/audio.mp3")
            .body(Body::empty())
            .unwrap()
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    let mid = "downloadredirect";
    qq.songs.put(mid.into(), Song {
        id: mid.into(), name: "Song".into(), artists: vec!["Artist".into()], album: "Album".into(),
        album_mid: "album".into(), duration_ms: 1000, cover: String::new(), vip: false, ttml: false,
    });
    qq.audio.put(Quality::Hq.cache_key(mid), Audio {
        url: reqwest::Url::parse(&format!("{base}/track.mp3")).unwrap(), size_bytes: Some(4), codec_hint: None,
    });
    assert_eq!(get(state, &format!("/qq/song/{mid}/download?quality=HQ")).await.status(), StatusCode::BAD_GATEWAY);
    task.abort();
}

#[tokio::test]
async fn media_redirect_to_non_allowlisted_host_stops_at_302() {
    let upstream = Router::new().fallback(any(|| async {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "https://evil.example.invalid/audio.mp3")
            .body(Body::empty())
            .unwrap()
    }));
    let Some((base, task)) = serve_local(upstream).await else {
        return;
    };
    let response = qq()
        .media
        .get(format!("{base}/start"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response.headers()[header::LOCATION],
        "https://evil.example.invalid/audio.mp3"
    );
    task.abort();
}

#[test]
fn download_filename_uses_metadata_and_keeps_the_cdn_extension() {
    let mut song = qq().song_dto(&raw_song()).unwrap();
    let url = reqwest::Url::parse("https://y.qq.com/test.flac?private=secret").unwrap();
    let value = download_disposition(&song, &url);
    assert!(value.contains("filename=\"download.flac\""));
    assert!(value.contains(
        "filename*=UTF-8''%E5%91%A8%E6%9D%B0%E4%BC%A6%20%2D%20%E6%99%B4%E5%A4%A9%2E%66%6C%61%63"
    ));
    assert!(!value.contains("secret"));
    song.name = "very long name / \r\n".repeat(40);
    assert!(download_disposition(&song, &url).ends_with("%2E%66%6C%61%63"));
    let no_extension = reqwest::Url::parse("https://y.qq.com/audio").unwrap();
    assert!(download_disposition(&song, &no_extension).contains("download.m4a"));
}

#[test]
fn filename_sanitizing_handles_empty_and_windows_reserved_bases() {
    let url = reqwest::Url::parse("https://y.qq.com/audio.flac").unwrap();
    let mut song = qq().song_dto(&raw_song()).unwrap();
    song.id = "mid123".into();
    song.artists.clear();
    song.name.clear();
    let empty = download_disposition(&song, &url);
    assert!(empty.contains("%6D%69%64%31%32%33%2E%66%6C%61%63"));
    song.id = "CON".into();
    let reserved = download_disposition(&song, &url);
    assert!(reserved.contains("%43%4F%4E%5F%2E%66%6C%61%63"));
}
#[test]
fn album_mid_is_recovered_from_public_cover_when_album_id_is_numeric() {
    let mut raw = raw_song();
    raw["al"]["id"] = json!(123);
    raw["al"]["picUrl"] =
        json!("https://y.qq.com/music/photo_new/T002R500x500M000003DFRzD192KKD_5.jpg");
    assert_eq!(qq().song_dto(&raw).unwrap().album_mid, "003DFRzD192KKD");
}

#[test]
fn song_without_an_album_mid_keeps_album_and_cover_empty() {
    let mut raw = raw_song();
    raw["al"] = json!({"name": "叶惠美"});
    let song = qq().song_dto(&raw).unwrap();
    assert_eq!(song.album_mid, "");
    assert_eq!(song.cover, "");
}

#[test]
fn covers_preserve_upstream_urls_and_use_the_existing_qq_domain_as_fallback() {
    let qq = qq();
    let mut raw = raw_song();
    raw["al"]["picUrl"] = json!("https://y.qq.com/cover.jpg?version=5");
    assert_eq!(
        qq.song_dto(&raw).unwrap().cover,
        "https://y.qq.com/cover.jpg?version=5"
    );
    for pic in [Value::Null, json!("https://example.invalid/private.jpg")] {
        raw["al"]["picUrl"] = pic;
        assert_eq!(
            qq.song_dto(&raw).unwrap().cover,
            "https://y.gtimg.cn/music/photo_new/T002R{w}x{h}M000003DFRzD192KKD.jpg"
        );
    }
}

fn audio(quality: Quality, size: Option<u64>, extension: &str) -> Audio {
    Audio {
        url: reqwest::Url::parse(&format!(
            "https://isure.stream.qqmusic.qq.com/{}.{}?vkey=private",
            quality.name(),
            extension
        ))
        .unwrap(),
        size_bytes: size,
        codec_hint: None,
    }
}

#[test]
fn quality_sizes_use_only_positive_bytes_and_verified_mapping() {
    for (quality, field) in [
        (Quality::Mp3128, "size_128mp3"),
        (Quality::Aac48, "size_48aac"),
        (Quality::Web, "size_96aac"),
        (Quality::Aac192, "size_192aac"),
        (Quality::Ogg96, "size_96ogg"),
        (Quality::Ogg192, "size_192ogg"),
        (Quality::Hq, "size_320mp3"),
        (Quality::Sq, "size_flac"),
        (Quality::Rs, "size_hires"),
        (Quality::Dts, "size_dts"),
        (Quality::Da, "size_dolby"),
        (Quality::Daeac3, "size_dolby"),
        (Quality::Ra360, "size_360ra"),
    ] {
        assert_eq!(quality.size_field(), Some(field));
    }
    for quality in [
        Quality::Nac,
        Quality::Qai,
        Quality::Q360v1,
        Quality::Q360v2,
        Quality::Dtsx,
        Quality::Da448,
        Quality::Da1500,
    ] {
        // No unverified size_new slot indices or DTSX field names.
        assert!(quality.size_field().is_none());
    }
    for value in [Value::Null, json!(0), json!(-1), json!(1.5), json!("123")] {
        let row = QualityRow::from_size(Quality::Hq, byte_size(&value), Quality::Hq.format());
        assert!(!row.available);
        assert!(row.size_bytes.is_none());
    }
    let row = QualityRow::from_size(
        Quality::Hq,
        byte_size(&json!(10792943)),
        Quality::Hq.format(),
    );
    assert!(row.available);
    assert_eq!(row.size_bytes, Some(10792943));
    assert_eq!(row.format, "MP3 320 kbps");
    for (value, expected) in [
        (json!([0, 0, 0]), None),
        (json!([0, 123, 0]), Some(123)),
        (json!([123, 123]), Some(123)),
        (json!([123, 456]), None),
        (Value::Null, None),
    ] {
        let file = json!({"size_360ra":value});
        assert_eq!(Quality::Ra360.info_size(&file), expected);
        assert_eq!(
            Quality::Ra360.needs_probe(&file),
            value == json!([123, 456])
        );
    }
}

#[test]
fn quality_whitelist_roundtrips_every_verified_code_in_group_order() {
    let expected = [
        "MP3128", "AAC48", "WEB", "AAC192", "OGG96", "OGG192", "HQ", "NAC", "SQ", "RS", "QAI",
        "Q360V1", "Q360V2", "RA360", "DTS", "DTSX", "DA", "DAEAC3", "DA448", "DA1500",
    ];
    assert_eq!(QUALITY_LIST.len(), expected.len());
    for (quality, name) in QUALITY_LIST.into_iter().zip(expected) {
        assert_eq!(quality.name(), name);
        assert_eq!(serde_json::to_value(quality).unwrap(), json!(name));
        assert_eq!(
            serde_json::from_value::<Quality>(json!(name)).unwrap(),
            quality
        );
        assert!(serde_json::from_value::<Quality>(json!(name.to_ascii_lowercase())).is_err());
    }
    for unknown in ["Q360V3", "Q360_5.1", "Q360_7.1.4", "Q360_3.0", "DA2000"] {
        assert!(serde_json::from_value::<Quality>(json!(unknown)).is_err());
    }
}

#[test]
fn basic_aac_and_ogg_tiers_keep_verified_bitrates_and_native_classification() {
    let file = json!({"size_128mp3":1280,"size_48aac":480,"size_96aac":960,
        "size_192aac":1920,"size_96ogg":961,"size_192ogg":1921});
    #[rustfmt::skip]
    let cases = [
        (Quality::Mp3128, 1280, "mp3", "MP3 128 kbps", "mp3", "audio/mpeg"),
        (Quality::Aac48, 480, "m4a", "AAC 48 kbps", "aac", "audio/mp4; codecs=\"mp4a.40.2\""),
        (Quality::Web, 960, "m4a", "AAC 96 kbps", "aac", "audio/mp4; codecs=\"mp4a.40.2\""),
        (Quality::Aac192, 1920, "AAC", "AAC 192 kbps", "aac", "audio/aac"),
        (Quality::Ogg96, 961, "ogg", "Ogg 96 kbps", "vorbis", "audio/ogg; codecs=\"vorbis\""),
        (Quality::Ogg192, 1921, "ogg", "Ogg 192 kbps", "vorbis", "audio/ogg; codecs=\"vorbis\""),
    ];
    for (quality, size, extension, format, codec, content_type) in cases {
        let source = audio(quality, Some(size), extension);
        let row = QualityRow::from_audio(quality, &file, Some(&source));
        assert_eq!(row.playback, Playback::Native, "{quality:?}");
        assert_eq!(row.codec, codec);
        assert_eq!(row.format, format);
        assert_eq!(row.content_type, content_type);
        let info_row = QualityRow::from_size(quality, quality.info_size(&file), quality.format());
        assert_eq!(info_row.size_bytes, Some(size));
        assert_eq!(info_row.playback, Playback::Native);
        assert_eq!(info_row.format, format);
    }
}

#[test]
fn dolby_tiers_follow_verified_wet_sample_entries() {
    let file = json!({"size_dolby":5112212,"size_new":[8936764,29870162]});
    #[rustfmt::skip]
    let cases = [
        (Quality::Da, 5112212, Codec::Ac4, Playback::Native, "AC-4 Dolby Atmos"),
        (Quality::Da448, 8936764, Codec::Ac4, Playback::Native, "AC-4 A-JOC 448 kbps"),
        (Quality::Da1500, 29870162, Codec::Ac4, Playback::Native, "AC-4 A-JOC 1500 kbps"),
        (Quality::Daeac3, 5112212, Codec::Ec3, Playback::Joc, "E-AC-3 JOC"),
    ];
    for (quality, size, codec, playback, format) in cases {
        let info_row = QualityRow::from_size(quality, Some(size), quality.format());
        assert_eq!(info_row.playback, playback, "{quality:?}");
        assert_eq!(info_row.codec, codec.name());
        assert_eq!(info_row.format, format);
        assert_eq!(QualityRow::from_audio(quality, &file, None).format, format);
        // Generic MP4 URLs, shared size_dolby and conflicting metadata cannot
        // override the fixed pipelines verified by WET's dac4/dec3 boxes.
        for hint in [None, Some(Codec::Ac4), Some(Codec::Ec3), Some(Codec::Flac)] {
            let mut source = audio(quality, Some(size), "mp4");
            source.codec_hint = hint;
            let row = QualityRow::from_audio(quality, &file, Some(&source));
            assert_eq!(row.playback, playback, "{quality:?}");
            assert_eq!(row.codec, codec.name());
            assert_eq!(row.format, format);
            assert_eq!(row.content_type, match codec {
                Codec::Ac4 => "audio/mp4; codecs=\"ac-4\"",
                Codec::Ec3 => "audio/mp4; codecs=\"ec-3\"",
                _ => unreachable!("dolby fixture codec"),
            });
            assert_eq!(row.size_bytes, Some(size));
        }
    }
}

#[test]
fn spatial_flac_is_native_and_undecodable_tiers_remain_download_only() {
    for quality in [Quality::Qai, Quality::Q360v1, Quality::Q360v2] {
        for (extension, expected) in [
            ("flac", Playback::Native),
            ("opus", Playback::Native),
            ("ac4", Playback::Native),
            ("dts", Playback::Download),
            ("mp4", Playback::Download),
        ] {
            let source = audio(quality, Some(123), extension);
            let row = QualityRow::from_audio(quality, &json!({"size_new":[123]}), Some(&source));
            assert_eq!(row.playback, expected);
            if extension == "opus" {
                assert_eq!(row.codec, "opus");
                assert_eq!(row.content_type, "audio/ogg; codecs=\"opus\"");
            }
        }
    }
    for quality in [Quality::Nac, Quality::Dts, Quality::Dtsx, Quality::Ra360] {
        for extension in ["nac", "dts", "dtsx", "mp4", "ac4", "flac", "mp3"] {
            let source = audio(quality, Some(123), extension);
            assert_eq!(
                QualityRow::from_audio(quality, &Value::Null, Some(&source)).playback,
                Playback::Download
            );
        }
    }
}

#[test]
fn probe_sizes_cross_check_file_fields_before_describing_the_format() {
    let file = json!({
        "size_128mp3": 4317292, "size_320mp3": 10792943,
        "size_flac": 55397039, "size_192aac": 6519764, "size_hires": 90000000
    });
    for (quality, size, extension, expected) in [
        (Quality::Sq, 55397039, "flac", "FLAC"),
        (Quality::Web, 6519764, "m4a", "AAC 192 kbps"),
        (Quality::Rs, 90000000, "flac", "FLAC Hi-Res"),
        (Quality::Nac, 2222222, "nac", "AICodec"),
    ] {
        let source = audio(quality, Some(size), extension);
        let row = QualityRow::from_audio(quality, &file, Some(&source));
        assert!(row.available);
        assert_eq!(row.size_bytes, Some(size));
        assert_eq!(row.format, expected);
    }
    // A verified default cannot override an actual MP3 payload returned for WEB.
    let source = audio(Quality::Web, Some(4317292), "mp3");
    assert_eq!(source.format(Quality::Web, &file), "MP3 128 kbps");
    // Equal-size fields are ambiguous: only the actual extension is described.
    assert_eq!(
        source.format(
            Quality::Web,
            &json!({"size_128mp3":4317292,"size_192aac":4317292})
        ),
        "MP3"
    );
    let source = audio(Quality::Sq, None, "flac");
    assert!(!QualityRow::from_audio(Quality::Sq, &file, Some(&source)).available);
    assert!(!QualityRow::from_audio(Quality::Sq, &file, None).available);
}

#[test]
fn audio_response_requires_a_public_qq_url_and_treats_missing_sizes_as_unavailable() {
    let valid = "https://isure.stream.qqmusic.qq.com/song.mp3?vkey=private";
    let source =
        Audio::from_response(&json!([{"track_info":{"file_url":valid,"file_size":10792943}}]))
            .ok()
            .unwrap();
    assert_eq!(source.size_bytes, Some(10792943));
    for track in [
        json!({"file_url":valid}),
        json!({"file_url":valid,"file_size":0}),
    ] {
        assert!(Audio::from_response(&json!([{"track_info":track}]))
            .ok()
            .unwrap()
            .size_bytes
            .is_none());
    }
    for data in [
        json!([]),
        json!([{"track_info":{"file_url":"https://example.invalid/private"}}]),
    ] {
        assert!(Audio::from_response(&data).is_err());
    }
}

#[test]
fn explicit_codec_metadata_precedes_container_and_size_and_stays_allowlisted() {
    for key in ["codec", "codecs", "audio_codec"] {
        for (name, codec, playback) in [
            ("ac4", "ac-4", Playback::Native),
            ("ac-4", "ac-4", Playback::Native),
            ("eac3", "ec-3", Playback::Joc),
            ("flac", "flac", Playback::Native),
        ] {
            let source = Audio::from_response(&json!([{"track_info":{
                "file_url":"https://isure.stream.qqmusic.qq.com/DA.mp4?vkey=private",
                "file_size":5112212, key:name
            }}]))
            .ok()
            .unwrap();
            let row = QualityRow::from_audio(
                Quality::Q360v1,
                &json!({"size_dolby":5112212}),
                Some(&source),
            );
            assert_eq!(row.codec, codec);
            assert_eq!(row.playback, playback);
        }
    }
    let source = Audio::from_response(&json!([{"track_info":{
        "file_url":"https://isure.stream.qqmusic.qq.com/file.bin?vkey=private",
        "file_size":123, "codec":"private upstream metadata"
    }}]))
    .ok()
    .unwrap();
    let row = QualityRow::from_audio(Quality::Q360v1, &Value::Null, Some(&source));
    assert_eq!(row.playback, Playback::Download);
    assert_eq!(row.codec, "unknown");
    assert!(!serde_json::to_string(&row).unwrap().contains("private"));
}

#[test]
fn download_content_types_follow_an_allowlisted_file_extension() {
    for (extension, expected) in [
        ("mp3", "audio/mpeg"),
        ("FLAC", "audio/flac"),
        ("m4a", "audio/mp4"),
        ("aac", "audio/aac"),
        ("ogg", "audio/ogg"),
        ("ape", "audio/x-ape"),
        ("nac", "application/octet-stream"),
        ("html", "application/octet-stream"),
    ] {
        let url =
            reqwest::Url::parse(&format!("https://y.qq.com/song.{extension}?x=.html")).unwrap();
        assert_eq!(download_content_type(&url), expected);
    }
}

#[tokio::test]
async fn quality_parameters_are_validated_before_upstream_requests() {
    let state = state(true);
    for action in ["play", "download"] {
        for quality in ["", "hq", "invalid", "HQ%20", "HQ%26SQ", "null"] {
            let path = format!("/qq/song/abc/{action}?quality={quality}");
            assert_eq!(
                get(state.clone(), &path).await.status(),
                StatusCode::BAD_REQUEST,
                "{path}"
            );
        }
    }
}

#[tokio::test]
async fn playback_defaults_to_hq_and_caches_are_isolated_by_quality() {
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    for quality in QUALITY_LIST {
        qq.audio
            .put(quality.cache_key("abc"), audio(quality, Some(123), "mp3"));
    }
    assert!(qq.audio.get(&Quality::Hq.cache_key("other")).is_none());
    for quality in QUALITY_LIST {
        let path = format!("/qq/song/abc/play?quality={}", quality.name());
        let response = get(state.clone(), &path).await;
        assert_eq!(response.status(), StatusCode::FOUND, "{path}");
        assert_eq!(
            response.headers()[header::LOCATION],
            audio(quality, Some(123), "mp3").url.as_str()
        );
    }
    let response = get(state.clone(), "/qq/song/abc/play").await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response.headers()[header::LOCATION],
        audio(Quality::Hq, Some(123), "mp3").url.as_str()
    );
}

#[tokio::test]
async fn qualities_endpoint_returns_all_tiers_and_uses_shared_cached_probes() {
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    qq.infos.put(
        "abc".into(),
        json!({"abc":{"track_info":{"mid":"abc","file":{
            "size_128mp3":4317292,"size_48aac":1600000,"size_96aac":3259882,
            "size_192aac":6519764,"size_96ogg":3200000,"size_192ogg":6400000,
            "size_320mp3":10792943,"size_flac":55397039,"size_hires":0,
            "size_dts":0,"size_360ra":[0,0,0],"size_dolby":5112212,
            "size_new":[58805051,80437198,186980254,8936764,29870162]
        }}}}),
    );
    for (quality, size, extension) in [
        (Quality::Web, Some(3259882), "m4a"),
        (Quality::Nac, None, "nac"),
        (Quality::Q360v1, Some(58805051), "flac"),
        (Quality::Q360v2, Some(80437198), "flac"),
        (Quality::Qai, Some(186980254), "flac"),
        (Quality::Dtsx, None, "dtsx"),
        (Quality::Da448, Some(8936764), "mp4"),
        (Quality::Da1500, Some(29870162), "mp4"),
    ] {
        qq.audio
            .put(quality.cache_key("abc"), audio(quality, size, extension));
    }
    // Verified fields work without a cached or live audio URL.
    assert!(qq.audio.get(&Quality::Hq.cache_key("abc")).is_none());
    assert!(qq.audio.get(&Quality::Da.cache_key("abc")).is_none());
    let response = get(state.clone(), "/qq/song/abc/qualities").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let rows = body["qualities"].as_array().unwrap();
    assert_eq!(rows.len(), 20);
    assert_eq!(
        rows.iter()
            .map(|row| row["quality"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "MP3128", "AAC48", "WEB", "AAC192", "OGG96", "OGG192", "HQ", "NAC", "SQ", "RS", "QAI",
            "Q360V1", "Q360V2", "RA360", "DTS", "DTSX", "DA", "DAEAC3", "DA448", "DA1500"
        ]
    );
    #[rustfmt::skip]
    let cases = [
        ("MP3128", true, Some(4317292), "MP3 128 kbps", "native", "mp3", "audio/mpeg"),
        ("AAC48", true, Some(1600000), "AAC 48 kbps", "native", "aac", "audio/mp4; codecs=\"mp4a.40.2\""),
        ("WEB", true, Some(3259882), "AAC 96 kbps", "native", "aac", "audio/mp4; codecs=\"mp4a.40.2\""),
        ("AAC192", true, Some(6519764), "AAC 192 kbps", "native", "aac", "audio/mp4; codecs=\"mp4a.40.2\""),
        ("OGG96", true, Some(3200000), "Ogg 96 kbps", "native", "vorbis", "audio/ogg; codecs=\"vorbis\""),
        ("OGG192", true, Some(6400000), "Ogg 192 kbps", "native", "vorbis", "audio/ogg; codecs=\"vorbis\""),
        ("HQ", true, Some(10792943), "MP3 320 kbps", "native", "mp3", "audio/mpeg"),
        ("NAC", false, None, "AICodec", "download", "nac", "audio/x-nac"),
        ("SQ", true, Some(55397039), "FLAC", "native", "flac", "audio/flac"),
        ("RS", false, None, "FLAC Hi-Res", "native", "flac", "audio/flac"),
        ("QAI", true, Some(186980254), "FLAC", "native", "flac", "audio/flac"),
        ("Q360V1", true, Some(58805051), "FLAC", "native", "flac", "audio/flac"),
        ("Q360V2", true, Some(80437198), "FLAC", "native", "flac", "audio/flac"),
        ("RA360", false, None, "360 Reality Audio", "download", "unknown", "application/octet-stream"),
        ("DTS", false, None, "DTS", "download", "dts", "audio/vnd.dts"),
        ("DTSX", false, None, "DTS:X", "download", "dtsx", "audio/vnd.dts.hd; codecs=\"dtsx\""),
        ("DA", true, Some(5112212), "AC-4 Dolby Atmos", "native", "ac-4", "audio/mp4; codecs=\"ac-4\""),
        ("DAEAC3", true, Some(5112212), "E-AC-3 JOC", "joc", "ec-3", "audio/mp4; codecs=\"ec-3\""),
        ("DA448", true, Some(8936764), "AC-4 A-JOC 448 kbps", "native", "ac-4", "audio/mp4; codecs=\"ac-4\""),
        ("DA1500", true, Some(29870162), "AC-4 A-JOC 1500 kbps", "native", "ac-4", "audio/mp4; codecs=\"ac-4\""),
    ];
    for (quality, available, size, format, playback, codec, content_type) in cases {
        let row = rows.iter().find(|row| row["quality"] == quality).unwrap();
        assert_eq!(row["available"], available, "{quality}");
        assert_eq!(row["sizeBytes"], json!(size), "{quality}");
        assert_eq!(row["format"], format, "{quality}");
        assert_eq!(row["playback"], playback, "{quality}");
        assert_eq!(row["codec"], codec, "{quality}");
        assert_eq!(row["contentType"], content_type, "{quality}");
    }
    for private in ["vkey", "file_url", "qqmusic.qq.com", "example.invalid"] {
        assert!(!body.to_string().contains(private));
    }

    // A live URL for another tier cannot make a zero or missing HQ size available.
    for file in [json!({"size_320mp3":0}), json!({})] {
        qq.infos.put(
            "abc".into(),
            json!({"abc":{"track_info":{"mid":"abc","file":file}}}),
        );
        let response = get(state.clone(), "/qq/song/abc/qualities").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        let row = body["qualities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["quality"] == "HQ")
            .unwrap();
        assert_eq!(row["available"], false);
        assert_eq!(row["sizeBytes"], Value::Null);
    }
}

#[tokio::test]
async fn basic_qualities_use_only_info_fields_and_negative_probe_cache() {
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    qq.infos.put(
        "abc".into(),
        json!({"mid":"abc","file":{"size_320mp3":123,"size_flac":456,"size_dolby":789}}),
    );
    let response = get(state.clone(), "/qq/song/abc/qualities?basic=1").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let rows = body["qualities"].as_array().unwrap();
    assert_eq!(rows.len(), 13);
    assert!(rows.iter().all(|row| {
        [
            "MP3128", "AAC48", "WEB", "AAC192", "OGG96", "OGG192", "HQ", "SQ", "RS", "RA360",
            "DTS", "DA", "DAEAC3",
        ]
        .contains(&row["quality"].as_str().unwrap())
    }));
    assert!(!rows.iter().any(|row| row["quality"] == "NAC"));
}

#[tokio::test]
async fn audio_url_ignores_negative_probe_cache_but_probe_uses_it() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = hits.clone();
    let upstream = Router::new().fallback(any(move |request: Request<Body>| {
        let seen = seen.clone();
        async move {
            if request.uri().path() == "/extra/music/url" {
                seen.fetch_add(1, Ordering::SeqCst);
                return Json(json!([{"track_info": {
                    "file_url": "https://isure.stream.qqmusic.qq.com/audio.mp3",
                    "file_size": 123
                }}])).into_response();
            }
            StatusCode::NOT_FOUND.into_response()
        }
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = Some(base);
    let state = Arc::new(AppState::with_config(config));
    let qq = state.qq.as_ref().unwrap();
    qq.audio_failures.put(Quality::Nac.cache_key("abc"), ());
    assert!(qq.audio_url("abc", Quality::Nac).await.is_ok());
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    qq.audio_failures.put(Quality::Dtsx.cache_key("abc"), ());
    assert!(qq.probe_audio_url("abc", Quality::Dtsx).await.is_err());
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    task.abort();
}

#[tokio::test]
async fn lyrics_sources_select_expected_ttl_and_all_failures_are_not_cached() {
    for (mode, expected) in [
        (0u8, Duration::from_secs(3600)),
        (1, Duration::from_secs(604800)),
        (2, Duration::from_secs(300)),
        (3, Duration::from_secs(300)),
    ] {
        let upstream = Router::new().fallback(any(move |request: Request<Body>| async move {
            let path = request.uri().path();
            match (mode, path) {
                (0, "/v1/lyrics/get") | (0, "/lyric/ttml") | (0, "/lyric/new") | (0, "/extra/music/info") => StatusCode::NOT_FOUND.into_response(),
                (1, "/v1/lyrics/get") => Json(json!({"data":{"lyrics":"<tt><body><p begin=\"0\">amll</p></body></tt>"}})).into_response(),
                (1, "/lyric/ttml") | (1, "/lyric/new") | (1, "/extra/music/info") => StatusCode::NOT_FOUND.into_response(),
                (2, "/v1/lyrics/get") => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                (2, "/lyric/ttml") | (2, "/lyric/new") | (2, "/extra/music/info") => StatusCode::NOT_FOUND.into_response(),
                (3, "/v1/lyrics/get") | (3, "/lyric/ttml") => StatusCode::NOT_FOUND.into_response(),
                (3, "/lyric/new") => Json(json!({"data":{"qrc":"[0,1000]candidate"}})).into_response(),
                (3, "/extra/music/info") => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }));
        let Some((base, task)) = serve_local(upstream).await else { return; };
        let mut config = AppState::new("http://127.0.0.1:1".into()).config;
        config.qq_api = Some(base.clone());
        config.amll_api = base;
        let state = Arc::new(AppState::with_config(config));
        let result = fetch_lyrics(state.qq.as_ref().unwrap(), "ttl").await;
        let (_, ttl) = match result { Ok(value) => value, Err(_) => panic!("lyrics mode {mode} unexpectedly failed") };
        assert_eq!(ttl, expected, "mode {mode}");
        task.abort();
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let upstream = Router::new().fallback(any(move |request: Request<Body>| {
        let seen = seen.clone();
        async move {
            if request.uri().path().starts_with("/v1/lyrics")
                || request.uri().path().starts_with("/lyric/")
                || request.uri().path() == "/extra/music/info"
            {
                seen.fetch_add(1, Ordering::SeqCst);
            }
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = Some(base.clone());
    config.amll_api = base;
    let state = Arc::new(AppState::with_config(config));
    for _ in 0..2 {
        assert_eq!(get(state.clone(), "/qq/song/fail/lyrics").await.status(), StatusCode::BAD_GATEWAY);
    }
    assert!(calls.load(Ordering::SeqCst) >= 8, "failed lyrics are retried and not cached");
    assert!(state.qq.as_ref().unwrap().lyrics.get("fail").is_none());
    task.abort();
}

#[tokio::test]
async fn quality_probes_are_bounded_and_cache_negative_results() {
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(AtomicUsize::new(0));
    let upstream = {
        let active = active.clone();
        let peak = peak.clone();
        let requests = requests.clone();
        Router::new().fallback(any(move |request: Request<Body>| {
            let active = active.clone();
            let peak = peak.clone();
            let requests = requests.clone();
            async move {
                match request.uri().path() {
                    "/extra/music/info" => {
                        let mut track = raw_song();
                        track["mid"] = json!("abc");
                        track["file"] = json!({});
                        Json(json!({"abc":{"track_info":track}})).into_response()
                    }
                    "/extra/music/url" => {
                        requests.fetch_add(1, Ordering::SeqCst);
                        peak.fetch_max(active.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        active.fetch_sub(1, Ordering::SeqCst);
                        Json(json!([{"track_info":{"file_size":0}}])).into_response()
                    }
                    _ => StatusCode::NOT_FOUND.into_response(),
                }
            }
        }))
    };
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = Some(base);
    let state = Arc::new(AppState::with_config(config));
    let qq = state.qq.as_ref().unwrap();
    assert!(qq.qualities("abc", false).await.is_ok(), "quality probe request failed");
    assert_eq!(requests.load(Ordering::SeqCst), 7);
    assert_eq!(peak.load(Ordering::SeqCst), 3);
    for quality in [Quality::Nac, Quality::Qai, Quality::Q360v1, Quality::Q360v2, Quality::Dtsx, Quality::Da448, Quality::Da1500] {
        assert!(qq.audio_failures.get(&quality.cache_key("abc")).is_some(), "{quality:?}");
    }
    task.abort();

    let attempts = Arc::new(AtomicUsize::new(0));
    let seen = attempts.clone();
    let upstream = Router::new().fallback(any(move |request: Request<Body>| {
        let seen = seen.clone();
        async move {
            if request.uri().path() != "/extra/music/url" {
                return StatusCode::NOT_FOUND.into_response();
            }
            if seen.fetch_add(1, Ordering::SeqCst) == 0 {
                StatusCode::SERVICE_UNAVAILABLE.into_response()
            } else {
                Json(json!([{"track_info":{"file_url":"https://isure.stream.qqmusic.qq.com/retry.mp3","file_size":123}}])).into_response()
            }
        }
    }));
    let Some((base, task)) = serve_local(upstream).await else { return; };
    let mut config = AppState::new("http://127.0.0.1:1".into()).config;
    config.qq_api = Some(base);
    let state = Arc::new(AppState::with_config(config));
    let qq = state.qq.as_ref().unwrap();
    assert!(qq.probe_audio_url("retry", Quality::Nac).await.is_err());
    assert!(qq.audio_failures.get(&Quality::Nac.cache_key("retry")).is_none());
    assert!(qq.probe_audio_url("retry", Quality::Nac).await.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    task.abort();
}

#[tokio::test]
async fn cached_payload_overrides_verified_default_without_cross_tier_leakage() {
    let state = state(true);
    let qq = state.qq.as_ref().unwrap();
    qq.infos.put(
        "abc".into(),
        json!({"abc":{"track_info":{"mid":"abc","file":{
            "size_96aac":960,"size_dolby":5112212
        }}}}),
    );
    for quality in QUALITY_LIST
        .into_iter()
        .filter(|quality| quality.size_field().is_none())
    {
        qq.audio
            .put(quality.cache_key("abc"), audio(quality, None, "nac"));
    }
    let mut da = audio(Quality::Da, Some(5112212), "mp4");
    da.codec_hint = Some(Codec::Ec3);
    qq.audio.put(Quality::Da.cache_key("abc"), da);
    qq.audio.put(
        Quality::Web.cache_key("abc"),
        audio(Quality::Web, Some(1280), "mp3"),
    );
    let response = get(state, "/qq/song/abc/qualities").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let rows = body["qualities"].as_array().unwrap();
    let row = |quality: &str| rows.iter().find(|row| row["quality"] == quality).unwrap();
    assert_eq!(row("WEB")["sizeBytes"], 1280);
    assert_eq!(row("WEB")["codec"], "mp3");
    assert_eq!(row("DA")["playback"], "native");
    assert_eq!(row("DA")["format"], "AC-4 Dolby Atmos");
    assert_eq!(row("DAEAC3")["playback"], "joc");
    assert_eq!(row("DAEAC3")["sizeBytes"], 5112212);
}
