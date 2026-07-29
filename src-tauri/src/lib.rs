mod model_manager;
mod providers;

use arboard::Clipboard;
use model_manager::{CustomModelInstallRequest, ManagedModelStatus};
use providers::{
    transcribe, transcribe_custom_model, transcribe_endpoint, translate, warm_whisper,
    CustomTranscriptionRequest, EndpointTranscriptionRequest, EngineStatus, TranscriptionRequest,
    TranscriptionResult, WhisperRuntime,
};
use serde::Serialize;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
#[cfg(desktop)]
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const PUSH_TO_TALK_HOLD_MS: u64 = 280;
const DEFAULT_MODEL_DOWNLOAD_ARGUMENT: &str = "--download-default-model";

#[derive(Default)]
struct TargetWindow(Arc<Mutex<isize>>);

struct PushToTalkShortcut {
    active: Arc<Mutex<Shortcut>>,
    hold: Arc<Mutex<HoldTracker>>,
}

#[derive(Clone)]
struct PushToTalkRuntime {
    app: AppHandle,
    target: Arc<Mutex<isize>>,
    active: Arc<Mutex<Shortcut>>,
    hold: Arc<Mutex<HoldTracker>>,
}

impl Default for PushToTalkShortcut {
    fn default() -> Self {
        Self {
            active: Arc::new(Mutex::new(default_push_to_talk_shortcut())),
            hold: Arc::new(Mutex::new(HoldTracker::default())),
        }
    }
}

#[derive(Default)]
struct HoldTracker {
    generation: u64,
    key_down: bool,
    recording: bool,
}

enum HoldRelease {
    Ignore,
    QuickTap,
    StopRecording,
}

impl HoldTracker {
    fn press(&mut self) -> u64 {
        if self.key_down {
            return self.generation;
        }
        self.generation = self.generation.wrapping_add(1);
        self.key_down = true;
        self.recording = false;
        self.generation
    }

    fn activate(&mut self, generation: u64) -> bool {
        if self.generation != generation || !self.key_down || self.recording {
            return false;
        }
        self.recording = true;
        true
    }

    fn release(&mut self) -> HoldRelease {
        if !self.key_down {
            return HoldRelease::Ignore;
        }
        self.key_down = false;
        self.generation = self.generation.wrapping_add(1);
        if std::mem::take(&mut self.recording) {
            HoldRelease::StopRecording
        } else {
            HoldRelease::QuickTap
        }
    }

