use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::Config;
use crate::events::{Envelope, RLEvent};

const RECONNECT_DELAY: Duration = Duration::from_secs(3);

pub async fn run(cfg: Arc<Config>) -> anyhow::Result<()> {
    tracing::info!("Rocket League (Stats API) に接続中: {}", cfg.tcp_addr);
    let mut announced_waiting = false;

    loop {
        match TcpStream::connect(&cfg.tcp_addr).await {
            Ok(stream) => {
                announced_waiting = false;
                tracing::info!("[OK] 接続しました — 試合の終了を待機中...");
                if let Err(e) = read_loop(stream, &cfg).await {
                    tracing::debug!("read loop error: {e}");
                }
                tracing::info!(
                    "[切断] Rocket League との接続が切れました — 再接続を試みます"
                );
            }
            Err(e) => {
                if !announced_waiting {
                    tracing::warn!(
                        "[待機] Rocket League に接続できません ({e}) — 起動と PacketSendRate=30 を確認してください"
                    );
                    announced_waiting = true;
                } else {
                    tracing::debug!("connect retry: {e}");
                }
            }
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

async fn read_loop(mut stream: TcpStream, cfg: &Arc<Config>) -> anyhow::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut tmp = [0u8; 8192];

    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&tmp[..n]);

        let consumed = parse_buf(&buf, cfg);
        buf.drain(..consumed);
    }
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
        RLEvent::UpdateState => tracing::debug!("UpdateState"),
        RLEvent::MatchCreated(d) => tracing::debug!("[MatchCreated] guid={}", d.match_guid),
        RLEvent::MatchInitialized(d) => {
            tracing::debug!("[MatchInitialized] guid={}", d.match_guid)
        }
        RLEvent::CountdownBegin(d) => tracing::debug!("[CountdownBegin] guid={}", d.match_guid),
        RLEvent::RoundStarted(d) => tracing::debug!("[RoundStarted] guid={}", d.match_guid),
        RLEvent::BallHit(d) => {
            let hitter = d.players.first().map(|p| p.name.as_str()).unwrap_or("?");
            tracing::debug!(
                "[BallHit] by={hitter} pre={:.0} post={:.0}",
                d.ball.pre_hit_speed,
                d.ball.post_hit_speed,
            );
        }
        RLEvent::CrossbarHit(d) => tracing::debug!("[CrossbarHit] speed={:.0}", d.ball_speed),
        RLEvent::GoalScored(d) => {
            tracing::debug!(
                "[GoalScored] scorer={} speed={:.0}",
                d.scorer.name,
                d.goal_speed,
            );
        }
        RLEvent::StatfeedEvent(d) => {
            tracing::debug!(
                "[StatfeedEvent] {} -> {}",
                d.main_target.name,
                d.event_name,
            );
        }
        RLEvent::GoalReplayStart(d) => {
            tracing::debug!("[GoalReplayStart] guid={}", d.match_guid)
        }
        RLEvent::ReplayWillEnd(d) => tracing::debug!("[ReplayWillEnd] guid={}", d.match_guid),
        RLEvent::GoalReplayEnd(d) => tracing::debug!("[GoalReplayEnd] guid={}", d.match_guid),
        RLEvent::MatchPaused(d) => tracing::debug!("[MatchPaused] guid={}", d.match_guid),
        RLEvent::MatchUnpaused(d) => tracing::debug!("[MatchUnpaused] guid={}", d.match_guid),
        RLEvent::MatchEnded(d) => {
            tracing::info!(
                "[試合終了] チーム{}の勝利 — リプレイの保存を待機中...",
                d.winner_team_num,
            );
            tracing::debug!("MatchEnded guid={}", d.match_guid);
            // MatchEnded から少し遡った時刻をベースラインにしてリプレイファイルを待つ
            let since = SystemTime::now() - Duration::from_secs(5);
            let cfg = cfg.clone();
            tokio::spawn(async move {
                match crate::watcher::wait_for_new_replay(
                    &cfg.demos_dir,
                    since,
                    cfg.watch_timeout,
                )
                .await
                {
                    Ok(path) => {
                        tracing::info!("[検出] リプレイファイルを発見");
                        tracing::debug!("found: {}", path.display());
                        match crate::uploader::upload(
                            &path,
                            &cfg.token,
                            &cfg.visibility,
                            cfg.group.as_deref(),
                        )
                        .await
                        {
                            Ok(id) => tracing::info!(
                                "[完了] アップロード成功: https://ballchasing.com/replay/{}",
                                id,
                            ),
                            Err(e) => tracing::error!("[失敗] アップロード: {e}"),
                        }
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("timed out") {
                            tracing::info!(
                                "[スキップ] リプレイが保存されなかったためスキップ"
                            );
                        } else {
                            tracing::error!("[失敗] リプレイ検出: {e}");
                        }
                    }
                }
            });
        }
        RLEvent::PodiumStart(d) => tracing::debug!("[PodiumStart] guid={}", d.match_guid),
        RLEvent::ReplayCreated(d) => tracing::debug!("[ReplayCreated] guid={}", d.match_guid),
        RLEvent::MatchDestroyed(d) => tracing::debug!("[MatchDestroyed] guid={}", d.match_guid),
        RLEvent::ClockUpdatedSeconds(d) => {
            tracing::debug!(
                "[ClockUpdatedSeconds] time={}s overtime={}",
                d.time_seconds,
                d.overtime,
            );
        }
        RLEvent::Unknown { event } => tracing::warn!("[Unknown] event={event}"),
    }
}
