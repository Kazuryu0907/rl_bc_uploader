# Rocket League Stats API

## 概要

ゲームクライアントがローカル WebSocket でゲームデータをブロードキャストする API。
カスタム HUD や配信オーバーレイなどのサードパーティアプリ向け。

- **種別**: ローカル WebSocket (リモート API ではない)
- **ドキュメント**: https://www.rocketleague.com/en/developer/stats-api

---

## 設定

`<Install Dir>\TAGame\Config\DefaultStatsAPI.ini` をクライアント起動前に編集する。
起動中の変更は再起動するまで反映されない。

| 設定名 | 型 | デフォルト | 説明 |
|---|---|---|---|
| `PacketSendRate` | float | 0 (無効) | UpdateState の毎秒送信回数。0 より大きい値にしないと WebSocket が開かない。最大 120 |
| `Port` | int | 49123 | 待受ポート番号 |

---

## メッセージエンベロープ

すべてのメッセージが共通フォーマット:

```json
{
  "Event": "EventName",
  "Data": {}
}
```

**フィールド可視性の凡例:**
- `CONDITIONAL` — 該当する状況のときのみ存在
- `SPECTATOR` — スペクテイター中、または同チームの場合のみ存在

`MatchGuid` はオンライン/LAN マッチのみセット。ローカルマッチでは空。

---

## Tick

### `UpdateState`

`PacketSendRate` に従って定期送信される。

```json
{
  "Event": "UpdateState",
  "Data": {
    "MatchGuid": "A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6",
    "Players": [ ],
    "Game": { }
  }
}
```

#### Players エントリ

| フィールド | 型 | 説明 |
|---|---|---|
| `Name` | string | 表示名 |
| `PrimaryId` | string | プラットフォーム ID (`"Steam|123|0"`, `"Epic|456|0"`) |
| `Shortcut` | int | スペクテイターショートカット番号 |
| `TeamNum` | int | チーム (0=Blue, 1=Orange) |
| `Score` | int | マッチスコア |
| `Goals` | int | ゴール数 |
| `Shots` | int | シュート数 |
| `Assists` | int | アシスト数 |
| `Saves` | int | セーブ数 |
| `Touches` | int | ボールタッチ数 |
| `CarTouches` | int | 車体タッチ数 |
| `Demos` | int | デモ数 |
| `bHasCar` | bool | `SPECTATOR` 車両保持中か |
| `Speed` | float | `SPECTATOR` 車速 (UU/s) |
| `Boost` | int | `SPECTATOR` ブースト量 0–100 |
| `bBoosting` | bool | `SPECTATOR` ブースト中か |
| `bOnGround` | bool | `SPECTATOR` 接地中か (車輪3本以上) |
| `bOnWall` | bool | `SPECTATOR` 壁走り中か |
| `bPowersliding` | bool | `SPECTATOR` パワースライド中か |
| `bDemolished` | bool | `SPECTATOR` デモされているか |
| `bSupersonic` | bool | `SPECTATOR` 超音速か |
| `Attacker` | object | `CONDITIONAL` デモした相手プレイヤー `{Name, Shortcut, TeamNum}` |

#### Game オブジェクト

| フィールド | 型 | 説明 |
|---|---|---|
| `Teams` | array | チーム一覧 (TeamNum 順) |
| `TimeSeconds` | int | 残り時間 (秒) |
| `bOvertime` | bool | 延長戦か |
| `Ball.Speed` | float | ボール速度 (UU/s) |
| `Ball.TeamNum` | int | 最後にタッチしたチーム (未タッチ=255) |
| `bReplay` | bool | ゴール/履歴リプレイ再生中か |
| `bHasWinner` | bool | 勝者決定済みか |
| `Winner` | string | 勝チーム名 (未決定は空文字) |
| `Arena` | string | マップアセット名 (例: `"Stadium_P"`) |
| `bHasTarget` | bool | スペクテイター対象ありか |
| `Target` | object | `CONDITIONAL` 視点対象プレイヤー `{Name, Shortcut, TeamNum}` |
| `Frame` | int | `CONDITIONAL` リプレイ中の現フレーム |
| `Elapsed` | float | `CONDITIONAL` リプレイ中のゲーム開始からの経過秒数 |

#### Teams エントリ

