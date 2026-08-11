//! 本地静态文件服务器：手写极简 HTTP/1.1 服务（零额外依赖）
//! 功能：目录浏览、文件下载、目录 ZIP 打包、网页上传、访问日志
//! ponytail: 每连接一个线程 + 文件读入内存响应（大文件用 io::copy 流式），
//! 没有连接池/并发上限，本地工具场景足够；升级路径是 tiny_http / axum。
use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::write::DeflateEncoder;
use flate2::Compression;
use flate2::Crc;
use serde::Serialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_LOG: usize = 500;
const MAX_ZIP_FILE: u64 = 100 * 1024 * 1024; // ZIP 打包单文件上限，防内存爆炸
const MAX_UPLOAD: usize = 500 * 1024 * 1024; // 上传单文件上限

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub time: String,
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub running: bool,
    pub root_dir: String,
    pub port: u16,
    pub urls: Vec<String>,
}

struct ServerHandle {
    root_dir: PathBuf,
    port: u16,
    running: Arc<AtomicBool>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
    join: Option<thread::JoinHandle<()>>,
}

static SERVER: OnceLock<Mutex<Option<ServerHandle>>> = OnceLock::new();

fn server_state() -> &'static Mutex<Option<ServerHandle>> {
    SERVER.get_or_init(|| Mutex::new(None))
}

/// 收集本机所有非回环 IPv4 地址（用于展示局域网访问地址）
fn local_ipv4s() -> Vec<String> {
    use sysinfo::Networks;
    let mut ips: Vec<String> = Vec::new();
    let networks = Networks::new_with_refreshed_list();
    for (_, data) in &networks {
        for ip in data.ip_networks() {
            if let std::net::IpAddr::V4(v4) = ip.addr {
                if !v4.is_loopback() {
                    ips.push(v4.to_string());
                }
            }
        }
    }
    ips.sort();
    ips.dedup();
    ips
}

fn build_urls(port: u16) -> Vec<String> {
    let mut urls = vec![format!("http://127.0.0.1:{}/", port)];
    for ip in local_ipv4s() {
        urls.push(format!("http://{}:{}/", ip, port));
    }
    urls
}

