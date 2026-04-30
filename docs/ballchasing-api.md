# ballchasing.com API

## 概要

Rocket League のリプレイファイルをアップロード・管理・統計分析するための REST API。

- **ベース URL**: `https://ballchasing.com`
- **ドキュメント**: https://ballchasing.com/doc/api

---

## 認証

すべてのリクエストでヘッダーにトークンを付与する:

```
Authorization: <token>
```

トークンはアップロードページで発行。

---

## レート制限

パトロン Tier によって制限が異なる (GC > Champion > Diamond > Gold > Standard の順)。
制限超過時は HTTP `429` が返る。冷却後にリトライする。

---

## エンドポイント

### Ping

| Method | Path | 説明 |
|---|---|---|
| GET | `/` | API キー検証・サービス死活確認 |

---

### リプレイ

#### アップロード

| Method | Path | 説明 |
|---|---|---|
| POST | `/v2/upload` | リプレイファイルをアップロード (`multipart/form-data`) |

クエリパラメータ:

| パラメータ | 値 | 説明 |
|---|---|---|
| `visibility` | `public` / `unlisted` / `private` | 公開設定 |

レスポンス: `201` (成功) / `409` (重複)

---

#### 一覧・検索

| Method | Path | 説明 |
|---|---|---|
| GET | `/replays` | リプレイ一覧・フィルタ検索 |

主なフィルタパラメータ:

| パラメータ | 説明 |
|---|---|
| `player-name` | プレイヤー名 |
| `player-id` | プレイヤー ID |
| `playlist` | プレイリスト |
| `season` | シーズン |
| `rank` | ランク |
| `map` | マップコード (`/maps` で取得) |
| 日時範囲 | 期間指定 |

---

#### 詳細・更新・削除

| Method | Path | 説明 |
|---|---|---|
| GET | `/replays/{id}` | 特定リプレイの詳細統計 |
| PATCH | `/replays/{id}` | タイトル / visibility / グループ割り当てを更新 |
| DELETE | `/replays/{id}` | リプレイを完全削除 (不可逆) |
| GET | `/replays/{id}/file` | リプレイファイルをダウンロード (低レートリミット) |

`GET /replays/{id}` レスポンスには以下の詳細が含まれる:
- ブースト使用状況
- 移動パターン
- ポジショニングデータ
- プレイヤーごとの詳細メトリクス
- 処理中・失敗リプレイはステータスインジケーター付き

---

### グループ

リプレイを階層的にグループ化して集計統計を取得する機能。

| Method | Path | 説明 |
|---|---|---|
| POST | `/groups` | グループ作成 |
| GET | `/groups` | グループ一覧・フィルタ |
| GET | `/groups/{id}` | グループ詳細 + ゲーム横断の集計統計 |
| PATCH | `/groups/{id}` | グループ属性更新 |
| DELETE | `/groups/{id}` | グループ削除 |

`POST /groups` パラメータ:
- プレイヤー/チーム識別ストラテジー
- 階層的なグルーピング設定

---

### Maps

| Method | Path | 説明 |
|---|---|---|
| GET | `/maps` | `/replays` フィルタで使える有効なマップコード一覧 |

---

## レスポンス形式

すべて JSON。
