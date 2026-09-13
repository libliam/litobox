// Ollama 本地大模型管理 - 后端 API proxy
// 纯本地调用 http://localhost:11434，不联网

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, command};

const DEFAULT_HOST: &str = "http://localhost:11434";

fn client() -> Result<Client, String> {
    // ponytail: 关闭系统代理。Ollama 只连 localhost，若 Windows 配置了系统代理（VPN/代理工具），
    // reqwest 默认的 system-proxy 会把 localhost 也路由到代理导致连接失败。Python requests 默认绕过 localhost 代理所以没问题。
    Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

/// 流式响应专用客户端：不设总超时（长回复可能耗时数分钟），仅限制连接超时
fn streaming_client() -> Result<Client, String> {
    Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建流式 HTTP 客户端失败: {}", e))
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

    let mut resp = streaming_client()?
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub code: String,
}

/// 执行 Python 工具代码，参数通过 stdin 传入（JSON），stdout 作为结果返回
fn run_python_tool(code: &str, args: &serde_json::Value) -> Result<String, String> {
    let args_json = serde_json::to_string(args).unwrap_or("{}".to_string());
    // ponytail: 把 params 注入全局作用域，用户代码可直接用参数名或 params["name"]
    let wrapper = format!(
        "import json, sys\nparams = json.loads(sys.stdin.read())\nglobals().update(params)\n{}\n",
        code
    );

    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("python");
    cmd.arg("-c")
        .arg(&wrapper)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 Python 失败（请确认已安装 python）: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(args_json.as_bytes())
            .map_err(|e| format!("写入参数失败: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("等待 Python 执行失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("Python 执行错误:\n{}", stderr));
    }

    Ok(if stdout.is_empty() {
        "(工具无输出)".to_string()
    } else {
        stdout
    })
}

/// 对话（流式输出，支持工具调用）
/// 事件：
///   "ollama-chat-chunk"   - 文本 token 块
///   "ollama-tool-call"    - 工具调用 { name, arguments, result }
///   "ollama-chat-done"    - 对话结束
#[command]
pub async fn ollama_chat(
    app: AppHandle,
    host: Option<String>,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolDef>>,
) -> Result<(), String> {
    let base = normalize_host(&host.unwrap_or_default());
    let url = format!("{}/api/chat", base);
    let tools_ref = tools.unwrap_or_default();
    debug_log!(
        "[ollama] chat model={}, messages={}, tools={}",
        model,
        messages.len(),
        tools_ref.len()
    );

    let mut current_messages = messages;
    let max_tool_rounds = 10; // 防止无限循环

    for round in 0..max_tool_rounds {
        // 构建 Ollama 工具定义（不含 code，code 由后端执行）
        let ollama_tools: Vec<serde_json::Value> = tools_ref
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": current_messages,
            "stream": true,
        });
        if !ollama_tools.is_empty() {
            body["tools"] = serde_json::Value::Array(ollama_tools);
        }

        let mut resp = streaming_client()?
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("对话请求失败: {}", e))?;

        if !resp.status().is_success() {
            let msg = resp.text().await.unwrap_or_default();
            return Err(format!("对话失败: {}", msg));
        }

        // 流式读取，收集 content 和 tool_calls
        let mut buffer = Vec::new();
        let mut full_content = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut has_tool_calls = false;

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
                    let msg_obj = value.get("message").cloned().unwrap_or_default();

                    // 收集文本内容并推送
                    if let Some(content) = msg_obj.get("content").and_then(|c| c.as_str()) {
                        if !content.is_empty() {
                            full_content.push_str(content);
                            let _ = app.emit(
                                "ollama-chat-chunk",
                                serde_json::json!({
                                    "model": model,
                                    "content": content,
                                }),
                            );
                        }
                    }

                    // 收集 tool_calls（可能分散在多个 chunk 中）
                    if let Some(tc) = msg_obj.get("tool_calls") {
                        if let Some(arr) = tc.as_array() {
                            for call in arr {
                                tool_calls.push(call.clone());
                            }
                            has_tool_calls = true;
                        }
                    }

                    let done = value.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
                    if done {
                        break;
                    }
                }
            }
        }

        // 如果没有工具调用，对话结束
        if !has_tool_calls || tool_calls.is_empty() {
            debug_log!("[ollama] chat done (round {})", round);
            let _ = app.emit("ollama-chat-done", serde_json::json!({ "model": model }));
            return Ok(());
        }

        debug_log!(
            "[ollama] tool calls detected (round {}): {} calls",
            round,
            tool_calls.len()
        );

        // 把 assistant 的回复（含 tool_calls）加入消息历史
        current_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: full_content.clone(),
            tool_calls: Some(serde_json::Value::Array(tool_calls.clone())),
            tool_call_id: None,
        });

        // 执行每个工具调用
        for call in &tool_calls {
            let func = call.get("function").cloned().unwrap_or_default();
            let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let arguments = func
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            // 查找工具定义并执行
            let tool_result = if let Some(tool) = tools_ref.iter().find(|t| t.name == name) {
                match run_python_tool(&tool.code, &arguments) {
                    Ok(out) => out,
                    Err(e) => format!("工具执行失败: {}", e),
                }
            } else {
                format!("未找到工具: {}", name)
            };

            let tool_call_id = call
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string();

            // 推送工具调用事件给前端展示
            let _ = app.emit(
                "ollama-tool-call",
                serde_json::json!({
                    "name": name,
                    "arguments": arguments,
                    "result": tool_result,
                }),
            );

            // 工具结果回灌给模型
            current_messages.push(ChatMessage {
                role: "tool".to_string(),
                content: tool_result,
                tool_calls: None,
                tool_call_id: Some(tool_call_id),
            });
        }

        // 继续下一轮循环，让模型基于工具结果生成回复
    }

    let _ = app.emit("ollama-chat-done", serde_json::json!({ "model": model }));
    Ok(())
}

/// 单独执行 Python 工具（供前端测试用）
#[command]
pub async fn ollama_run_tool(code: String, arguments: serde_json::Value) -> Result<String, String> {
    debug_log!("[ollama] run tool with args: {:?}", arguments);
    run_python_tool(&code, &arguments)
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