fn stop_inner() {
    let mut guard = server_state().lock().unwrap();
    if let Some(h) = guard.take() {
        h.running.store(false, Ordering::Relaxed);
        if let Some(j) = h.join {
            let _ = j.join(); // accept_loop 会在 ~60ms 内退出
        }
    }
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn http_server_start(root_dir: String, port: u16) -> Result<ServerInfo, String> {
    let root = PathBuf::from(&root_dir);
    if !root.is_dir() {
        return Err("服务目录不存在".into());
    }
    stop_inner();

    // 端口自动避让：优先指定端口，被占用则 +1，最多探测 20 个
    let mut p = if port == 0 { 8000 } else { port };
    let listener = loop {
        match TcpListener::bind(("0.0.0.0", p)) {
            Ok(l) => break l,
            Err(_) => {
                if p >= port.saturating_add(20) {
                    return Err("未找到空闲端口".into());
                }
                p += 1;
            }
        }
    };
    if listener.set_nonblocking(true).is_err() {
        return Err("设置监听失败".into());
    }
    debug_log!("静态服务器启动: root={} port={}", root_dir, p);

    let running = Arc::new(AtomicBool::new(true));
    let logs: Arc<Mutex<Vec<LogEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let root2 = root.clone();
    let running2 = running.clone();
    let logs2 = logs.clone();
    let join = thread::spawn(move || accept_loop(listener, root2, running2, logs2));

    let info = ServerInfo {
        running: true,
        root_dir,
        port: p,
        urls: build_urls(p),
    };
    let mut guard = server_state().lock().unwrap();
    *guard = Some(ServerHandle {
        root_dir: root,
        port: p,
        running,
        logs,
        join: Some(join),
    });
    Ok(info)
}

#[tauri::command]
pub fn http_server_stop() -> Result<(), String> {
    stop_inner();
    debug_log!("静态服务器已停止");
    Ok(())
}

#[tauri::command]
pub fn http_server_status() -> ServerInfo {
    let guard = server_state().lock().unwrap();
    match &*guard {
        Some(h) => ServerInfo {
            running: h.running.load(Ordering::Relaxed),
            root_dir: h.root_dir.to_string_lossy().to_string(),
            port: h.port,
            urls: build_urls(h.port),
        },
        None => ServerInfo {
            running: false,
            root_dir: String::new(),
            port: 0,
            urls: vec![],
        },
    }
}

#[tauri::command]
pub fn http_server_logs() -> Vec<LogEntry> {
    let guard = server_state().lock().unwrap();
    match &*guard {
        Some(h) => h.logs.lock().unwrap().clone(),
        None => vec![],
    }
}

#[tauri::command]
pub fn http_server_clear_logs() -> Result<(), String> {
    let guard = server_state().lock().unwrap();
    if let Some(h) = &*guard {
        h.logs.lock().unwrap().clear();
    }
    Ok(())
}

/// 在系统默认浏览器中打开 URL
#[tauri::command]
pub fn http_open_url(url: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .creation_flags(0x08000000)
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Ok(())
}

// ============ 服务器核心 ============

fn accept_loop(listener: TcpListener, root: PathBuf, running: Arc<AtomicBool>, logs: Arc<Mutex<Vec<LogEntry>>>) {
    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let root = root.clone();
                let logs = logs.clone();
                thread::spawn(move || handle_client(stream, &root, &logs));
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(60));
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
    debug_log!("静态服务器 accept 循环退出");
}

fn handle_client(stream: TcpStream, root: &Path, logs: &Mutex<Vec<LogEntry>>) {
    let client_ip = stream
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_default();
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    let read_half = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(read_half);
    let mut write_half = stream;

    // 支持 keep-alive：单连接最多处理 50 个请求
    for _ in 0..50 {
        let mut req_line = String::new();
        if reader.read_line(&mut req_line).is_err() {
            break;
        }
        let trimmed = req_line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue; // keep-alive 间隔空行
        }
        // 解析请求头
        let mut content_length = 0usize;
        let mut keep_alive = true;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
            let l = line.trim_end_matches(['\r', '\n']);
            if l.is_empty() {
                break;
            }
            let lower = l.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            }
            if lower.starts_with("connection:") && lower.contains("close") {
                keep_alive = false;
            }
        }
        // 读请求体
        let mut body = vec![0u8; content_length.min(MAX_UPLOAD)];
        if content_length > 0 {
            if reader.read_exact(&mut body).is_err() {
                break;
            }
        }
        let mut parts = trimmed.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let target = parts.next().unwrap_or("/").to_string();
        let status;
        let resp = route(&method, &target, &body, root);
        status = match &resp {
            Response::Bytes(s, _, _, _) => *s,
            Response::File(s, _, _, _) => *s,
        };
        let ok = write_response(&mut write_half, &method, &resp, keep_alive);
        // 记录访问日志
        let now = SystemTime::now();
        let log = LogEntry {
            time: format_systemtime(now),
            ip: client_ip.clone(),
            method: method.clone(),
            path: target.clone(),
            status,
        };
        let mut lguard = logs.lock().unwrap();
        lguard.push(log);
        if lguard.len() > MAX_LOG {
            let drain = lguard.len() - MAX_LOG;
            lguard.drain(0..drain);
        }
        drop(lguard);
        if !ok || !keep_alive {
            break;
        }
    }
}

enum Response {
    Bytes(u16, Vec<u8>, String, Vec<(String, String)>),
    File(u16, PathBuf, String, Vec<(String, String)>),
}

