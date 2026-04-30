mod events;
mod listener;
mod update;
mod uploader;
mod watcher;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

pub struct Config {
    pub token: String,
    pub demos_dir: PathBuf,
    pub tcp_addr: String,
    pub visibility: String,
    pub group: Option<String>,
    pub watch_timeout: Duration,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn pause_then_exit(code: u8) -> ! {
    eprintln!();
    eprintln!("Enter キーを押すと終了します...");
    let _ = std::io::stdin().read_line(&mut String::new());
    std::process::exit(code as i32);
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    // File: full debug + timestamps. Console: info-and-up, plain message only.
    let file_appender = tracing_appender::rolling::daily("logs", "rl_uploader.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_filter(LevelFilter::DEBUG);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    println!("=== rl_bc_uploader v{VERSION} ===");
    println!();

    if std::env::var("SKIP_UPDATE").is_err() {
        match tokio::task::spawn_blocking(update::check_and_apply).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::warn!("[アップデート] 確認失敗: {e}"),
            Err(e) => tracing::warn!("[アップデート] タスクエラー: {e}"),
        }
    }

    let token = match std::env::var("BALLCHASING_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => {
            eprintln!();
            eprintln!("[エラー] BALLCHASING_TOKEN が .env に設定されていません");
            eprintln!();
            eprintln!("セットアップ手順は README を参照してください:");
            eprintln!("  https://github.com/Kazuryu0907/rl_bc_uploader#セットアップ初回だけ5分");
            pause_then_exit(1);
        }
    };

    let demos_dir = match std::env::var("USERPROFILE") {
        Ok(p) => PathBuf::from(p)
            .join("Documents")
            .join("My Games")
            .join("Rocket League")
            .join("TAGame")
            .join("Demos"),
        Err(_) => {
            eprintln!("[エラー] USERPROFILE 環境変数が見つかりません(Windows 以外?)");
            pause_then_exit(1);
        }
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

    let cfg = Arc::new(Config {
        token,
        demos_dir,
        tcp_addr,
        visibility,
        group,
        watch_timeout,
    });

    tracing::debug!(
        "config: tcp_addr={} demos_dir={} visibility={} group={:?} watch_timeout={}s",
        cfg.tcp_addr,
        cfg.demos_dir.display(),
        cfg.visibility,
        cfg.group,
        cfg.watch_timeout.as_secs(),
    );

    if let Err(e) = listener::run(cfg).await {
        tracing::error!("[エラー] 接続エラー: {e}");
        eprintln!();
        eprintln!("Rocket League が起動していて、Stats API (PacketSendRate=30) が");
        eprintln!("有効になっているか確認してください。");
        eprintln!("詳しくは README の「うまく動かないとき」を参照。");
        pause_then_exit(1);
    }

    ExitCode::SUCCESS
}
