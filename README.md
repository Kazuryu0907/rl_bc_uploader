# rl_uploader

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

Rocket League の試合終了を検知して、保存されたリプレイファイルを自動で [ballchasing.com](https://ballchasing.com) にアップロードする小さな常駐ツール。

**BakkesMod 不要** — Rocket League 内蔵の公式 Stats API (TCP) を直接購読します。

---

## 特長

- Rocket League の `MatchEnded` イベントを TCP で受信
- 試合終了後に手動セーブされた `.replay` ファイルを自動検出
- ballchasing.com に multipart アップロード(visibility / group 指定可)
- 設定は `.env` で完結
- 常時稼働の小さな Rust バイナリ(リリースビルドで数 MB)
- **起動時に GitHub Releases から自動アップデート**(`self_update`)

---

## 動作要件

- **Rocket League** + Stats API 有効化(下記参照)
- **Rust toolchain**(stable, edition 2024)
- **ballchasing.com アカウント** + API トークン

### Stats API の有効化

Rocket League を **起動する前に** `<RL Install Dir>\TAGame\Config\DefaultStatsAPI.ini` を編集します:

```ini
PacketSendRate=30
Port=49123
```

| 項目 | 注意 |
|---|---|
| `PacketSendRate=0` | デフォルトは 0 で、これだと TCP サーバーが起動しません。**必ず 0 より大きい値にしてください**(30 程度で十分)。 |
| 起動中の編集 | 反映されません。RL を再起動してください。 |

### API トークンの取得

1. ballchasing.com にログイン
2. https://ballchasing.com/upload にアクセスして API token をコピー

---

## セットアップ

```sh
cp .env.example .env
# .env を編集して BALLCHASING_TOKEN を設定
cargo build --release
```

## 実行

```sh
./target/release/rl_uploader
```

Rocket League が起動して Stats API がリッスン中であることを確認してから実行してください。試合終了 → リプレイ手動セーブで自動アップロードされます。

ログ(`logs/rl_uploader.log.<UTC日付>`、日次ローテーション)で注目すべきイベント:

| イベント | 意味 |
|---|---|
| `connecting to ...` / `connected` | Stats API への接続成功 |
| `[MatchEnded]` | 試合終了を受信 |
| `[Upload] found: <path>` | リプレイファイルを検出 |
| `[Upload] success id=<uuid>` | アップロード成功(`https://ballchasing.com/replay/<uuid>` で閲覧可) |
| `[Upload] failed: ...` | アップロード失敗(ネットワーク・認証・409 重複等) |
| `[Upload] watcher error: timed out` | `WATCH_TIMEOUT_SECS` 内に手動セーブされなかった(設計通り) |

---

## 設定

すべて `.env` で設定します。詳細は [`.env.example`](./.env.example) も参照。

| 変数 | 必須 | デフォルト | 説明 |
|---|---|---|---|
| `BALLCHASING_TOKEN` | ✅ | — | ballchasing API トークン |
| `BALLCHASING_VISIBILITY` |   | `private` | `public` / `unlisted` / `private` |
| `BALLCHASING_GROUP` |   | (空) | アップロード先のグループ ID(ballchasing 上で作成) |
| `RLS_TCP_ADDR` |   | `127.0.0.1:49123` | RL Stats API の接続先 |
| `WATCH_TIMEOUT_SECS` |   | `300` | `MatchEnded` 後にリプレイ検出を待つ最大秒数 |
| `SKIP_UPDATE` |   | (空) | 任意の値を設定すると起動時の自動アップデート確認をスキップ |

---

## 仕組み

```
Rocket League (Stats API)
       │ TCP push (port 49123)
       ▼
   listener (TCP client, JSON envelope parser)
       │
       ├─ on MatchEnded:
       │      spawn(watcher → uploader)
       │
       ▼
   watcher (polls TAGame\Demos\ every 2s)
       │ found new .replay
       ▼
   uploader (multipart POST → ballchasing /api/v2/upload)
```

ポイント:

- `listener` は TCP **クライアント**として Stats API に接続します。サーバー側は Rocket League 自身。
- `watcher` は `MatchEnded - 5s` をベースラインに 2 秒ごとにポーリングし、新規 `.replay` を `created()`(なければ `modified()`)で判定。
- 重複検出は ballchasing 側に委譲(同じファイルなら HTTP 409 が返る)。

---

## 自動アップデート

起動時に GitHub Releases (`Kazuryu0907/rl_bc_uploader`) を確認し、新しい安定版があれば自動でダウンロードして自身を置き換えます。新しいバージョンに置き換わったらプロセスを exit するので、再起動してください(タスクスケジューラ等の常駐配下なら自動で再起動される想定)。

- 確認をスキップするには `SKIP_UPDATE=1` を設定して実行
- ログには `[update] up-to-date (vX.Y.Z)` または `[update] applied vX.Y.Z` が出力される
- ネットワーク不通等で確認失敗しても起動は継続される(`[update] check failed: ...`)

### リリースの作り方(メンテナ向け)

タグ push のみでリリースが自動公開されます:

```sh
git tag v0.1.0
git push origin v0.1.0
```

`.github/workflows/release.yml` が:
1. Windows 上で `cargo build --release`
2. `rl_uploader-vX.Y.Z-x86_64-pc-windows-msvc.zip` にパッケージ
3. GitHub Release を作成し zip を添付

`Cargo.toml` の `version` とタグの `vX.Y.Z` が一致するようにしてください(`self_update` は `cargo_crate_version!` で現バージョンを判定するため)。

---

## トラブルシューティング

### 起動するが `connected` のあと無音
- Rocket League が起動していない
- `PacketSendRate=0` のまま — `DefaultStatsAPI.ini` を確認して RL を再起動
- ファイアウォールが localhost ポートをブロックしている可能性

### `[Upload] watcher error: timed out`
- 試合終了後にリプレイを手動セーブしなかった、もしくは `WATCH_TIMEOUT_SECS` を超過した
- 頻発する場合は `WATCH_TIMEOUT_SECS` を伸ばす

### `[Upload] failed: HTTP 401`
- `BALLCHASING_TOKEN` が無効。次のコマンドで切り分け可能:
  ```sh
  curl https://ballchasing.com/api/ -H "Authorization: <token>"
  ```
  正しければ自分のアカウント情報が JSON で返ります。

### `[Upload] failed: HTTP 429`
- ballchasing のレートリミット超過。アカウントの quota は `GET /api/` で確認可能。

---

## 開発

```sh
cargo check       # 型・borrow チェック
cargo build       # debug
cargo build --release
```

ログレベルを変える場合は `src/main.rs` の `with_max_level(...)` を編集。

---

## ライセンス

MIT または Apache-2.0 の **デュアルライセンス**。利用者がいずれかを選択できます。

- [LICENSE-MIT](./LICENSE-MIT)
- [LICENSE-APACHE](./LICENSE-APACHE)

---

## 謝辞

- [Rocket League Stats API](https://www.rocketleague.com/en/developer/stats-api) — Psyonix 公式
- [ballchasing.com](https://ballchasing.com) — リプレイホスティング・統計分析プラットフォーム

---

## English (Summary)

`rl_uploader` is a small always-on Rust binary that listens to Rocket League's built-in Stats API over TCP, detects `MatchEnded`, waits for the player's manually-saved `.replay` to land in `Documents\My Games\Rocket League\TAGame\Demos`, and uploads it to [ballchasing.com](https://ballchasing.com).

**No BakkesMod required** — uses RL's official Stats API directly.

### Quick start

1. Edit `<RL Install Dir>\TAGame\Config\DefaultStatsAPI.ini` *before* launching RL:
   ```ini
   PacketSendRate=30
   Port=49123
   ```
   `PacketSendRate=0` (the default) leaves the TCP server closed.
2. Get your API token from https://ballchasing.com/upload.
3. Configure and run:
   ```sh
   cp .env.example .env   # then fill in BALLCHASING_TOKEN
   cargo build --release
   ./target/release/rl_uploader
   ```

### Configuration

All via `.env`. Only `BALLCHASING_TOKEN` is required; see [`.env.example`](./.env.example) for the rest (`BALLCHASING_VISIBILITY`, `BALLCHASING_GROUP`, `RLS_TCP_ADDR`, `WATCH_TIMEOUT_SECS`).

### How it works

`listener` (TCP client) → on `MatchEnded` → `watcher` polls `TAGame\Demos\` for a new `.replay` → `uploader` multipart-POSTs to `https://ballchasing.com/api/v2/upload`. Logs go to `logs/rl_uploader.log.<UTC_DATE>`.

### License

Dual-licensed under [MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE) at your option.