fn error(status: u16, msg: &str) -> Response {
    Response::Bytes(
        status,
        format!("{} {}", status, msg).into_bytes(),
        "text/plain; charset=utf-8".into(),
        vec![],
    )
}

fn status_text(s: u16) -> &'static str {
    match s {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn route(method: &str, target: &str, body: &[u8], root: &Path) -> Response {
    if method == "OPTIONS" {
        return Response::Bytes(
            200,
            Vec::new(),
            "text/plain; charset=utf-8".into(),
            vec![
                ("Access-Control-Allow-Methods".into(), "GET, POST, OPTIONS".into()),
                ("Access-Control-Allow-Headers".into(), "Content-Type".into()),
                ("Access-Control-Max-Age".into(), "3600".into()),
            ],
        );
    }
    if method != "GET" && method != "HEAD" && method != "POST" {
        return error(405, "Method Not Allowed");
    }
    let (path_part, query) = match target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (target, ""),
    };
    let decoded_path = percent_decode(path_part);

    // 上传接口
    if decoded_path == "/api/upload" {
        if method != "POST" {
            return error(405, "Method Not Allowed");
        }
        return handle_upload(body, root);
    }
    // 目录打包 ZIP 下载
    if decoded_path == "/zip" {
        let rel = query_param(query, "path").map(|v| percent_decode(v)).unwrap_or_default();
        return handle_zip(&rel, root);
    }
    // 强制下载
    if decoded_path == "/download" {
        let rel = query_param(query, "path").map(|v| percent_decode(v)).unwrap_or_default();
        let target = match resolve_path(&rel, root) {
            Ok(t) => t,
            Err(_) => return error(404, "Not Found"),
        };
        if !target.is_file() {
            return error(404, "Not Found");
        }
        let name = target
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "download".into());
        let disp = format!("attachment; filename=\"{}\"", name);
        let mime = mime_for(&target).to_string();
        return Response::File(200, target, mime, vec![("Content-Disposition".into(), disp)]);
    }

    // 静态资源：目录 → 列表页；文件 → 内联返回
    let target = match resolve_path(decoded_path.trim_start_matches('/'), root) {
        Ok(t) => t,
        Err(_) => return error(403, "Forbidden"),
    };
    if target.is_dir() {
        match dir_listing(&target, root) {
            Ok(html) => Response::Bytes(200, html.into_bytes(), "text/html; charset=utf-8".into(), vec![]),
            Err(_) => error(500, "读取目录失败"),
        }
    } else if target.is_file() {
        let mime = mime_for(&target).to_string();
        Response::File(200, target, mime, vec![])
    } else {
        error(404, "Not Found")
    }
}

fn write_response(stream: &mut TcpStream, method: &str, resp: &Response, keep_alive: bool) -> bool {
    let conn = if keep_alive { "keep-alive" } else { "close" };
    let (status, ctype, headers) = match resp {
        Response::Bytes(s, _, c, h) => (*s, c.clone(), h.clone()),
        Response::File(s, _, c, h) => (*s, c.clone(), h.clone()),
    };
    let len = match resp {
        Response::Bytes(_, b, _, _) => b.len() as u64,
        Response::File(_, p, _, _) => std::fs::metadata(p).map(|m| m.len()).unwrap_or(0),
    };
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: {}\r\n",
        status, status_text(status), ctype, len, conn
    );
    for (k, v) in &headers {
        head.push_str(&format!("{}: {}\r\n", k, v));
    }
    head.push_str("\r\n");
    if stream.write_all(head.as_bytes()).is_err() {
        return false;
    }
    match resp {
        Response::Bytes(_, b, _, _) => {
            if method != "HEAD" && !b.is_empty() && stream.write_all(b).is_err() {
                return false;
            }
        }
        Response::File(_, p, _, _) => {
            if method != "HEAD" {
                if let Ok(mut f) = std::fs::File::open(p) {
                    if std::io::copy(&mut f, stream).is_err() {
                        return false;
                    }
                }
            }
        }
    }
    stream.flush().ok();
    true
}