| フィールド | 型 | 説明 |
|---|---|---|
| `Name` | string | チーム名 |
| `TeamNum` | int | チームインデックス |
| `Score` | int | チームゴール数 |
| `ColorPrimary` | string | メインカラー (hex, # なし) |
| `ColorSecondary` | string | サブカラー (hex, # なし) |

---

## Events

イベントはそのイベントが発生したティックで即座に送信される (PacketSendRate に依存しない)。

### `BallHit`

ボールヒット後 1 フレームで送信。

| フィールド | 型 | 説明 |
|---|---|---|
| `Players` | array | ヒットしたプレイヤー一覧 `{Name, Shortcut, TeamNum}` |
| `Ball.PreHitSpeed` | float | ヒット前ボール速度 (UU/s) |
| `Ball.PostHitSpeed` | float | ヒット後ボール速度 (UU/s) |
| `Ball.Location` | vector | ヒット時のボール位置 `{X, Y, Z}` |

### `CrossbarHit`

ボールがクロスバーに当たったとき。

| フィールド | 型 | 説明 |
|---|---|---|
| `BallSpeed` | float | 衝突時のボール速度 |
| `ImpactForce` | float | クロスバー法線方向の衝突力 |
| `BallLocation` | vector | 衝突位置 `{X, Y, Z}` |
| `BallLastTouch.Player` | object | 最後にタッチしたプレイヤー `{Name, Shortcut, TeamNum}` |
| `BallLastTouch.Speed` | float | そのタッチ後のボール速度 |

### `GoalScored`

ゴールが入ったとき。

| フィールド | 型 | 説明 |
|---|---|---|
| `GoalSpeed` | float | ゴールライン通過時のボール速度 (UU/s) |
| `GoalTime` | float | 直前ラウンドの長さ (秒) |
| `ImpactLocation` | vector | ゴール時のボール位置 `{X, Y, Z}` |
| `Scorer` | object | 得点者 `{Name, Shortcut, TeamNum}` |
| `Assister` | object | `CONDITIONAL` アシスト者 (同形状) |
| `BallLastTouch.Player` | object | ゴール直前に最後にタッチしたプレイヤー |
| `BallLastTouch.Speed` | float | そのタッチ後のボール速度 |

### `MatchEnded`

試合終了・勝者決定時。

| フィールド | 型 | 説明 |
|---|---|---|
| `WinnerTeamNum` | int | 勝チームインデックス |

### `StatfeedEvent`

誰かがスタットを獲得したとき。

| フィールド | 型 | 説明 |
|---|---|---|
| `EventName` | string | スタット内部名 (例: `"Demolish"`, `"Save"`) |
| `Type` | string | 表示ラベル (例: `"Demolition"`) |
| `MainTarget` | object | スタット獲得プレイヤー `{Name, Shortcut, TeamNum}` |
| `SecondaryTarget` | object | `CONDITIONAL` 対象プレイヤー (例: デモされた側、同形状) |

### `ClockUpdatedSeconds`

ゲーム内クロック変化時。

| フィールド | 型 | 説明 |
|---|---|---|
| `TimeSeconds` | int | 残り時間 (秒) |
| `bOvertime` | bool | 延長戦か |

### その他イベント (Data は `MatchGuid` のみ)

| イベント | タイミング |
|---|---|
| `MatchCreated` | 全チーム生成・レプリケート完了時 |
| `MatchInitialized` | 最初のカウントダウン開始時 |
| `MatchDestroyed` | ゲーム退出時 |
| `MatchPaused` | 管理者による一時停止時 |
| `MatchUnpaused` | 一時停止解除時 |
| `CountdownBegin` | 各ラウンドのカウントダウン開始時 |
| `RoundStarted` | カウントダウン終了後、試合アクティブ状態に遷移時 |
| `GoalReplayStart` | ゴールリプレイ開始時 |
| `ReplayWillEnd` | ゴールリプレイ中にボールが爆発したとき (スキップ時は発火しない)。**注: ドキュメントは `GoalReplayWillEnd` だが実際のイベント名は `ReplayWillEnd`** |
| `GoalReplayEnd` | ゴールリプレイ終了時 |
| `PodiumStart` | 試合後ポディウム画面遷移時 |
| `ReplayCreated` | **Match History からリプレイをロードしたとき。リプレイファイルの保存とは無関係** |

## 実測による知見

| 知見 | 詳細 |
|---|---|
| `GoalScored` に `scorer=""`, `speed=0` のものが混入する | `ReplayWillEnd` 直後に発火するゴールリプレイ終了の副産物。本物のゴールは `scorer` が空にならない |
| `MatchEnded` → 約3秒後に `PodiumStart` | アップロードトリガーの候補 |
| `MatchDestroyed` の guid が試合 guid と異なることがある | guid による試合追跡に注意 |
| `MatchCreated` 直後は guid が空の場合がある | 次の試合の guid は少し遅れて確定する |
| プロトコルは WebSocket ではなく **生 TCP** | JSON オブジェクトを区切りなく連結してストリーム送信する |