    fn reset(&mut self) {
        self.key_down = false;
        self.recording = false;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InsertResult {
    inserted: bool,
    clipboard_restored: bool,
    message: String,
}

#[tauri::command]
fn engine_status() -> Vec<EngineStatus> {
    providers::statuses()
}

#[tauri::command]
fn managed_model_status(model_id: String) -> Result<ManagedModelStatus, String> {
    model_manager::catalog_status(&model_id)
}

#[tauri::command]
fn default_model_download_requested() -> bool {
    std::env::args().any(|argument| argument == DEFAULT_MODEL_DOWNLOAD_ARGUMENT)
}

#[tauri::command]
fn custom_model_status(file_name: String) -> Result<ManagedModelStatus, String> {
    model_manager::custom_status(&file_name)
}

#[tauri::command]
async fn install_managed_model(
    app: tauri::AppHandle,
    model_id: String,
) -> Result<ManagedModelStatus, String> {
    tauri::async_runtime::spawn_blocking(move || model_manager::install_catalog(&app, &model_id))
        .await
        .map_err(|error| format!("The model installation task stopped unexpectedly: {error}"))?
}

#[tauri::command]
async fn install_custom_model(
    app: tauri::AppHandle,
    request: CustomModelInstallRequest,
) -> Result<ManagedModelStatus, String> {
    tauri::async_runtime::spawn_blocking(move || model_manager::install_custom(&app, &request))
        .await
        .map_err(|error| format!("The model installation task stopped unexpectedly: {error}"))?
}

#[tauri::command]
async fn transcribe_audio(request: TranscriptionRequest, app: AppHandle) -> TranscriptionResult {
    let engine = request.engine.clone();
    let duration_ms = request.duration_ms;
    let whisper_runtime = app.state::<WhisperRuntime>().inner().clone();
    match tauri::async_runtime::spawn_blocking(move || transcribe(request, &whisper_runtime)).await
    {
        Ok(result) => result,
        Err(error) => TranscriptionResult {
            engine,
            model: "unknown".into(),
            text: String::new(),
            elapsed_ms: 0,
            audio_duration_ms: duration_ms,
            ok: false,
            error: Some(format!(
                "The local transcription task stopped unexpectedly: {error}"
            )),
        },
    }
}

#[tauri::command]
async fn transcribe_model_endpoint(request: EndpointTranscriptionRequest) -> TranscriptionResult {
    let connection_id = request.connection_id.clone();
    let model = request.model.clone();
    let duration_ms = request.duration_ms;
    match tauri::async_runtime::spawn_blocking(move || transcribe_endpoint(request)).await {
        Ok(result) => result,
        Err(error) => TranscriptionResult {
            engine: connection_id,
            model,
            text: String::new(),
            elapsed_ms: 0,
            audio_duration_ms: duration_ms,
            ok: false,
            error: Some(format!(
                "The endpoint transcription task stopped unexpectedly: {error}"
            )),
        },
    }
}

#[tauri::command]
async fn transcribe_managed_model(request: CustomTranscriptionRequest) -> TranscriptionResult {
    let connection_id = request.connection_id.clone();
    let model = request.file_name.clone();
    let duration_ms = request.duration_ms;
    match tauri::async_runtime::spawn_blocking(move || transcribe_custom_model(request)).await {
        Ok(result) => result,
        Err(error) => TranscriptionResult {
            engine: connection_id,
            model,
            text: String::new(),
            elapsed_ms: 0,
            audio_duration_ms: duration_ms,
            ok: false,
            error: Some(format!(
                "The managed model task stopped unexpectedly: {error}"
            )),
        },
    }
}

#[tauri::command]
async fn translate_text(
    text: String,
    source_language: String,
    target_language: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        translate(&text, &source_language, &target_language)
    })
    .await
    .map_err(|error| format!("The translation task stopped unexpectedly: {error}"))?
}

#[tauri::command]
fn insert_text(text: String, target: State<'_, TargetWindow>) -> InsertResult {
    if text.trim().is_empty() {
        return InsertResult {
            inserted: false,
            clipboard_restored: false,
            message: "There is no transcript to insert.".into(),
        };
    }

    let target_handle = target
        .0
        .lock()
        .map(|stored_handle| *stored_handle)
        .unwrap_or_default();
    match paste_into_target(&text, target_handle) {
        Ok(restored) => InsertResult {
            inserted: true,
            clipboard_restored: restored,
            message: "Transcript inserted into the previous app.".into(),
        },
        Err(error) => InsertResult {
            inserted: false,
            clipboard_restored: false,
            message: error,
        },
    }
}

#[tauri::command]
fn normalize_transcript(text: String) -> String {
    let mut words: Vec<&str> = Vec::new();
    for word in text.split_whitespace() {
        let repeated = words
            .last()
            .is_some_and(|previous| previous.eq_ignore_ascii_case(word));
        if !repeated {
            words.push(word);
        }
    }
    words.join(" ")
}

#[tauri::command]
fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().map_err(|error| error.to_string())? {
        window.unmaximize().map_err(|error| error.to_string())
    } else {
        window.maximize().map_err(|error| error.to_string())
    }
}

#[tauri::command]
fn close_window(app: AppHandle) -> Result<(), String> {
    hide_main_window(&app)
}

#[tauri::command]
fn set_push_to_talk_shortcut(
    shortcut: String,
    app: AppHandle,
    active_shortcut: State<'_, PushToTalkShortcut>,
) -> Result<String, String> {
    let next = parse_push_to_talk_shortcut(&shortcut)?;
    replace_push_to_talk_shortcut(&app, next, active_shortcut.inner())?;
    Ok(next.to_string())
}