/// 上传：body 为 JSON { path, name, data(base64) }，保存到 root/path/name
fn handle_upload(body: &[u8], root: &Path) -> Response {
    let parsed: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return error(400, "请求体不是合法 JSON"),
    };
    let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let data = parsed.get("data").and_then(|v| v.as_str()).unwrap_or("");
    let upath = parsed.get("path").and_then(|v| v.as_str()).unwrap_or("/");
    if name.is_empty() || data.is_empty() {
        return error(400, "缺少 name 或 data");
    }
    // 仅取文件名成分，防目录穿越
    let fname = Path::new(name)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string();
    if fname.is_empty() || fname == "." || fname == ".." {
        return error(400, "非法文件名");
    }
    let mut target = match resolve_path(upath, root) {
        Ok(t) => t,
        Err(_) => return error(403, "非法上传路径"),
    };
    target.push(&fname);
    let bytes = match STANDARD.decode(data) {
        Ok(b) => b,
        Err(_) => return error(400, "Base64 解码失败"),
    };
    if bytes.len() > MAX_UPLOAD {
        return error(413, "文件超过 500MB 上限");
    }
    if let Some(parent) = target.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return error(500, "创建目录失败");
        }
    }
    if std::fs::write(&target, &bytes).is_err() {
        return error(500, "写入文件失败");
    }
    debug_log!("静态服务器收到上传: {}", target.to_string_lossy());
    Response::Bytes(200, b"{\"ok\":true}".to_vec(), "application/json".into(), vec![])
}

/// 目录打包为 ZIP 下载
fn handle_zip(rel: &str, root: &Path) -> Response {
    let target = match resolve_path(rel, root) {
        Ok(t) => t,
        Err(_) => return error(404, "Not Found"),
    };
    if !target.is_dir() {
        return error(400, "目标不是目录");
    }
    match build_zip(&target) {
        Ok(bytes) => {
            let name = target
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "archive".into());
            let disp = format!("attachment; filename=\"{}.zip\"", name);
            Response::Bytes(200, bytes, "application/zip".into(), vec![("Content-Disposition".into(), disp)])
        }
        Err(e) => error(500, &e),
    }
}

// ============ 路径与编解码 ============

