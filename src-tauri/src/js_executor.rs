use boa_engine::{Context, JsValue, NativeFunction, Source, js_string};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub result: Option<String>,
    pub logs: Vec<LogEntry>,
    pub error: Option<String>,
    pub line: Option<usize>,
}

#[tauri::command]
pub fn execute_js(code: String, input: String, timeout_ms: u64) -> ExecuteResult {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = run_js(&code, &input);
        let _ = tx.send(result);
    });

    match rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(result) => result,
        Err(_) => ExecuteResult {
            success: false,
            result: None,
            logs: vec![],
            error: Some("执行超时（超过 5 秒）".to_string()),
            line: None,
        },
    }
}

fn run_js(code: &str, input_json: &str) -> ExecuteResult {
    let mut context = Context::default();
    let logs: Arc<Mutex<Vec<LogEntry>>> = Arc::new(Mutex::new(Vec::new()));

    // 注入 console 对象 — from_closure is unsafe in boa 0.21
    let logs_clone = Arc::clone(&logs);
    let log_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_clone.lock().unwrap().push(LogEntry {
                level: "log".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        })
    };

    let logs_clone = Arc::clone(&logs);
    let warn_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_clone.lock().unwrap().push(LogEntry {
                level: "warn".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        })
    };

    let logs_clone = Arc::clone(&logs);
    let error_fn = unsafe {
        NativeFunction::from_closure(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_clone.lock().unwrap().push(LogEntry {
                level: "error".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        })
    };

    let console_obj = boa_engine::object::ObjectInitializer::new(&mut context)
        .function(log_fn, js_string!("log"), 0)
        .function(warn_fn, js_string!("warn"), 0)
        .function(error_fn, js_string!("error"), 0)
        .build();

    context
        .register_global_property(
            js_string!("console"),
            console_obj,
            boa_engine::property::Attribute::all(),
        )
        .unwrap();

    // 注入 input 全局变量
    if !input_json.trim().is_empty() {
        match serde_json::from_str::<serde_json::Value>(input_json) {
            Ok(json_val) => {
                let js_val = json_to_js_value(&json_val, &mut context);
                context
                    .register_global_property(
                        js_string!("input"),
                        js_val,
                        boa_engine::property::Attribute::all(),
                    )
                    .unwrap();
            }
            Err(e) => {
                return ExecuteResult {
                    success: false,
                    result: None,
                    logs: logs.lock().unwrap().clone(),
                    error: Some(format!("入参 JSON 格式错误: {}", e)),
                    line: None,
                };
            }
        }
    }

    // 执行代码
    match context.eval(Source::from_bytes(code)) {
        Ok(result) => ExecuteResult {
            success: true,
            result: Some(result.display().to_string()),
            logs: logs.lock().unwrap().clone(),
            error: None,
            line: None,
        },
        Err(e) => {
            let error_msg = e.to_string();
            let line = extract_line_number(&error_msg);
            ExecuteResult {
                success: false,
                result: None,
                logs: logs.lock().unwrap().clone(),
                error: Some(error_msg),
                line,
            }
        }
    }
}

fn json_to_js_value(json: &serde_json::Value, context: &mut Context) -> JsValue {
    match json {
        serde_json::Value::Null => JsValue::null(),
        serde_json::Value::Bool(b) => JsValue::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::from(i)
            } else if let Some(f) = n.as_f64() {
                JsValue::from(f)
            } else {
                JsValue::null()
            }
        }
        serde_json::Value::String(s) => JsValue::from(js_string!(s.as_str())),
        serde_json::Value::Array(arr) => {
            let js_arr = boa_engine::object::builtins::JsArray::new(context);
            for item in arr.iter() {
                let js_item = json_to_js_value(item, context);
                js_arr.push(js_item, context).unwrap();
            }
            js_arr.into()
        }
        serde_json::Value::Object(obj) => {
            // 先转换所有值为 JsValue，避免借用冲突
            let entries: Vec<_> = obj.iter().map(|(key, value)| {
                (key.clone(), json_to_js_value(value, context))
            }).collect();
            
            let mut js_obj = boa_engine::object::ObjectInitializer::new(context);
            for (key, js_value) in entries {
                js_obj.property(
                    js_string!(key.as_str()),
                    js_value,
                    boa_engine::property::Attribute::all(),
                );
            }
            js_obj.build().into()
        }
    }
}

fn extract_line_number(error_msg: &str) -> Option<usize> {
    // boa 错误格式: "ReferenceError: xxx is not defined\n  at <anonymous>:3:1"
    for line in error_msg.lines() {
        if line.contains("at") {
            if let Some(colon_pos) = line.rfind(':') {
                if let Some(prev_pos) = line[..colon_pos].rfind(':') {
                    let line_num_str = &line[prev_pos + 1..colon_pos];
                    if let Ok(num) = line_num_str.trim().parse::<usize>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}
