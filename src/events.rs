use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Common sub-types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerRef {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Shortcut")]
    pub shortcut: i32,
    #[serde(rename = "TeamNum")]
    pub team_num: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vector3 {
    #[serde(rename = "X")]
    pub x: f64,
    #[serde(rename = "Y")]
    pub y: f64,
    #[serde(rename = "Z")]
    pub z: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BallLastTouch {
    #[serde(rename = "Player")]
    pub player: PlayerRef,
    #[serde(rename = "Speed")]
    pub speed: f64,
}

// ---------------------------------------------------------------------------
// BallHit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BallHitBall {
    #[serde(rename = "PreHitSpeed")]
    pub pre_hit_speed: f64,
    #[serde(rename = "PostHitSpeed")]
    pub post_hit_speed: f64,
    #[serde(rename = "Location")]
    pub location: Vector3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BallHitData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "Players")]
    pub players: Vec<PlayerRef>,
    #[serde(rename = "Ball")]
    pub ball: BallHitBall,
}

// ---------------------------------------------------------------------------
// CrossbarHit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrossbarHitData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "BallSpeed")]
    pub ball_speed: f64,
    #[serde(rename = "ImpactForce")]
    pub impact_force: f64,
    #[serde(rename = "BallLocation")]
    pub ball_location: Vector3,
    #[serde(rename = "BallLastTouch")]
    pub ball_last_touch: BallLastTouch,
}

// ---------------------------------------------------------------------------
// GoalScored
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoalScoredData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "GoalSpeed")]
    pub goal_speed: f64,
    #[serde(rename = "GoalTime")]
    pub goal_time: f64,
    #[serde(rename = "ImpactLocation")]
    pub impact_location: Vector3,
    #[serde(rename = "Scorer")]
    pub scorer: PlayerRef,
    #[serde(rename = "BallLastTouch")]
    pub ball_last_touch: BallLastTouch,
    // CONDITIONAL
    #[serde(rename = "Assister", skip_serializing_if = "Option::is_none")]
    pub assister: Option<PlayerRef>,
}

// ---------------------------------------------------------------------------
// MatchEnded
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchEndedData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "WinnerTeamNum")]
    pub winner_team_num: i32,
}

// ---------------------------------------------------------------------------
// ClockUpdatedSeconds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClockUpdatedSecondsData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "TimeSeconds")]
    pub time_seconds: i32,
    #[serde(rename = "bOvertime")]
    pub overtime: bool,
}

// ---------------------------------------------------------------------------
// StatfeedEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatfeedEventData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "Type")]
    pub r#type: String,
    #[serde(rename = "MainTarget")]
    pub main_target: PlayerRef,
    // CONDITIONAL
    #[serde(rename = "SecondaryTarget", skip_serializing_if = "Option::is_none")]
    pub secondary_target: Option<PlayerRef>,
}

// ---------------------------------------------------------------------------
// MatchGuid-only events
// (MatchCreated, MatchInitialized, MatchDestroyed, MatchPaused, MatchUnpaused,
//  CountdownBegin, RoundStarted, GoalReplayStart, GoalReplayWillEnd,
//  GoalReplayEnd, PodiumStart, ReplayCreated)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchGuidData {
    #[serde(rename = "MatchGuid")]
    pub match_guid: String,
}

// ---------------------------------------------------------------------------
// Top-level envelope
// RL Stats API は {"Event":"...","Data":"<escaped json string>"} の形式で
// Data が JSON 文字列にエスケープされた二重構造になっている。
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct Envelope {
    #[serde(rename = "Event")]
    pub event: String,
    /// Data フィールドは JSON エスケープされた文字列
    #[serde(rename = "Data")]
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum RLEvent {
    UpdateState,
    BallHit(BallHitData),
    CrossbarHit(CrossbarHitData),
    GoalScored(GoalScoredData),
    MatchEnded(MatchEndedData),
    ClockUpdatedSeconds(ClockUpdatedSecondsData),
    StatfeedEvent(StatfeedEventData),
    MatchCreated(MatchGuidData),
    MatchInitialized(MatchGuidData),
    MatchDestroyed(MatchGuidData),
    MatchPaused(MatchGuidData),
    MatchUnpaused(MatchGuidData),
    CountdownBegin(MatchGuidData),
    RoundStarted(MatchGuidData),
    GoalReplayStart(MatchGuidData),
    /// ドキュメントは "GoalReplayWillEnd" だが実際のイベント名は "ReplayWillEnd"
    ReplayWillEnd(MatchGuidData),
    GoalReplayEnd(MatchGuidData),
    PodiumStart(MatchGuidData),
    ReplayCreated(MatchGuidData),
    Unknown { event: String },
}

impl RLEvent {
    pub fn from_envelope(env: Envelope) -> anyhow::Result<Self> {
        let d = &env.data;
        let ev = match env.event.as_str() {
            "UpdateState"          => RLEvent::UpdateState,
            "BallHit"              => RLEvent::BallHit(serde_json::from_str(d)?),
            "CrossbarHit"          => RLEvent::CrossbarHit(serde_json::from_str(d)?),
            "GoalScored"           => RLEvent::GoalScored(serde_json::from_str(d)?),
            "MatchEnded"           => RLEvent::MatchEnded(serde_json::from_str(d)?),
            "ClockUpdatedSeconds"  => RLEvent::ClockUpdatedSeconds(serde_json::from_str(d)?),
            "StatfeedEvent"        => RLEvent::StatfeedEvent(serde_json::from_str(d)?),
            "MatchCreated"         => RLEvent::MatchCreated(serde_json::from_str(d)?),
            "MatchInitialized"     => RLEvent::MatchInitialized(serde_json::from_str(d)?),
            "MatchDestroyed"       => RLEvent::MatchDestroyed(serde_json::from_str(d)?),
            "MatchPaused"          => RLEvent::MatchPaused(serde_json::from_str(d)?),
            "MatchUnpaused"        => RLEvent::MatchUnpaused(serde_json::from_str(d)?),
            "CountdownBegin"       => RLEvent::CountdownBegin(serde_json::from_str(d)?),
            "RoundStarted"         => RLEvent::RoundStarted(serde_json::from_str(d)?),
            "GoalReplayStart"      => RLEvent::GoalReplayStart(serde_json::from_str(d)?),
            "ReplayWillEnd"        => RLEvent::ReplayWillEnd(serde_json::from_str(d)?),
            "GoalReplayEnd"        => RLEvent::GoalReplayEnd(serde_json::from_str(d)?),
            "PodiumStart"          => RLEvent::PodiumStart(serde_json::from_str(d)?),
            "ReplayCreated"        => RLEvent::ReplayCreated(serde_json::from_str(d)?),
            other                  => RLEvent::Unknown { event: other.to_string() },
        };
        Ok(ev)
    }
}
