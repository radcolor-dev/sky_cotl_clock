use discord_rich_presence::{
    activity::{Activity, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const DISCORD_CLIENT_ID: Option<&str> = option_env!("ISEKAI_DISCORD_CLIENT_ID");

#[derive(Default)]
struct DiscordRpcManager {
    client: Option<DiscordIpcClient>,
    client_id: Option<String>,
    connected: bool,
    active: bool,
    last_error: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordRpcButtonPayload {
    label: String,
    url: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordRpcPresencePayload {
    details: String,
    state: String,
    large_image_key: String,
    large_image_text: String,
    small_image_key: String,
    small_image_text: String,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
    buttons: Vec<DiscordRpcButtonPayload>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscordRpcStatus {
    configured: bool,
    connected: bool,
    active: bool,
    last_error: Option<String>,
}

#[tauri::command]
fn is_process_running(process_names: Vec<String>) -> Result<bool, String> {
    let normalized_names = normalize_process_names(process_names);

    if normalized_names.is_empty() {
        return Ok(false);
    }

    let processes = process_list()?;

    Ok(processes
        .into_iter()
        .any(|process| process_matches(&process, &normalized_names)))
}

#[tauri::command]
fn discord_rpc_status(
    client_id: String,
    state: tauri::State<'_, Mutex<DiscordRpcManager>>,
) -> DiscordRpcStatus {
    let manager = state.lock().expect("discord rpc mutex poisoned");
    manager.status(resolve_discord_client_id(&client_id).is_some())
}

#[tauri::command]
fn discord_rpc_update(
    payload: DiscordRpcPresencePayload,
    client_id: String,
    state: tauri::State<'_, Mutex<DiscordRpcManager>>,
) -> Result<DiscordRpcStatus, String> {
    let mut manager = state.lock().expect("discord rpc mutex poisoned");
    let resolved_client_id = resolve_discord_client_id(&client_id)
        .ok_or_else(|| manager.set_error("Discord client ID is not configured".to_string()))?;
    manager.update(payload, resolved_client_id)?;
    Ok(manager.status(true))
}

#[tauri::command]
fn discord_rpc_clear(
    state: tauri::State<'_, Mutex<DiscordRpcManager>>,
) -> Result<DiscordRpcStatus, String> {
    let mut manager = state.lock().expect("discord rpc mutex poisoned");
    manager.clear()?;
    Ok(manager.status(resolve_discord_client_id("").is_some()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(DiscordRpcManager::default()))
        .setup(|app| {
            let menu = MenuBuilder::new(app)
                .text("show-main", "Show Isekai")
                .separator()
                .text("quit", "Quit")
                .build()?;

            let mut tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Isekai")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show-main" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                    | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => show_main_window(tray.app_handle()),
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }

            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            is_process_running,
            discord_rpc_status,
            discord_rpc_update,
            discord_rpc_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

impl DiscordRpcManager {
    fn status(&self, configured: bool) -> DiscordRpcStatus {
        DiscordRpcStatus {
            configured,
            connected: self.connected,
            active: self.active,
            last_error: self.last_error.clone(),
        }
    }

    fn update(
        &mut self,
        payload: DiscordRpcPresencePayload,
        client_id: String,
    ) -> Result<(), String> {
        self.ensure_connected(&client_id)?;
        let activity = build_discord_activity(&payload);

        if let Some(client) = self.client.as_mut() {
            match client.set_activity(activity.clone()) {
                Ok(()) => {
                    self.connected = true;
                    self.active = true;
                    self.last_error = None;
                    Ok(())
                }
                Err(error) => {
                    self.connected = false;
                    self.client = None;
                    self.client_id = None;
                    self.last_error = Some(error.to_string());
                    self.ensure_connected(&client_id)?;
                    let client = self
                        .client
                        .as_mut()
                        .ok_or_else(|| "Discord RPC client unavailable".to_string())?;
                    client
                        .set_activity(activity)
                        .map_err(|retry_error| self.set_error(retry_error.to_string()))?;
                    self.connected = true;
                    self.active = true;
                    self.last_error = None;
                    Ok(())
                }
            }
        } else {
            Err(self.set_error("Discord RPC client unavailable".to_string()))
        }
    }

    fn clear(&mut self) -> Result<(), String> {
        if let Some(client) = self.client.as_mut() {
            if let Err(error) = client.clear_activity() {
                self.connected = false;
                self.client = None;
                self.client_id = None;
                self.active = false;
                return Err(self.set_error(error.to_string()));
            }
        }

        self.active = false;
        self.last_error = None;
        Ok(())
    }

    fn ensure_connected(&mut self, client_id: &str) -> Result<(), String> {
        if self.connected && self.client.is_some() && self.client_id.as_deref() == Some(client_id) {
            return Ok(());
        }

        if self.client.is_some() {
            let _ = self.clear();
        }

        let mut client = DiscordIpcClient::new(client_id);
        client
            .connect()
            .map_err(|error| self.set_error(error.to_string()))?;

        self.client = Some(client);
        self.client_id = Some(client_id.to_string());
        self.connected = true;
        self.last_error = None;
        Ok(())
    }

    fn set_error(&mut self, error: String) -> String {
        self.last_error = Some(error.clone());
        error
    }
}

fn resolve_discord_client_id(override_client_id: &str) -> Option<String> {
    let trimmed_override = override_client_id.trim();
    if !trimmed_override.is_empty() {
        return Some(trimmed_override.to_string());
    }

    DISCORD_CLIENT_ID
        .map(str::trim)
        .filter(|client_id| !client_id.is_empty())
        .map(ToOwned::to_owned)
}

fn build_discord_activity(payload: &DiscordRpcPresencePayload) -> Activity<'_> {
    let mut activity = Activity::new()
        .details(payload.details.as_str())
        .state(payload.state.as_str())
        .assets(
            Assets::new()
                .large_image(payload.large_image_key.as_str())
                .large_text(payload.large_image_text.as_str())
                .small_image(payload.small_image_key.as_str())
                .small_text(payload.small_image_text.as_str()),
        );

    let mut timestamps = Timestamps::new();
    let has_timestamps = payload.start_timestamp.is_some() || payload.end_timestamp.is_some();
    if let Some(start) = payload.start_timestamp {
        timestamps = timestamps.start(start);
    }
    if let Some(end) = payload.end_timestamp {
        timestamps = timestamps.end(end);
    }
    if has_timestamps {
        activity = activity.timestamps(timestamps);
    }

    if !payload.buttons.is_empty() {
        activity = activity.buttons(
            payload
                .buttons
                .iter()
                .take(2)
                .map(|button| Button::new(button.label.as_str(), button.url.as_str()))
                .collect(),
        );
    }

    activity
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn strip_exe(name: &str) -> &str {
    name.strip_suffix(".exe").unwrap_or(name)
}

fn normalize_process_names(process_names: Vec<String>) -> Vec<String> {
    process_names
        .into_iter()
        .map(|name| name.trim().trim_matches('"').to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect()
}

fn process_matches(process: &str, normalized_names: &[String]) -> bool {
    let process = process
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(process)
        .to_ascii_lowercase();

    normalized_names
        .iter()
        .any(|name| process == name.as_str() || process == strip_exe(name))
}

#[cfg(target_os = "windows")]
fn process_list() -> Result<Vec<String>, String> {
    let mut command = std::process::Command::new("tasklist");
    command
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|error| format!("failed to run tasklist: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(|line| line.split(',').next())
        .map(|name| name.trim().trim_matches('"').to_string())
        .filter(|name| !name.is_empty())
        .collect())
}

#[cfg(target_os = "macos")]
fn process_list() -> Result<Vec<String>, String> {
    unix_process_list()
}

#[cfg(target_os = "linux")]
fn process_list() -> Result<Vec<String>, String> {
    unix_process_list()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn unix_process_list() -> Result<Vec<String>, String> {
    let output = std::process::Command::new("ps")
        .args(["-axo", "comm="])
        .output()
        .map_err(|error| format!("failed to run ps: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(|line| line.rsplit('/').next())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn process_list() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}