/// 解析 URL 路径为 root 内安全路径，`..` 弹栈但永不逃逸出 root
fn resolve_path(rel: &str, root: &Path) -> Result<PathBuf, ()> {
    let mut safe = PathBuf::new();
    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => safe.push(c),
            Component::ParentDir => {
                if safe.file_name().is_some() {
                    safe.pop();
                }
            }
            _ => {}
        }
    }
    Ok(root.join(safe))
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| {
        let (k, v) = match kv.split_once('=') {
            Some((k, v)) => (k, v),
            None => (kv, ""),
        };
        if k == key {
            Some(v)
        } else {
            None
        }
    })
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hex = std::str::from_utf8(&b[i + 1..i + 3]).ok();
            if let Some(h) = hex {
                if let Ok(v) = u8::from_str_radix(h, 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn mime_for(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "txt" => "text/plain; charset=utf-8",
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "xml" => "application/xml",
        "yaml" | "yml" => "text/yaml; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "wasm" => "application/wasm",
        "webmanifest" => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

// ============ 目录列表页 ============

fn dir_listing(target: &Path, root: &Path) -> Result<String, std::io::Error> {
    let mut entries: Vec<(bool, String, u64, u64)> = Vec::new(); // (is_dir, name, size, mtime_secs)
    for e in std::fs::read_dir(target)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().to_string();
        let is_dir = e.file_type()?.is_dir();
        let md = e.metadata().ok();
        let size = md.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = md
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        entries.push((is_dir, name, size, mtime));
    }
    // 目录优先，再按名称排序（Windows 风格不区分大小写）
    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));

    let rel = target
        .strip_prefix(root)
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    let display_path = if rel.is_empty() { "/".to_string() } else { format!("/{}", rel) };
    let target_name = target
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "根目录".into());
    let parent_rel = match Path::new(&rel).parent() {
        Some(p) => p.to_string_lossy().replace('\\', "/"),
        None => String::new(),
    };
    let parent_href = if parent_rel.is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", parent_rel)
    };

    let mut rows = String::new();
    // 上级目录
    if !rel.is_empty() {
        rows.push_str(&format!(
            "<tr class=\"parent\"><td><a href=\"{}\">.. （返回上级）</a></td><td></td><td></td><td></td></tr>",
            percent_encode(&parent_href)
        ));
    }
    for (is_dir, name, size, mtime) in &entries {
        let base = if display_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", display_path, name)
        };
        let href = if *is_dir { format!("{}/", base) } else { base.clone() };
        let href_enc = percent_encode(&href);
        let zip_path = percent_encode(&base);
        let size_str = if *is_dir { "-".to_string() } else { fmt_size(*size) };
        let time_str = format_systemtime(UNIX_EPOCH + Duration::from_secs(*mtime));
        if *is_dir {
            rows.push_str(&format!(
                "<tr><td class=\"name\">📁 <a href=\"{}\">{}</a></td><td>{}</td><td>{}</td><td><a class=\"act\" href=\"/zip?path={}\">打包ZIP</a></td></tr>",
                href_enc,
                html_escape(name),
                size_str,
                time_str,
                zip_path
            ));
        } else {
            rows.push_str(&format!(
                "<tr><td class=\"name\">📄 <a href=\"{}\">{}</a></td><td>{}</td><td>{}</td><td><a class=\"act\" href=\"/download?path={}\">下载</a></td></tr>",
                href_enc,
                html_escape(name),
                size_str,
                time_str,
                zip_path
            ));
        }
    }

    let crumb = html_escape(&display_path);
    let upload_path = html_escape(&display_path);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>浏览 - {title}</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:"Segoe UI","Microsoft YaHei",sans-serif;background:#f5f6f8;color:#333;padding:24px;max-width:960px;margin:0 auto}}
