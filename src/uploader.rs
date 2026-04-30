use std::path::Path;
use anyhow::Context;

/// リプレイファイルを ballchasing.com にアップロードする。
/// 成功時は割り当てられたリプレイ ID を返す。
/// 重複の場合は Err("duplicate id=...") を返す。
pub async fn upload(
    path: &Path,
    token: &str,
    visibility: &str,
    group: Option<&str>,
) -> anyhow::Result<String> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("replay.replay")
        .to_string();

    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("reading {}", path.display()))?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str("application/octet-stream")?;

    let form = reqwest::multipart::Form::new().part("file", part);

    let mut query: Vec<(&str, &str)> = vec![("visibility", visibility)];
    if let Some(g) = group {
        query.push(("group", g));
    }

    let client = reqwest::Client::new();
    let resp = client
        .post("https://ballchasing.com/api/v2/upload")
        .query(&query)
        .header("Authorization", token)
        .multipart(form)
        .send()
        .await
        .context("sending upload request to ballchasing")?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.as_u16() == 201 {
        let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        let id = v["id"].as_str().unwrap_or("?").to_string();
        Ok(id)
    } else if status.as_u16() == 409 {
        let v: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        let id = v["id"].as_str().unwrap_or("?").to_string();
        anyhow::bail!("duplicate replay (existing id={})", id)
    } else {
        anyhow::bail!("upload failed: HTTP {} — {}", status, body)
    }
}
