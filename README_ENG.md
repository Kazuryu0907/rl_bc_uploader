# rl_bc_uploader

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> 日本語版: [README.md](./README.md)

`rl_bc_uploader` is a small always-on Windows binary that listens to Rocket League's built-in Stats API over TCP, detects `MatchEnded`, waits for the player's manually-saved `.replay` to land in `Documents\My Games\Rocket League\TAGame\Demos`, and uploads it to [ballchasing.com](https://ballchasing.com).

**No BakkesMod required** — uses RL's official Stats API directly. **Self-updates** from GitHub Releases on startup.

---

## Quick start (gamers)

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

---

## Build from source

```sh
cargo build --release
./target/release/rl_uploader
```

---

## Configuration

All via `.env`. Only `BALLCHASING_TOKEN` is required; see [`.env.example`](./.env.example) for the rest (`BALLCHASING_VISIBILITY`, `BALLCHASING_GROUP`, `RLS_TCP_ADDR`, `WATCH_TIMEOUT_SECS`, `SKIP_UPDATE`).

---

## License

Dual-licensed under [MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE) at your option.
