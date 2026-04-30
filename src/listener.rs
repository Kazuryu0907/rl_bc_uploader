use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::Config;
use crate::events::{Envelope, RLEvent};

pub async fn run(cfg: Arc<Config>) -> anyhow::Result<()> {
    tracing::info!("connecting to {}", cfg.tcp_addr);

    let mut stream = TcpStream::connect(&cfg.tcp_addr).await?;
    tracing::info!("connected");

    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut tmp = [0u8; 8192];

    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            tracing::info!("connection closed by server");
            break;
        }
        buf.extend_from_slice(&tmp[..n]);

        let consumed = parse_buf(&buf, &cfg);
        buf.drain(..consumed);
    }

    Ok(())
}

/// バッファから完結した JSON オブジェクトをすべて処理し、消費したバイト数を返す
fn parse_buf(buf: &[u8], cfg: &Arc<Config>) -> usize {
    let mut iter = serde_json::Deserializer::from_slice(buf).into_iter::<Envelope>();
    let mut last_ok = 0usize;

    loop {
        let offset_before = iter.byte_offset();
        match iter.next() {
            None => {
                last_ok = iter.byte_offset();
                break;
            }
            Some(Ok(envelope)) => {
                last_ok = iter.byte_offset();
                match RLEvent::from_envelope(envelope) {
                    Ok(event) => handle(event, cfg),
                    Err(e) => tracing::warn!("event parse error: {e}"),
                }
            }
            Some(Err(e)) if e.is_eof() => break, // 不完全、次の read を待つ
            Some(Err(e)) => {
                tracing::warn!("json error: {e}");
                last_ok = offset_before.saturating_add(1);
                break;
            }
        }
    }

    last_ok
}

fn handle(event: RLEvent, cfg: &Arc<Config>) {
    match event {
        RLEvent::UpdateState(_) => {
            tracing::debug!("UpdateState");
        }
        RLEvent::MatchCreated(d) => {
            tracing::info!("[MatchCreated] guid={}", d.match_guid);
        }
        RLEvent::MatchInitialized(d) => {
            tracing::info!("[MatchInitialized] guid={}", d.match_guid);
        }
        RLEvent::CountdownBegin(d) => {
            tracing::info!("[CountdownBegin] guid={}", d.match_guid);
        }
        RLEvent::RoundStarted(d) => {
            tracing::info!("[RoundStarted] guid={}", d.match_guid);
        }
        RLEvent::BallHit(d) => {
            let hitter = d.players.first().map(|p| p.name.as_str()).unwrap_or("?");
            tracing::debug!(
                "[BallHit] by={hitter} pre={:.0} post={:.0}",
                d.ball.pre_hit_speed,
                d.ball.post_hit_speed,
            );
        }
        RLEvent::CrossbarHit(d) => {
            tracing::info!("[CrossbarHit] speed={:.0}", d.ball_speed);
        }
        RLEvent::GoalScored(d) => {
            tracing::info!(
                "[GoalScored] scorer={} speed={:.0}",
                d.scorer.name,
                d.goal_speed,
            );
        }
        RLEvent::StatfeedEvent(d) => {
            tracing::info!(
                "[StatfeedEvent] {} -> {}",
                d.main_target.name,
                d.event_name,
            );
        }
        RLEvent::GoalReplayStart(d) => {
            tracing::info!("[GoalReplayStart] guid={}", d.match_guid);
        }
        RLEvent::ReplayWillEnd(d) => {
            tracing::info!("[ReplayWillEnd] guid={}", d.match_guid);
        }
        RLEvent::GoalReplayEnd(d) => {
            tracing::info!("[GoalReplayEnd] guid={}", d.match_guid);
        }
        RLEvent::MatchPaused(d) => {
            tracing::info!("[MatchPaused] guid={}", d.match_guid);
        }
        RLEvent::MatchUnpaused(d) => {
            tracing::info!("[MatchUnpaused] guid={}", d.match_guid);
        }
        RLEvent::MatchEnded(d) => {
            tracing::info!(
                "[MatchEnded] guid={} winner_team={}",
                d.match_guid,
                d.winner_team_num,
            );
            // MatchEnded から少し遡った時刻をベースラインにしてリプレイファイルを待つ
            let since = SystemTime::now() - Duration::from_secs(5);
            let cfg = cfg.clone();
            tokio::spawn(async move {
                tracing::info!("[Upload] waiting for replay file…");
                match crate::watcher::wait_for_new_replay(
                    &cfg.demos_dir,
                    since,
                    cfg.watch_timeout,
                )
                .await
                {
                    Ok(path) => {
                        tracing::info!("[Upload] found: {}", path.display());
                        match crate::uploader::upload(
                            &path,
                            &cfg.token,
                            &cfg.visibility,
                            cfg.group.as_deref(),
                        )
                        .await
                        {
                            Ok(id) => tracing::info!("[Upload] success id={id}"),
                            Err(e) => tracing::error!("[Upload] failed: {e}"),
                        }
                    }
                    Err(e) => tracing::error!("[Upload] watcher error: {e}"),
                }
            });
        }
        RLEvent::PodiumStart(d) => {
            tracing::info!("[PodiumStart] guid={}", d.match_guid);
        }
        RLEvent::ReplayCreated(d) => {
            tracing::info!("[ReplayCreated] guid={}", d.match_guid);
        }
        RLEvent::MatchDestroyed(d) => {
            tracing::info!("[MatchDestroyed] guid={}", d.match_guid);
        }
        RLEvent::ClockUpdatedSeconds(d) => {
            tracing::debug!(
                "[ClockUpdatedSeconds] time={}s overtime={}",
                d.time_seconds,
                d.overtime,
            );
        }
        RLEvent::Unknown { event } => {
            tracing::warn!("[Unknown] event={event}");
        }
    }
}