fn remember_foreground(target: &Arc<Mutex<isize>>) -> Option<isize> {
    #[cfg(windows)]
    {
        let handle = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
        if handle.is_null() {
            return None;
        }
        if let Ok(mut stored_handle) = target.lock() {
            *stored_handle = handle as isize;
            return Some(*stored_handle);
        }
    }
    None
}

fn paste_into_target(text: &str, target_handle: isize) -> Result<bool, String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("Clipboard is unavailable: {error}"))?;
    let previous = clipboard.get_text().ok();
    clipboard
        .set_text(text.to_owned())
        .map_err(|error| format!("Could not copy the transcript: {error}"))?;

    send_windows_paste(target_handle)?;
    thread::sleep(Duration::from_millis(160));
    Ok(match previous {
        Some(previous_text) => clipboard.set_text(previous_text).is_ok(),
        None => false,
    })
}

#[cfg(windows)]
fn send_windows_paste(target_handle: isize) -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

    if target_handle == 0 {
        return Err(
            "No target app was captured. Use the push-to-talk shortcut while the target field is active."
                .into(),
        );
    }
    let target = target_handle as _;
    if unsafe { GetForegroundWindow() } != target {
        let activated = unsafe { SetForegroundWindow(target) } != 0;
        thread::sleep(Duration::from_millis(90));
        if !activated || unsafe { GetForegroundWindow() } != target {
            return Err(
                "Windows could not return to the captured text field. The transcript remains on the clipboard."
                    .into(),
            );
        }
    }
    let inputs = paste_inputs();
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("Windows blocked text insertion. The transcript remains on the clipboard.".into())
    }
}

#[cfg(windows)]
fn paste_inputs() -> [windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT; 4] {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_KEYUP, VK_CONTROL};
    [
        keyboard_input(VK_CONTROL, 0),
        keyboard_input(b'V' as u16, 0),
        keyboard_input(b'V' as u16, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ]
}

#[cfg(windows)]
fn keyboard_input(
    virtual_key: u16,
    flags: u32,
) -> windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(desktop)]
fn replay_quick_tap(app: &AppHandle, shortcut: Shortcut) -> Result<(), String> {
    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|error| format!("Could not pause the shortcut for pass-through: {error}"))?;
    thread::sleep(Duration::from_millis(35));
    let send_result = send_shortcut_tap(shortcut);
    thread::sleep(Duration::from_millis(35));
    let registration_result = app
        .global_shortcut()
        .register(shortcut)
        .map_err(|error| format!("Could not reactivate the shortcut after pass-through: {error}"));
    match (send_result, registration_result) {
        (_, Err(registration_error)) => Err(registration_error),
        (Err(send_error), Ok(())) => Err(send_error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[cfg(windows)]
fn send_shortcut_tap(shortcut: Shortcut) -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_CONTROL, VK_LWIN, VK_MENU,
        VK_SHIFT,
    };

    let mut modifiers = Vec::with_capacity(4);
    if shortcut.mods.ctrl() {
        modifiers.push(VK_CONTROL);
    }
    if shortcut.mods.alt() {
        modifiers.push(VK_MENU);
    }
    if shortcut.mods.shift() {
        modifiers.push(VK_SHIFT);
    }
    if shortcut.mods.meta() {
        modifiers.push(VK_LWIN);
    }
    let virtual_key = virtual_key_for_code(shortcut.key).ok_or_else(|| {
        format!(
            "Windows cannot pass through the selected key ({:?}). Choose a standard keyboard, media, or function key.",
            shortcut.key
        )
    })?;
    let key_flags = if is_extended_key(shortcut.key) {
        KEYEVENTF_EXTENDEDKEY
    } else {
        0
    };
    let mut inputs: Vec<INPUT> = modifiers
        .iter()
        .map(|modifier| keyboard_input(*modifier, 0))
        .collect();
    inputs.push(keyboard_input(virtual_key, key_flags));
    inputs.push(keyboard_input(virtual_key, key_flags | KEYEVENTF_KEYUP));
    inputs.extend(
        modifiers
            .iter()
            .rev()
            .map(|modifier| keyboard_input(*modifier, KEYEVENTF_KEYUP)),
    );
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("Windows blocked the shortcut pass-through.".into())
    }
}

