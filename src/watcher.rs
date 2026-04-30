use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

/// MatchEnded 以降に作成された新しい .replay ファイルが見つかるまでポーリングする。
/// `since` より新しいファイルが見つかれば Ok(path)。`timeout` 経過で Err。
pub async fn wait_for_new_replay(
    demos_dir: &Path,
    since: SystemTime,
    timeout: Duration,
) -> anyhow::Result<PathBuf> {
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!(
                "timed out ({}s) waiting for new replay in {}",
                timeout.as_secs(),
                demos_dir.display()
            );
        }

        let mut rd = tokio::fs::read_dir(demos_dir).await?;
        let mut newest: Option<(SystemTime, PathBuf)> = None;

        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("replay") {
                continue;
            }
            let meta = match tokio::fs::metadata(&path).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            // created() が使えない場合は modified() にフォールバック
            let ts = meta.created().or_else(|_| meta.modified())?;
            if ts > since {
                match &newest {
                    None => newest = Some((ts, path)),
                    Some((t, _)) if ts > *t => newest = Some((ts, path)),
                    _ => {}
                }
            }
        }

        if let Some((_, path)) = newest {
            return Ok(path);
        }

        sleep(Duration::from_secs(2)).await;
    }
}
