//! 番茄钟：阶段状态机 + 每秒 tick 事件 + 到点系统通知 + 独立置顶透明浮窗
//!
//! 计时线程只在 running 期间存活（每次 start 重新 spawn），
//! 到点后自动切换到下一阶段并保持暂停，等待用户手动开始。
//! 前端（主工具页 / 浮窗）通过监听 pomodoro-tick / pomodoro-finished 事件驱动 UI。

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

const PHASE_IDLE: &str = "idle";
const PHASE_FOCUS: &str = "focus";
const PHASE_SHORT: &str = "short_break";
const PHASE_LONG: &str = "long_break";

const DEFAULT_FOCUS_SECS: u32 = 25 * 60;
const DEFAULT_SHORT_SECS: u32 = 5 * 60;
const DEFAULT_LONG_SECS: u32 = 15 * 60;
const DEFAULT_INTERVAL: u32 = 4;

/// 广播给前端的状态快照（camelCase 便于 JS 直接使用）
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroSnapshot {
    pub phase: String,
    pub running: bool,
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub cycle_focus_done: u32,
    pub focus_secs: u32,
    pub short_break_secs: u32,
    pub long_break_secs: u32,
    pub long_break_interval: u32,
}

pub struct PomodoroInner {
    inner: Mutex<PomodoroData>,
}

struct PomodoroData {
    phase: String,
    running: bool,
    remaining_secs: u32,
    total_secs: u32,
    cycle_focus_done: u32,
    focus_secs: u32,
    short_break_secs: u32,
    long_break_secs: u32,
    long_break_interval: u32,
}

impl PomodoroData {
    fn snapshot(&self) -> PomodoroSnapshot {
        PomodoroSnapshot {
            phase: self.phase.clone(),
            running: self.running,
            remaining_secs: self.remaining_secs,
            total_secs: self.total_secs,
            cycle_focus_done: self.cycle_focus_done,
            focus_secs: self.focus_secs,
            short_break_secs: self.short_break_secs,
            long_break_secs: self.long_break_secs,
            long_break_interval: self.long_break_interval,
        }
    }

    fn phase_secs(&self) -> u32 {
        match self.phase.as_str() {
            PHASE_FOCUS => self.focus_secs,
            PHASE_SHORT => self.short_break_secs,
            PHASE_LONG => self.long_break_secs,
            _ => 0,
        }
    }

    /// 计算下一阶段（不修改状态）
    fn next_phase(&self) -> String {
        match self.phase.as_str() {
            PHASE_FOCUS => {
                // 用"完成后的轮数"判断：完成第 interval 的倍数轮后进入长休息
                let done = self.cycle_focus_done + 1;
                if done % self.long_break_interval.max(1) == 0 {
                    PHASE_LONG.to_string()
                } else {
                    PHASE_SHORT.to_string()
                }
            }
            _ => PHASE_FOCUS.to_string(),
        }
    }
}

/// 从 DB 读取番茄钟时长设置
fn load_settings() -> (u32, u32, u32, u32) {
    match crate::db::db_get_config("pomodoro_settings".to_string()) {
        Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) => {
                let g = |k: &str, d: u32| {
                    v.get(k)
                        .and_then(|x| x.as_u64())
                        .map(|n| n as u32)
                        .unwrap_or(d)
                };
                (
                    g("focusSecs", DEFAULT_FOCUS_SECS),
                    g("shortBreakSecs", DEFAULT_SHORT_SECS),
                    g("longBreakSecs", DEFAULT_LONG_SECS),
                    g("longBreakInterval", DEFAULT_INTERVAL),
                )
            }
            Err(_) => (
                DEFAULT_FOCUS_SECS,
                DEFAULT_SHORT_SECS,
                DEFAULT_LONG_SECS,
                DEFAULT_INTERVAL,
            ),
        },
        Err(_) => (
            DEFAULT_FOCUS_SECS,
            DEFAULT_SHORT_SECS,
            DEFAULT_LONG_SECS,
            DEFAULT_INTERVAL,
        ),
    }
}

/// 主窗口可见时在右下角弹一条系统通知
fn show_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

impl PomodoroInner {
    pub fn new() -> Self {
        let (f, s, l, i) = load_settings();
        PomodoroInner {
            inner: Mutex::new(PomodoroData {
                phase: PHASE_IDLE.to_string(),
                running: false,
                remaining_secs: 0,
                total_secs: 0,
                cycle_focus_done: 0,
                focus_secs: f,
                short_break_secs: s,
                long_break_secs: l,
                long_break_interval: i,
            }),
        }
    }
}