#[cfg(not(windows))]
fn send_shortcut_tap(_shortcut: Shortcut) -> Result<(), String> {
    Err("Quick-tap pass-through is currently available on Windows.".into())
}

#[cfg(windows)]
fn virtual_key_for_code(code: Code) -> Option<u16> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse as key;
    Some(match code {
        Code::KeyA => b'A' as u16,
        Code::KeyB => b'B' as u16,
        Code::KeyC => b'C' as u16,
        Code::KeyD => b'D' as u16,
        Code::KeyE => b'E' as u16,
        Code::KeyF => b'F' as u16,
        Code::KeyG => b'G' as u16,
        Code::KeyH => b'H' as u16,
        Code::KeyI => b'I' as u16,
        Code::KeyJ => b'J' as u16,
        Code::KeyK => b'K' as u16,
        Code::KeyL => b'L' as u16,
        Code::KeyM => b'M' as u16,
        Code::KeyN => b'N' as u16,
        Code::KeyO => b'O' as u16,
        Code::KeyP => b'P' as u16,
        Code::KeyQ => b'Q' as u16,
        Code::KeyR => b'R' as u16,
        Code::KeyS => b'S' as u16,
        Code::KeyT => b'T' as u16,
        Code::KeyU => b'U' as u16,
        Code::KeyV => b'V' as u16,
        Code::KeyW => b'W' as u16,
        Code::KeyX => b'X' as u16,
        Code::KeyY => b'Y' as u16,
        Code::KeyZ => b'Z' as u16,
        Code::Digit0 => b'0' as u16,
        Code::Digit1 => b'1' as u16,
        Code::Digit2 => b'2' as u16,
        Code::Digit3 => b'3' as u16,
        Code::Digit4 => b'4' as u16,
        Code::Digit5 => b'5' as u16,
        Code::Digit6 => b'6' as u16,
        Code::Digit7 => b'7' as u16,
        Code::Digit8 => b'8' as u16,
        Code::Digit9 => b'9' as u16,
        Code::Equal => key::VK_OEM_PLUS,
        Code::Comma => key::VK_OEM_COMMA,
        Code::Minus => key::VK_OEM_MINUS,
        Code::Period => key::VK_OEM_PERIOD,
        Code::Semicolon => key::VK_OEM_1,
        Code::Slash => key::VK_OEM_2,
        Code::Backquote => key::VK_OEM_3,
        Code::BracketLeft => key::VK_OEM_4,
        Code::Backslash => key::VK_OEM_5,
        Code::BracketRight => key::VK_OEM_6,
        Code::Quote => key::VK_OEM_7,
        Code::Backspace => key::VK_BACK,
        Code::Tab => key::VK_TAB,
        Code::Space => key::VK_SPACE,
        Code::Enter | Code::NumpadEnter => key::VK_RETURN,
        Code::CapsLock => key::VK_CAPITAL,
        Code::Escape => key::VK_ESCAPE,
        Code::PageUp => key::VK_PRIOR,
        Code::PageDown => key::VK_NEXT,
        Code::End => key::VK_END,
        Code::Home => key::VK_HOME,
        Code::ArrowLeft => key::VK_LEFT,
        Code::ArrowUp => key::VK_UP,
        Code::ArrowRight => key::VK_RIGHT,
        Code::ArrowDown => key::VK_DOWN,
        Code::PrintScreen => key::VK_SNAPSHOT,
        Code::Insert => key::VK_INSERT,
        Code::Delete => key::VK_DELETE,
        Code::F1 => key::VK_F1,
        Code::F2 => key::VK_F2,
        Code::F3 => key::VK_F3,
        Code::F4 => key::VK_F4,
        Code::F5 => key::VK_F5,
        Code::F6 => key::VK_F6,
        Code::F7 => key::VK_F7,
        Code::F8 => key::VK_F8,
        Code::F9 => key::VK_F9,
        Code::F10 => key::VK_F10,
        Code::F11 => key::VK_F11,
        Code::F12 => key::VK_F12,
        Code::F13 => key::VK_F13,
        Code::F14 => key::VK_F14,
        Code::F15 => key::VK_F15,
        Code::F16 => key::VK_F16,
        Code::F17 => key::VK_F17,
        Code::F18 => key::VK_F18,
        Code::F19 => key::VK_F19,
        Code::F20 => key::VK_F20,
        Code::F21 => key::VK_F21,
        Code::F22 => key::VK_F22,
        Code::F23 => key::VK_F23,
        Code::F24 => key::VK_F24,
        Code::NumLock => key::VK_NUMLOCK,
        Code::Numpad0 => key::VK_NUMPAD0,
        Code::Numpad1 => key::VK_NUMPAD1,
        Code::Numpad2 => key::VK_NUMPAD2,
        Code::Numpad3 => key::VK_NUMPAD3,
        Code::Numpad4 => key::VK_NUMPAD4,
        Code::Numpad5 => key::VK_NUMPAD5,
        Code::Numpad6 => key::VK_NUMPAD6,
        Code::Numpad7 => key::VK_NUMPAD7,
        Code::Numpad8 => key::VK_NUMPAD8,
        Code::Numpad9 => key::VK_NUMPAD9,
        Code::NumpadAdd => key::VK_ADD,
        Code::NumpadDecimal => key::VK_DECIMAL,
        Code::NumpadDivide => key::VK_DIVIDE,
        Code::NumpadEqual => b'E' as u16,
        Code::NumpadMultiply => key::VK_MULTIPLY,
        Code::NumpadSubtract => key::VK_SUBTRACT,
        Code::ScrollLock => key::VK_SCROLL,
        Code::AudioVolumeDown => key::VK_VOLUME_DOWN,
        Code::AudioVolumeUp => key::VK_VOLUME_UP,
        Code::AudioVolumeMute => key::VK_VOLUME_MUTE,
        Code::MediaPlay => key::VK_PLAY,
        Code::MediaPause | Code::Pause => key::VK_PAUSE,
        Code::MediaPlayPause => key::VK_MEDIA_PLAY_PAUSE,
        Code::MediaStop => key::VK_MEDIA_STOP,
        Code::MediaTrackNext => key::VK_MEDIA_NEXT_TRACK,
        Code::MediaTrackPrevious => key::VK_MEDIA_PREV_TRACK,
        _ => return None,
    })
}