h1{{font-size:18px;margin-bottom:16px;color:#0d9488;word-break:break-all}}
.crumb{{font-size:13px;color:#888;margin-bottom:12px}}
.upload{{margin-bottom:16px;display:flex;gap:8px;align-items:center;flex-wrap:wrap}}
.upload input[type=file]{{font-size:13px}}
button{{background:#0d9488;color:#fff;border:none;padding:6px 14px;border-radius:4px;cursor:pointer;font-size:13px}}
button:hover{{background:#0f766e}}
table{{width:100%;border-collapse:collapse;background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 1px 4px rgba(0,0,0,.08)}}
th{{background:#eef2f7;text-align:left;font-size:12px;color:#666;padding:8px 12px}}
td{{padding:8px 12px;font-size:13px;border-top:1px solid #f0f0f0}}
tr:hover{{background:#f8fafc}}
a{{color:#0d9488;text-decoration:none}}
a:hover{{text-decoration:underline}}
.name a{{color:#1f2937}}
td.size{{color:#888;white-space:nowrap}}
td.time{{color:#aaa;white-space:nowrap;font-size:12px}}
td.act a{{font-size:12px}}
.empty{{color:#aaa;padding:24px;text-align:center}}
.msg{{font-size:13px;padding:4px 10px;border-radius:4px}}
.msg.ok{{background:#d1fae5;color:#065f46}}
.msg.err{{background:#fee2e2;color:#991b1b}}
</style></head><body>
<h1>📂 {title}</h1>
<div class="crumb">当前路径: {crumb}</div>
<div class="upload">
  <input type="file" id="file" multiple>
  <button onclick="doUpload()">上传到当前目录</button>
  <span id="msg"></span>
</div>
<table>
<tr><th style="width:50%">名称</th><th style="width:90px">大小</th><th style="width:160px">修改时间</th><th style="width:90px">操作</th></tr>
{rows}
</table>
<script>
async function doUpload(){{
  const files=document.getElementById('file').files;
  const msg=document.getElementById('msg');
  if(!files.length){{msg.className='msg err';msg.textContent='请先选择文件';return}}
  const path={upload_path_json};
  for(const f of files){{
    msg.className='msg';msg.textContent='上传 '+f.name+' ...';
    try{{
      const b64=await new Promise((res,rej)=>{{
        const r=new FileReader();
        r.onload=()=>res(String(r.result).split(',')[1]);
        r.onerror=rej;
        r.readAsDataURL(f);
      }});
      const resp=await fetch('/api/upload',{{
        method:'POST',
        headers:{{'Content-Type':'application/json'}},
        body:JSON.stringify({{path,name:f.name,data:b64}})
      }});
      const j=await resp.json();
      if(j.ok){{msg.className='msg ok';msg.textContent='已上传 '+f.name}}
      else{{msg.className='msg err';msg.textContent='失败: '+(j.error||'未知错误')+' ('+f.name+')';return}}
    }}catch(e){{msg.className='msg err';msg.textContent='上传失败: '+e.message;return}}
  }}
  setTimeout(()=>location.reload(),600);
}}
</script>
</body></html>"#,
        title = html_escape(&target_name),
        crumb = crumb,
        rows = rows,
        upload_path_json = serde_json::to_string(&upload_path).unwrap_or_else(|_| "\"/\"".into()),
    );
    Ok(html)
}

fn fmt_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let b = bytes as f64;
    if b >= KB * KB * KB {
        format!("{:.2} GB", b / (KB * KB * KB))
    } else if b >= KB * KB {
        format!("{:.2} MB", b / (KB * KB))
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

// ============ ZIP 打包（手写格式 + flate2 deflate） ============

fn collect_files(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>, skipped: &mut usize) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_files(&p, base, out, skipped);
            } else if let Ok(md) = e.metadata() {
                if md.len() > MAX_ZIP_FILE {
                    *skipped += 1;
                    continue;
                }
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, p));
            }
        }
    }
}

fn build_zip(dir: &Path) -> Result<Vec<u8>, String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut skipped = 0usize;
    collect_files(dir, dir, &mut files, &mut skipped);
    if files.is_empty() {
        return Err(if skipped > 0 {
            "目录内没有可打包的文件（超过 100MB 的文件已跳过）".into()
        } else {
            "目录为空".into()
        });
    }
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    let mut offset: u32 = 0;
    for (rel, path) in &files {
        let data = std::fs::read(path).map_err(|e| format!("读取 {} 失败: {}", rel, e))?;
        let name_bytes = rel.as_bytes();
        // deflate 压缩
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&data).map_err(|e| e.to_string())?;
        let comp = enc.finish().map_err(|e| e.to_string())?;
        let mut crc = Crc::new();
        crc.update(&data);
        let crc_val = crc.sum();
        let mt = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
            .unwrap_or(0);
        let (dos_time, dos_date) = dos_time_date(mt);
        // local file header
        out.extend_from_slice(b"PK\x03\x04");
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // utf8 文件名
        out.extend_from_slice(&8u16.to_le_bytes()); // deflate
        out.extend_from_slice(&dos_time.to_le_bytes());
        out.extend_from_slice(&dos_date.to_le_bytes());
        out.extend_from_slice(&crc_val.to_le_bytes());
        out.extend_from_slice(&(comp.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(&comp);
        // central directory header
        central.extend_from_slice(b"PK\x01\x02");
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0x0800u16.to_le_bytes());
        central.extend_from_slice(&8u16.to_le_bytes());
        central.extend_from_slice(&dos_time.to_le_bytes());
        central.extend_from_slice(&dos_date.to_le_bytes());
        central.extend_from_slice(&crc_val.to_le_bytes());
        central.extend_from_slice(&(comp.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name_bytes);
        offset += 30 + name_bytes.len() as u32 + comp.len() as u32;
    }
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&central);
    let cd_size = central.len() as u32;
    let count = files.len() as u16;
    // end of central directory
    out.extend_from_slice(b"PK\x05\x06");
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(out)
}

fn dos_time_date(secs: u64) -> (u16, u16) {
    let (y, mo, d, h, mi, s) = utc_ymd_hms(secs);
    let date = if (1980..=2107).contains(&y) {
        (((y - 1980) as u16) << 9) | ((mo as u16) << 5) | (d as u16)
    } else {
        0x0021 // 1980-01-01
    };
    let time = ((h as u16) << 11) | ((mi as u16) << 5) | ((s / 2) as u16);
    (time, date)
}

/// UNIX 秒 → UTC (y, m, d, h, mi, s)，Howard Hinnant civil_from_days 算法
fn utc_ymd_hms(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let s = (rem % 60) as u32;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y } as u32;
    (y, m, d, h, mi, s)
}

/// 格式化 SystemTime 为本地时间：YYYY-MM-DD HH:MM:SS
#[cfg(windows)]
fn format_systemtime(t: SystemTime) -> String {
    use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows_sys::Win32::Storage::FileSystem::FileTimeToLocalFileTime;
    use windows_sys::Win32::System::Time::FileTimeToSystemTime;

    let dur = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let intervals = dur.as_nanos() as u64 / 100 + 116_444_736_000_000_000;
    let utc_ft = FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    };
    let mut local_ft = utc_ft;
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        FileTimeToLocalFileTime(&utc_ft, &mut local_ft);
        FileTimeToSystemTime(&local_ft, &mut st);
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    )
}

#[cfg(not(windows))]
fn format_systemtime(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, mo, d, h, mi, s) = utc_ymd_hms(secs);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn percent_roundtrip() {
        // 中文 + 空格 + 特殊字符编解码往返一致
        let s = "测试 文件-1.5~.txt";
        assert_eq!(percent_decode(&percent_encode(s)), s);
        assert_eq!(percent_decode("%E6%B5%8B"), "测");
        assert_eq!(percent_decode("abc%2Fdef"), "abc/def");
        assert_eq!(percent_decode("a%2"), "a%2"); // 非法 % 保留原样
    }

    #[test]
    fn resolve_path_blocks_traversal() {
        let root = Path::new("C:/base");
        // 穿越组件被剥离
        assert_eq!(resolve_path("../secret", root).unwrap(), root.join("secret"));
        assert_eq!(resolve_path("a/../../b", root).unwrap(), root.join("b"));
        assert_eq!(resolve_path("/abs/path", root).unwrap(), root.join("abs/path"));
    }

    #[test]
    fn zip_has_valid_structure() {
        let dir = std::env::temp_dir().join("litobox_zip_test");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.txt"), "hello").unwrap();
        fs::write(dir.join("sub/b.txt"), "world").unwrap();
        let zip = build_zip(&dir).unwrap();
        // 本地文件头 + 中央目录尾（EOCD 签名位于末尾 22 字节前 4 字节）
        assert_eq!(&zip[0..4], b"PK\x03\x04");
        let eocd = zip.len() - 22;
        assert_eq!(&zip[eocd..eocd + 4], b"PK\x05\x06");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dos_date_valid() {
        // 1970 超出 DOS 范围 → 回退到 1980-01-01
        let (_t, d) = dos_time_date(0);
        assert_eq!(d, 0x0021);
        // 2020 年落在合法区间
        let (t, d) = dos_time_date(1_600_000_000);
        assert!((1980..=2107).contains(&((d >> 9) + 1980)));
        assert!(t > 0);
    }
}
