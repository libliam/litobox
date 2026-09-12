// Ollama 本地大模型管理 - 后端 API proxy
// 纯本地调用 http://localhost:11434，不联网

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, command};

const DEFAULT_HOST: &str = "http://localhost:11434";

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

fn normalize_host(host: &str) -> String {
    let h = host.trim();
    if h.is_empty() {
        DEFAULT_HOST.to_string()
    } else if h.starts_with("http://") || h.starts_with("https://") {
        h.to_string()
    } else {
        format!("http://{}", h)
    }
}

#[derive(Serialize)]
pub struct OllamaVersion {
    pub version: String,
}

/// 检测 Ollama 服务是否可用，返回版本号
#[command]
pub async fn ollama_check(host: Option<String>) -> Result<OllamaVersion, String> {
    let base = normalize_host(&host.unwrap_or_default());
    debug_log!("[ollama] check service at {}", base);
    let url = format!("{}/api/version", base);
    let resp = client()?
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("无法连接 Ollama 服务 ({}): {}", base, e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama 服务返回错误状态: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析版本响应失败: {}", e))?;

    let version = body
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    debug_log!("[ollama] version = {}", version);
    Ok(OllamaVersion { version })
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

#[derive(Serialize)]
pub struct OllamaListResponse {
    pub models: Vec<OllamaModel>,
}

/// 获取已下载的模型列表
#[command]
pub async fn ollama_list(host: Option<String>) -> Result<OllamaListResponse, String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/tags", base);
    debug_log!("[ollama] list models at {}", base);

    let resp = client()?
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求模型列表失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("获取模型列表失败: HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析模型列表失败: {}", e))?;

    let models: Vec<OllamaModel> = body
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| serde_json::from_value::<OllamaModel>(m.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    debug_log!("[ollama] found {} models", models.len());
    Ok(OllamaListResponse { models })
}

/// 获取模型详情
#[command]
pub async fn ollama_show(host: Option<String>, name: String) -> Result<serde_json::Value, String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/show", base);
    debug_log!("[ollama] show model: {}", name);

    let resp = client()?
        .post(&url)
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| format!("请求模型详情失败: {}", e))?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("获取模型详情失败: {}", msg));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析模型详情失败: {}", e))?;

    Ok(body)
}

/// 删除模型
#[command]
pub async fn ollama_delete(host: Option<String>, name: String) -> Result<(), String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/delete", base);
    debug_log!("[ollama] delete model: {}", name);

    let resp = client()?
        .delete(&url)
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| format!("删除模型请求失败: {}", e))?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("删除模型失败: {}", msg));
    }

    debug_log!("[ollama] deleted model: {}", name);
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct RunningModel {
    pub name: String,
    pub size: u64,
    pub digest: String,
    #[serde(default)]
    pub expires_at: String,
    #[serde(default)]
    pub size_vram: u64,
}

#[derive(Serialize)]
pub struct PsResponse {
    pub models: Vec<RunningModel>,
}

/// 获取运行中的模型
#[command]
pub async fn ollama_ps(host: Option<String>) -> Result<PsResponse, String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/ps", base);
    debug_log!("[ollama] ps at {}", base);

    let resp = client()?
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求运行状态失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("获取运行状态失败: HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析运行状态失败: {}", e))?;

    let models: Vec<RunningModel> = body
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| serde_json::from_value::<RunningModel>(m.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    debug_log!("[ollama] {} running models", models.len());
    Ok(PsResponse { models })
}