#[cfg(windows)]
fn is_extended_key(code: Code) -> bool {
    matches!(
        code,
        Code::Insert
            | Code::Delete
            | Code::Home
            | Code::End
            | Code::PageUp
            | Code::PageDown
            | Code::ArrowLeft
            | Code::ArrowRight
            | Code::ArrowUp
            | Code::ArrowDown
            | Code::PrintScreen
            | Code::NumpadDivide
            | Code::NumpadEnter
            | Code::AudioVolumeDown
            | Code::AudioVolumeUp
            | Code::AudioVolumeMute
            | Code::MediaPlay
            | Code::MediaPause
            | Code::MediaPlayPause
            | Code::MediaStop
            | Code::MediaTrackNext
            | Code::MediaTrackPrevious
    )
}

fn setup_application(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| std::io::Error::other("TypeSpeak main window was not created"))?;
    let overlay = app
        .get_webview_window("recording-overlay")
        .ok_or_else(|| std::io::Error::other("TypeSpeak recording overlay was not created"))?;
    window.show()?;
    window.center()?;
    window.set_focus()?;
    minimize_to_tray_when_main_window_closes(&window);
    overlay.set_focusable(false)?;
    overlay.set_ignore_cursor_events(true)?;
    #[cfg(debug_assertions)]
    println!("TypeSpeak main window is ready.");
    install_push_to_talk(app)?;
    install_system_tray(app)?;
    warm_whisper_after_launch(app);
    Ok(())
}

fn minimize_to_tray_when_main_window_closes(window: &WebviewWindow) {
    let main_window = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Err(error) = main_window.hide() {
                eprintln!("TypeSpeak could not hide in the system tray: {error}");
            }
        }
    });
}

fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "TypeSpeak's main window is unavailable.".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

fn show_main_window(app: &AppHandle, view: Option<&str>) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "TypeSpeak's main window is unavailable.".to_string())?;
    window
        .unminimize()
        .and_then(|_| window.show())
        .and_then(|_| window.set_focus())
        .map_err(|error| error.to_string())?;
    if let Some(view) = view {
        app.emit("typespeak://navigate", view)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(desktop)]
fn install_system_tray(app: &tauri::App) -> Result<(), Box<dyn Error>> {
    let menu = MenuBuilder::new(app)
        .text("open", "Open TypeSpeak")
        .text("recent", "Recent transcripts")
        .text("settings", "Settings")
        .separator()
        .text("quit", "Quit TypeSpeak")
        .build()?;
    let mut tray = TrayIconBuilder::with_id("typespeak-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("TypeSpeak — hold your shortcut to dictate")
        .on_menu_event(|app, event| match event.id().0.as_str() {
            "open" => {
                if let Err(error) = show_main_window(app, Some("dictate")) {
                    eprintln!("TypeSpeak could not open from the tray: {error}");
                }
            }
            "recent" => {
                if let Err(error) = show_main_window(app, Some("recent")) {
                    eprintln!("TypeSpeak could not open Recent from the tray: {error}");
                }
            }
            "settings" => {
                if let Err(error) = show_main_window(app, Some("settings")) {
                    eprintln!("TypeSpeak could not open Settings from the tray: {error}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let should_open = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );
            if should_open {
                if let Err(error) = show_main_window(tray.app_handle(), None) {
                    eprintln!("TypeSpeak could not open from the tray icon: {error}");
                }
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn warm_whisper_after_launch(app: &tauri::App) {
    let whisper_runtime = app.state::<WhisperRuntime>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = warm_whisper(&whisper_runtime) {
            eprintln!("TypeSpeak could not warm the Whisper model: {error}");
        }
    });
}

#[cfg(desktop)]
fn install_push_to_talk(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    let push_to_talk = default_push_to_talk_shortcut();
    let shortcut_state = app.state::<PushToTalkShortcut>();
    let runtime = PushToTalkRuntime {
        app: app.handle().clone(),
        target: app.state::<TargetWindow>().0.clone(),
        active: shortcut_state.active.clone(),
        hold: shortcut_state.hold.clone(),
    };
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, _shortcut, event| {
                runtime.on_event(event.state());
            })
            .build(),
    )?;
    app.global_shortcut().register(push_to_talk)?;
    Ok(())
}

#[cfg(desktop)]
impl PushToTalkRuntime {
    fn on_event(&self, state: ShortcutState) {
        match state {
            ShortcutState::Pressed => self.on_pressed(),
            ShortcutState::Released => self.on_released(),
        }
    }

    fn on_pressed(&self) {
        let generation = match self.hold.lock() {
            Ok(mut hold) => hold.press(),
            Err(_) => return,
        };
        let runtime = self.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(PUSH_TO_TALK_HOLD_MS));
            let Ok(mut hold) = runtime.hold.lock() else {
                return;
            };
            if !hold.activate(generation) {
                return;
            }
            // Keep activation and the emitted "pressed" event ordered against release.
            // This prevents a boundary-timing release from arriving before recording starts.
            let target_handle = remember_foreground(&runtime.target).unwrap_or_default();
            if let Err(error) = show_recording_overlay(&runtime.app, target_handle) {
                eprintln!("TypeSpeak could not show its recording overlay: {error}");
            }
            let _ = runtime.app.emit("typespeak://hotkey", "pressed");
            drop(hold);
        });
    }

    fn on_released(&self) {
        let release = self
            .hold
            .lock()
            .map(|mut hold| hold.release())
            .unwrap_or(HoldRelease::Ignore);
        match release {
            HoldRelease::Ignore => {}
            HoldRelease::StopRecording => {
                if let Err(error) = hide_recording_overlay(&self.app) {
                    eprintln!("TypeSpeak could not hide its recording overlay: {error}");
                }
                let _ = self.app.emit("typespeak://hotkey", "released");
            }
            HoldRelease::QuickTap => {
                let shortcut = match self.active.lock() {
                    Ok(active) => *active,
                    Err(_) => return,
                };
                let app = self.app.clone();
                thread::spawn(move || {
                    if let Err(error) = replay_quick_tap(&app, shortcut) {
                        eprintln!("TypeSpeak could not pass through a quick shortcut tap: {error}");
                    }
                });
            }
        }
    }
}

fn default_push_to_talk_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space)
}