impl Default for PomodoroInner {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取数据锁；即使被 panic 污染（poison）也恢复数据，避免命令静默失效
fn lock_data<'a>(state: &'a State<'_, PomodoroInner>) -> std::sync::MutexGuard<'a, PomodoroData> {
    state.inner.lock().unwrap_or_else(|e| e.into_inner())
}

/// 启动后台计时线程（仅在 running 期间存活）
fn spawn_ticker(app: AppHandle) {
    thread::spawn(move || loop {
        let state = app.state::<PomodoroInner>();
        let mut data = lock_data(&state);
        if !data.running {
            break;
        }
        if data.remaining_secs == 0 {
            // ---- 当前阶段完成 ----
            let finished_phase = data.phase.clone();
            if finished_phase == PHASE_FOCUS {
                data.cycle_focus_done += 1;
            }
            let next = data.next_phase();
            let next_secs = match next.as_str() {
                PHASE_FOCUS => data.focus_secs,
                PHASE_SHORT => data.short_break_secs,
                _ => data.long_break_secs,
            };
            data.phase = next.clone();
            data.remaining_secs = next_secs;
            data.total_secs = next_secs;
            data.running = false;
            let snapshot = data.snapshot();
            drop(data);
            // 通知文案
            let (title, body) = match finished_phase.as_str() {
                PHASE_FOCUS => (
                    "🍅 专注完成！".to_string(),
                    format!("休息 {} 分钟，点击开始继续", next_secs / 60),
                ),
                _ => (
                    "☕ 休息结束".to_string(),
                    format!("开始下一轮专注（第 {} 轮）", snapshot.cycle_focus_done + 1),
                ),
            };
            show_notification(&app, &title, &body);
            let payload = serde_json::json!({
                "finishedPhase": finished_phase,
                "nextPhase": next,
                "snapshot": snapshot,
            });
            let _ = app.emit("pomodoro-finished", &payload);
            break;
        }
        data.remaining_secs -= 1;
        let snapshot = data.snapshot();
        drop(data);
        let _ = app.emit("pomodoro-tick", &snapshot);
        thread::sleep(Duration::from_secs(1));
    });
}

// ==================== 命令 ====================

#[tauri::command]
pub fn pomodoro_state(state: State<PomodoroInner>) -> PomodoroSnapshot {
    lock_data(&state).snapshot()
}

#[tauri::command]
pub fn pomodoro_set_settings(
    app: AppHandle,
    state: State<PomodoroInner>,
    focus_secs: u32,
    short_break_secs: u32,
    long_break_secs: u32,
    long_break_interval: u32,
) -> Result<(), String> {
    let mut data = lock_data(&state);
    data.focus_secs = focus_secs.max(60);
    data.short_break_secs = short_break_secs.max(60);
    data.long_break_secs = long_break_secs.max(60);
    data.long_break_interval = long_break_interval.max(1);
    // idle 时立即应用到当前阶段
    if data.phase == PHASE_IDLE {
        data.remaining_secs = 0;
        data.total_secs = 0;
    }
    let json = serde_json::json!({
        "focusSecs": data.focus_secs,
        "shortBreakSecs": data.short_break_secs,
        "longBreakSecs": data.long_break_secs,
        "longBreakInterval": data.long_break_interval,
    })
    .to_string();
    drop(data);
    crate::db::db_set_config("pomodoro_settings".to_string(), json)?;
    let _ = app.emit("pomodoro-tick", &lock_data(&state).snapshot());
    Ok(())
}

#[tauri::command]
pub fn pomodoro_start(app: AppHandle, state: State<PomodoroInner>) -> Result<(), String> {
    {
        let mut data = state.inner.lock().unwrap();
        if data.phase == PHASE_IDLE {
            data.phase = PHASE_FOCUS.to_string();
            data.remaining_secs = data.focus_secs;
            data.total_secs = data.focus_secs;
        }
        if data.remaining_secs == 0 {
            data.remaining_secs = data.phase_secs();
            data.total_secs = data.remaining_secs;
        }
        data.running = true;
        let snapshot = data.snapshot();
        drop(data);
        debug_log!("[pomodoro] start: phase={} remaining={}s", snapshot.phase, snapshot.remaining_secs);
        let _ = app.emit("pomodoro-tick", &snapshot);
    }
    spawn_ticker(app);
    Ok(())
}

#[tauri::command]
pub fn pomodoro_pause(app: AppHandle, state: State<PomodoroInner>) -> Result<(), String> {
    let mut data = lock_data(&state);
    data.running = false;
    debug_log!("[pomodoro] pause: remaining={}s", data.remaining_secs);
    let snapshot = data.snapshot();
    drop(data);
    let _ = app.emit("pomodoro-tick", &snapshot);
    Ok(())
}

#[tauri::command]
pub fn pomodoro_toggle(app: AppHandle, state: State<PomodoroInner>) -> Result<(), String> {
    debug_log!("[pomodoro] toggle called");
    if lock_data(&state).running {
        pomodoro_pause(app, state)
    } else {
        pomodoro_start(app, state)
    }
}

#[tauri::command]
pub fn pomodoro_skip(app: AppHandle, state: State<PomodoroInner>) -> Result<(), String> {
    {
        let mut data = state.inner.lock().unwrap();
        if data.phase != PHASE_IDLE {
            if data.phase == PHASE_FOCUS {
                data.cycle_focus_done += 1;
            }
            let next = data.next_phase();
            let next_secs = match next.as_str() {
                PHASE_FOCUS => data.focus_secs,
                PHASE_SHORT => data.short_break_secs,
                _ => data.long_break_secs,
            };
            data.phase = next;
            data.remaining_secs = next_secs;
            data.total_secs = next_secs;
            data.running = false;
            let snapshot = data.snapshot();
            drop(data);
            let _ = app.emit("pomodoro-tick", &snapshot);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn pomodoro_reset(app: AppHandle, state: State<PomodoroInner>) -> Result<(), String> {
    {
        let mut data = lock_data(&state);
        data.phase = PHASE_IDLE.to_string();
        data.running = false;
        data.remaining_secs = 0;
        data.total_secs = 0;
        data.cycle_focus_done = 0;
        let snapshot = data.snapshot();
        drop(data);
        let _ = app.emit("pomodoro-tick", &snapshot);
    }
    Ok(())
}
