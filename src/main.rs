mod events;
mod listener;
mod uploader;
mod watcher;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub struct Config {
    pub token: String,
    pub demos_dir: PathBuf,
    pub tcp_addr: String,
    pub visibility: String,
    pub group: Option<String>,
    pub watch_timeout: Duration,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = std::env::var("BALLCHASING_TOKEN")
        .expect("BALLCHASING_TOKEN not set in .env");

    let demos_dir = {
        let profile = std::env::var("USERPROFILE").expect("USERPROFILE not set");
        PathBuf::from(profile)
            .join("Documents")
            .join("My Games")
            .join("Rocket League")
            .join("TAGame")
            .join("Demos")
    };

    let tcp_addr = std::env::var("RLS_TCP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:49123".to_string());

    let visibility = std::env::var("BALLCHASING_VISIBILITY")
        .unwrap_or_else(|_| "private".to_string());

    let group = std::env::var("BALLCHASING_GROUP")
        .ok()
        .filter(|s| !s.is_empty());

    let watch_timeout_secs: u64 = std::env::var("WATCH_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);
    let watch_timeout = Duration::from_secs(watch_timeout_secs);

    let file_appender = tracing_appender::rolling::daily("logs", "rl_uploader.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking.with_max_level(tracing::Level::DEBUG))
        .with_ansi(false)
        .init();

    let cfg = Arc::new(Config {
        token,
        demos_dir,
        tcp_addr,
        visibility,
        group,
        watch_timeout,
    });

    tracing::info!(
        "config: tcp_addr={} demos_dir={} visibility={} group={:?} watch_timeout={}s",
        cfg.tcp_addr,
        cfg.demos_dir.display(),
        cfg.visibility,
        cfg.group,
        cfg.watch_timeout.as_secs(),
    );

    if let Err(e) = listener::run(cfg).await {
        tracing::error!("listener error: {e}");
    }
}