/// 拉取模型（流式进度推送）
/// 通过事件 "ollama-pull-progress" 推送进度，事件 "ollama-pull-done" 推送完成
#[command]
pub async fn ollama_pull(
    app: AppHandle,
    host: Option<String>,
    name: String,
) -> Result<(), String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/pull", base);
    debug_log!("[ollama] pull model: {}", name);

    let mut resp = client()?
        .post(&url)
        .json(&serde_json::json!({ "name": name, "stream": true }))
        .send()
        .await
        .map_err(|e| format!("拉取模型请求失败: {}", e))?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("拉取模型失败: {}", msg));
    }

    let mut buffer = Vec::new();

    loop {
        match resp.chunk().await {
            Ok(Some(bytes)) => {
                buffer.extend_from_slice(&bytes);
            }
            Ok(None) => break,
            Err(e) => return Err(format!("读取流式响应失败: {}", e)),
        }

        // NDJSON: 按行分割处理
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = buffer.drain(..=pos).collect::<Vec<u8>>();
            let line_str = String::from_utf8_lossy(&line).trim().to_string();
            if line_str.is_empty() {
                continue;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line_str) {
                let status = value.get("status").and_then(|s| s.as_str()).unwrap_or("");
                debug_log!("[ollama] pull status: {}", status);

                let _ = app.emit(
                    "ollama-pull-progress",
                    serde_json::json!({
                        "name": name,
                        "status": status,
                        "total": value.get("total").and_then(|t| t.as_u64()).unwrap_or(0),
                        "completed": value.get("completed").and_then(|c| c.as_u64()).unwrap_or(0),
                        "digest": value.get("digest").and_then(|d| d.as_str()).unwrap_or(""),
                    }),
                );

                if status == "success" {
                    debug_log!("[ollama] pull success: {}", name);
                    let _ = app.emit("ollama-pull-done", serde_json::json!({ "name": name }));
                    return Ok(());
                }
            }
        }
    }

    let _ = app.emit("ollama-pull-done", serde_json::json!({ "name": name }));
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 对话（流式输出）
/// 通过事件 "ollama-chat-chunk" 推送每个 token 块，"ollama-chat-done" 推送完成
#[command]
pub async fn ollama_chat(
    app: AppHandle,
    host: Option<String>,
    model: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/chat", base);
    debug_log!("[ollama] chat with model: {}, messages: {}", model, messages.len());

    let mut resp = client()?
        .post(&url)
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
        }))
        .send()
        .await
        .map_err(|e| format!("对话请求失败: {}", e))?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("对话失败: {}", msg));
    }

    let mut buffer = Vec::new();

    loop {
        match resp.chunk().await {
            Ok(Some(bytes)) => buffer.extend_from_slice(&bytes),
            Ok(None) => break,
            Err(e) => return Err(format!("读取对话流失败: {}", e)),
        }

        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = buffer.drain(..=pos).collect::<Vec<u8>>();
            let line_str = String::from_utf8_lossy(&line).trim().to_string();
            if line_str.is_empty() {
                continue;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line_str) {
                let content = value
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");

                if !content.is_empty() {
                    let _ = app.emit(
                        "ollama-chat-chunk",
                        serde_json::json!({
                            "model": model,
                            "content": content,
                        }),
                    );
                }

                let done = value.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
                if done {
                    debug_log!("[ollama] chat done");
                    let _ = app.emit("ollama-chat-done", serde_json::json!({ "model": model }));
                    return Ok(());
                }
            }
        }
    }

    let _ = app.emit("ollama-chat-done", serde_json::json!({ "model": model }));
    Ok(())
}

/// 停止正在运行的模型（释放显存）
#[command]
pub async fn ollama_stop(host: Option<String>, name: String) -> Result<(), String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/generate", base);
    debug_log!("[ollama] stop model: {}", name);

    // Ollama 通过 generate 接口传入空 prompt + keep_alive: 0 来停止模型
    let resp = client()?
        .post(&url)
        .json(&serde_json::json!({
            "model": name,
            "prompt": "",
            "keep_alive": "0",
            "stream": false,
        }))
        .send()
        .await
        .map_err(|e| format!("停止模型请求失败: {}", e))?;

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("停止模型失败: {}", msg));
    }

    debug_log!("[ollama] stopped model: {}", name);
    Ok(())
}
