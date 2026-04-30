# rl_bc_uploader

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**Rocket League の試合が終わると、自動でリプレイを [ballchasing.com](https://ballchasing.com) にアップロードしてくれるツール。**

毎回手動でアップロードする手間がなくなります。BakkesMod も不要です。

---

## 必要なもの

- Windows 10 / 11(64-bit)
- Rocket League(Steam 版・Epic 版どちらでも OK)
- ballchasing.com のアカウント(無料)

---

## セットアップ(初回だけ・5分)

エンジニアじゃなくてもできます。順番にやってください。

### 1. ballchasing.com の Token を取得

1. https://ballchasing.com にアクセスしてログイン(アカウントが無ければ無料で作れます)
2. https://ballchasing.com/upload を開く
3. ページ上部に表示されている **API Key**(40 文字くらいの英数字)をコピー — あとで使います

### 2. Rocket League の設定ファイルを書き換える

Rocket League の中にある設定ファイル `DefaultStatsAPI.ini` を編集すると、このツールが必要なデータを受け取れるようになります。

> **Rocket League を完全に終了してから**やってください。起動中の編集は反映されません。

1. エクスプローラーで Rocket League のインストール先の `TAGame\Config\` フォルダを開く
   - **Steam 版**: `C:\Program Files (x86)\Steam\steamapps\common\rocketleague\TAGame\Config\`(デフォルト。Steam ライブラリを別ドライブにしてる場合はそちら)
   - **Epic 版**: `C:\Program Files\Epic Games\rocketleague\TAGame\Config\`
2. `DefaultStatsAPI.ini` をメモ帳で開く(右クリック → 「プログラムから開く」 → メモ帳)
3. `PacketSendRate=0` の行を `PacketSendRate=30` に書き換えて保存

> ⚠️ `PacketSendRate=0` のままだとツールがデータを受け取れません。必ず `30` などの正の数にしてください。

### 3. ツール本体をダウンロード

[最新のリリース](https://github.com/Kazuryu0907/rl_bc_uploader/releases/latest) から、`rl_uploader-vX.Y.Z-x86_64-pc-windows-msvc.zip` をダウンロード。

zip を展開して中の `rl_uploader.exe` を**好きな場所**に置きます(例: `C:\Tools\rl_uploader\`、デスクトップ等)。**ここに置いた場所がツールの作業フォルダになります** — このあと作る設定ファイルも同じフォルダに置きます。

### 4. 設定ファイル `.env` を用意する

zip を展開すると `rl_uploader.exe` と一緒に `.env.example` という設定テンプレートが入っています。これを **`.env` にリネーム**します(`.example` の部分を消すだけ)。

> Windows で拡張子が見えない場合は、エクスプローラーの「表示」メニュー →「ファイル名拡張子」にチェックを入れてください。

リネームしたら `.env` をメモ帳で開いて、`BALLCHASING_TOKEN=` の行に手順 1 でコピーした Token を貼り付けて保存:

```
BALLCHASING_TOKEN=AbCdEf1234567890aBcDeF1234567890aBcDeF12
```

(`BALLCHASING_TOKEN=` 以外の行はそのままで OK。詳細は [詳細設定](#詳細設定オプション) 参照)

### 5. 起動

1. Rocket League を起動
2. `rl_uploader.exe` をダブルクリック
3. 黒いコンソールウィンドウが開いてそのまま動き続けたら **成功**(閉じないでください)

---

## 普段の使い方

1. Rocket League で普通に試合をプレイ
2. 試合終了画面で **「リプレイを保存」** ボタンを押す
3. 30 秒〜 1 分ほどで自動的に ballchasing.com にアップロードされる
4. https://ballchasing.com/replays で自分のアップロード済みリプレイが見られる

> **試合終了画面で「リプレイを保存」を押さないとアップロードされません**。これは「保存したい試合だけ送る」設計です。

`rl_uploader.exe` のウィンドウは Rocket League をプレイ中はずっと開いたままにしておいてください。閉じると自動アップロードも止まります。

---

## うまく動かないとき

### `rl_uploader.exe` をダブルクリックしても黒い画面が一瞬で消える
- 同じフォルダに `.env` ファイルがない、または `BALLCHASING_TOKEN=` の行が空
- → 手順 4 を見直してください(`.env.example` を `.env` にリネームし忘れていないか、リネーム後も `.env.example.example` のような名前になっていないか)

### 試合終了してもアップロードされない
- Rocket League の `DefaultStatsAPI.ini` の `PacketSendRate` が `0` のまま
  - → 手順 2 を見直してから Rocket League を**再起動**
- 試合終了画面で「リプレイを保存」を押し忘れた
  - → 押し忘れた試合はアップロードできません(リプレイファイル自体が作られないため)

### ログに `[Upload] failed: HTTP 401` と出る
- ballchasing.com の Token が間違っている
- → 手順 1 をやり直して Token を貼り直し、`rl_uploader.exe` を再起動

### ログはどこ?
- `rl_uploader.exe` と同じフォルダの `logs\rl_uploader.log.<日付>` に出ています
- 質問するときはこのログの該当部分を一緒に貼ってください

---

## 自動アップデート

起動時に最新版を確認し、新しいバージョンがあれば**自動でダウンロードして置き換え**ます。アップデート後はプロセスが終了するので、`rl_uploader.exe` を再度ダブルクリックして起動し直してください。

オフラインで動かしたい・チェックを止めたい場合は `.env` に `SKIP_UPDATE=1` を追加。

---

## 詳細設定(オプション)

`.env` で以下の項目を調整できます。

| 変数 | デフォルト | 説明 |
|---|---|---|
| `BALLCHASING_TOKEN` | (必須) | ballchasing API トークン |
| `BALLCHASING_VISIBILITY` | `private` | 公開設定: `public` / `unlisted` / `private` |
| `BALLCHASING_GROUP` | (空) | アップロード先のグループ ID(ballchasing 上で作成) |
| `RLS_TCP_ADDR` | `127.0.0.1:49123` | Rocket League の Stats API 接続先 |
| `WATCH_TIMEOUT_SECS` | `300` | 試合終了後にリプレイ検出を待つ最大秒数 |
| `SKIP_UPDATE` | (空) | 任意の値で起動時の自動アップデート確認をスキップ |

---

## 仕組み(技術的な解説)

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

- `listener` は TCP **クライアント**として Stats API に接続します。サーバー側は Rocket League 自身。
- `watcher` は `MatchEnded - 5s` をベースラインに 2 秒ごとにポーリングし、新規 `.replay` を `created()`(なければ `modified()`)で判定。
- 重複検出は ballchasing 側に委譲(同じファイルなら HTTP 409 が返る)。

ログ(`logs/rl_uploader.log.<UTC日付>`、日次ローテーション)で注目すべきイベント:

| イベント | 意味 |
|---|---|
| `connecting to ...` / `connected` | Stats API への接続成功 |
| `[MatchEnded]` | 試合終了を受信 |
| `[Upload] found: <path>` | リプレイファイルを検出 |
| `[Upload] success id=<uuid>` | アップロード成功 |
| `[Upload] failed: ...` | アップロード失敗 |
| `[Upload] watcher error: timed out` | `WATCH_TIMEOUT_SECS` 内に手動セーブされなかった(設計通り) |

---

## 開発・メンテナ向け

### ソースからビルド

```sh
cargo build --release
./target/release/rl_uploader
```

`cargo check` で型チェックのみ。ログレベルは `src/main.rs` の `with_max_level(...)` で変更可。

### リリースの作り方

タグ push のみでリリースが自動公開されます:

```sh
git tag v0.1.0
git push origin v0.1.0
```

`.github/workflows/release.yml` が:
1. Windows 上で `cargo build --release`
2. `rl_uploader-vX.Y.Z-x86_64-pc-windows-msvc.zip` にパッケージ
3. GitHub Release を作成し zip を添付

`Cargo.toml` の `version` とタグの `vX.Y.Z` を一致させてください(`self_update` は `cargo_crate_version!` で現バージョンを判定するため)。

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

`rl_bc_uploader` is a small always-on Windows binary that listens to Rocket League's built-in Stats API over TCP, detects `MatchEnded`, waits for the player's manually-saved `.replay` to land in `Documents\My Games\Rocket League\TAGame\Demos`, and uploads it to [ballchasing.com](https://ballchasing.com).

**No BakkesMod required** — uses RL's official Stats API directly. **Self-updates** from GitHub Releases on startup.

### Quick start (gamers)

1. Get your API token from https://ballchasing.com/upload.
2. Edit `<RL Install Dir>\TAGame\Config\DefaultStatsAPI.ini` *before* launching RL:
   ```ini
   PacketSendRate=30
   Port=49123
   ```
   `PacketSendRate=0` (the default) leaves the TCP server closed.
3. Download `rl_uploader-vX.Y.Z-x86_64-pc-windows-msvc.zip` from [the latest release](https://github.com/Kazuryu0907/rl_bc_uploader/releases/latest), extract anywhere.
4. Rename the bundled `.env.example` to `.env` and fill in your `BALLCHASING_TOKEN`.
5. Launch RL, then double-click `rl_uploader.exe`. Save replays from the post-match screen — they'll be uploaded automatically.

### Build from source

```sh
cargo build --release
./target/release/rl_uploader
```

### Configuration

All via `.env`. Only `BALLCHASING_TOKEN` is required; see [`.env.example`](./.env.example) for the rest (`BALLCHASING_VISIBILITY`, `BALLCHASING_GROUP`, `RLS_TCP_ADDR`, `WATCH_TIMEOUT_SECS`, `SKIP_UPDATE`).

### License

Dual-licensed under [MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE) at your option.
