use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub body_type: Option<String>, // "json", "form", "text"
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u128,
    pub size_bytes: usize,
}

fn default_timeout() -> u64 {
    30000
}

#[command]
pub async fn send_http_request(request: HttpRequest) -> Result<HttpResponse, String> {
    let client = Client::builder()
        .timeout(Duration::from_millis(request.timeout_ms))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let method = request.method.to_uppercase();
    let mut req_builder = match method.as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        "PATCH" => client.patch(&request.url),
        "HEAD" => client.head(&request.url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &request.url),
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };

    // 添加请求头
    for (key, value) in &request.headers {
        req_builder = req_builder.header(key, value);
    }

    // 添加请求体
    if let Some(body) = &request.body {
        match request.body_type.as_deref() {
            Some("json") => {
                req_builder = req_builder
                    .header("Content-Type", "application/json")
                    .body(body.clone());
            }
            Some("form") => {
                req_builder = req_builder
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(body.clone());
            }
            _ => {
                req_builder = req_builder.body(body.clone());
            }
        }
    }

    let start = std::time::Instant::now();
    let response = req_builder
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let status_text = status.canonical_reason().unwrap_or("").to_string();

    // 收集响应头
    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let time_ms = start.elapsed().as_millis();
    let size_bytes = body.len();

    Ok(HttpResponse {
        status: status.as_u16(),
        status_text,
        headers,
        body,
        time_ms,
        size_bytes,
    })
}