fn parse_push_to_talk_shortcut(value: &str) -> Result<Shortcut, String> {
    Shortcut::from_str(value)
        .map_err(|error| format!("TypeSpeak could not understand that shortcut: {error}"))
}

fn replace_push_to_talk_shortcut(
    app: &AppHandle,
    next: Shortcut,
    state: &PushToTalkShortcut,
) -> Result<(), String> {
    let mut active = state
        .active
        .lock()
        .map_err(|_| "The shortcut state is unavailable.".to_string())?;
    if *active == next {
        return Ok(());
    }
    app.global_shortcut()
        .unregister(*active)
        .map_err(|error| format!("Could not release the previous shortcut: {error}"))?;
    if let Err(error) = app.global_shortcut().register(next) {
        let restore_error = app.global_shortcut().register(*active).err();
        return Err(shortcut_registration_error(error, restore_error));
    }
    *active = next;
    if let Ok(mut hold) = state.hold.lock() {
        hold.reset();
    }
    Ok(())
}

fn shortcut_registration_error(
    registration: tauri_plugin_global_shortcut::Error,
    restoration: Option<tauri_plugin_global_shortcut::Error>,
) -> String {
    match restoration {
        Some(error) => format!(
            "That shortcut is unavailable ({registration}). The previous shortcut also could not be restored ({error})."
        ),
        None => format!(
            "That shortcut is already used by Windows or another app. The previous shortcut is still active. ({registration})"
        ),
    }
}

fn show_recording_overlay(app: &AppHandle, target_handle: isize) -> Result<(), String> {
    let overlay = recording_overlay(app)?;
    position_recording_overlay(&overlay, target_handle)?;
    overlay.show().map_err(|error| error.to_string())
}

fn hide_recording_overlay(app: &AppHandle) -> Result<(), String> {
    recording_overlay(app)?
        .hide()
        .map_err(|error| error.to_string())
}

fn recording_overlay(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("recording-overlay")
        .ok_or_else(|| "The recording overlay window is unavailable.".into())
}

fn position_recording_overlay(overlay: &WebviewWindow, target_handle: isize) -> Result<(), String> {
    let overlay_size = overlay.outer_size().map_err(|error| error.to_string())?;
    #[cfg(windows)]
    if let Some(position) = target_monitor_overlay_position(target_handle, overlay_size) {
        return overlay
            .set_position(position)
            .map_err(|error| error.to_string());
    }
    let position = primary_monitor_overlay_position(overlay, overlay_size)?;
    overlay
        .set_position(position)
        .map_err(|error| error.to_string())
}

fn primary_monitor_overlay_position(
    overlay: &WebviewWindow,
    overlay_size: PhysicalSize<u32>,
) -> Result<PhysicalPosition<i32>, String> {
    let monitor = overlay
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Windows did not report a primary monitor.".to_string())?;
    Ok(bottom_center_position(
        MonitorArea {
            left: monitor.position().x,
            top: monitor.position().y,
            width: monitor.size().width,
            height: monitor.size().height,
        },
        overlay_size,
    ))
}

