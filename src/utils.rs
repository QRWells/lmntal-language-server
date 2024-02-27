use std::time::Duration;

use lmntalc::util::Pos;
use reqwest::ClientBuilder;
use semver::Version;
use tower_lsp::lsp_types::Position;

pub fn to_position(pos: Pos) -> Position {
    Position {
        line: pos.line,
        character: pos.column,
    }
}

pub fn span_to_range(span: lmntalc::util::Span) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range {
        start: to_position(span.low()),
        end: to_position(span.high()),
    }
}

pub async fn check_update() -> Option<Version> {
    let client = ClientBuilder::new()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(2))
        .build();

    if let Ok(client) = client {
        let response = client
            .get("https://crates.io/api/v1/crates/lmntal-language-server")
            .send()
            .await;

        if let Ok(response) = response {
            return response
                .json::<serde_json::Value>()
                .await
                .map_or(None, |json| {
                    let version = json["crate"]["max_version"].as_str()?;
                    let version = Version::parse(version).ok()?;
                    let current = Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
                    if version > current {
                        Some(version)
                    } else {
                        None
                    }
                });
        }
    }

    None
}