#[cfg(windows)]
fn target_monitor_overlay_position(
    target_handle: isize,
    overlay_size: PhysicalSize<u32>,
) -> Option<PhysicalPosition<i32>> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    if target_handle == 0 {
        return None;
    }
    let monitor = unsafe { MonitorFromWindow(target_handle as _, MONITOR_DEFAULTTONEAREST) };
    let mut monitor_info: MONITORINFO = unsafe { zeroed() };
    monitor_info.cbSize = size_of::<MONITORINFO>() as u32;
    if monitor.is_null() || unsafe { GetMonitorInfoW(monitor, &mut monitor_info) } == 0 {
        return None;
    }
    let work = monitor_info.rcWork;
    Some(bottom_center_position(
        MonitorArea {
            left: work.left,
            top: work.top,
            width: (work.right - work.left) as u32,
            height: (work.bottom - work.top) as u32,
        },
        overlay_size,
    ))
}

struct MonitorArea {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
}

fn bottom_center_position(
    monitor: MonitorArea,
    overlay: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    const BOTTOM_MARGIN: i32 = 18;
    let x = monitor.left + (monitor.width.saturating_sub(overlay.width) / 2) as i32;
    let y = monitor.top + monitor.height.saturating_sub(overlay.height) as i32 - BOTTOM_MARGIN;
    PhysicalPosition::new(x, y)
}

pub fn run() {
    tauri::Builder::default()
        .manage(TargetWindow::default())
        .manage(PushToTalkShortcut::default())
        .manage(WhisperRuntime::default())
        .setup(setup_application)
        .invoke_handler(tauri::generate_handler![
            engine_status,
            managed_model_status,
            default_model_download_requested,
            custom_model_status,
            install_managed_model,
            install_custom_model,
            transcribe_audio,
            transcribe_model_endpoint,
            transcribe_managed_model,
            translate_text,
            insert_text,
            normalize_transcript,
            minimize_window,
            toggle_maximize_window,
            close_window,
            set_push_to_talk_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running TypeSpeak");
}

#[cfg(test)]
mod tests {
    use super::{
        bottom_center_position, normalize_transcript, parse_push_to_talk_shortcut, HoldRelease,
        HoldTracker, MonitorArea,
    };
    use tauri::{PhysicalPosition, PhysicalSize};

    #[test]
    fn removes_only_consecutive_duplicate_tokens() {
        let normalized =
            normalize_transcript("بدي بدي إبعت the the final version بدي إبعت".to_string());
        assert_eq!(normalized, "بدي إبعت the final version بدي إبعت");
    }

    #[test]
    fn collapses_whitespace_without_translating() {
        let normalized = normalize_transcript("  خلّينا   ship it  اليوم ".to_string());
        assert_eq!(normalized, "خلّينا ship it اليوم");
    }

    #[test]
    fn shortcuts_accept_single_windows_keys() {
        assert!(parse_push_to_talk_shortcut("Control+Shift+KeyM").is_ok());
        assert!(parse_push_to_talk_shortcut("F8").is_ok());
        assert!(parse_push_to_talk_shortcut("KeyM").is_ok());
        assert!(parse_push_to_talk_shortcut("Insert").is_ok());
        assert!(parse_push_to_talk_shortcut("PrintScreen").is_ok());
    }

    #[test]
    fn quick_tap_never_starts_recording() {
        let mut hold = HoldTracker::default();
        hold.press();
        assert!(matches!(hold.release(), HoldRelease::QuickTap));
        assert!(matches!(hold.release(), HoldRelease::Ignore));
    }

    #[test]
    fn hold_starts_once_and_stops_on_release() {
        let mut hold = HoldTracker::default();
        let generation = hold.press();
        assert!(hold.activate(generation));
        assert!(!hold.activate(generation));
        assert!(matches!(hold.release(), HoldRelease::StopRecording));
    }

    #[test]
    fn released_generation_cannot_activate_late() {
        let mut hold = HoldTracker::default();
        let generation = hold.press();
        assert!(matches!(hold.release(), HoldRelease::QuickTap));
        assert!(!hold.activate(generation));
    }

    #[test]
    fn recording_overlay_stays_centered_above_the_taskbar() {
        let monitor = MonitorArea {
            left: 1920,
            top: 0,
            width: 1920,
            height: 1040,
        };
        let position = bottom_center_position(monitor, PhysicalSize::new(118, 42));
        assert_eq!(position, PhysicalPosition::new(2821, 980));
    }
}
