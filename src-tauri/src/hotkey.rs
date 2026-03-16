use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub accelerator: String,
    pub enabled: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            accelerator: "Cmd+Shift+V".to_string(),
            enabled: true,
        }
    }
}

pub struct HotkeyManager {
    current_hotkey: Option<String>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            current_hotkey: None,
        }
    }

    pub fn register_hotkey(
        &mut self,
        app_handle: &AppHandle,
        accelerator: &str,
    ) -> Result<(), String> {
        // For now, we'll just track the hotkey string
        // The actual hotkey registration will be done in tauri.conf.json
        // or through the global-shortcut plugin's static configuration
        self.current_hotkey = Some(accelerator.to_string());
        eprintln!("Hotkey registered: {}", accelerator);
        Ok(())
    }

    pub fn unregister_hotkey(&mut self, _app_handle: &AppHandle) -> Result<(), String> {
        self.current_hotkey = None;
        Ok(())
    }

    pub fn get_current_hotkey(&self) -> Option<&str> {
        self.current_hotkey.as_deref()
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands
#[tauri::command]
pub fn register_hotkey(
    accelerator: String,
    state: tauri::State<'_, std::sync::Mutex<HotkeyManager>>,
    _app_handle: AppHandle,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.register_hotkey(&_app_handle, &accelerator)
}

#[tauri::command]
pub fn unregister_hotkey(
    state: tauri::State<'_, std::sync::Mutex<HotkeyManager>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| e.to_string())?;
    manager.unregister_hotkey(&app_handle)
}

#[tauri::command]
pub fn get_current_hotkey(
    state: tauri::State<'_, std::sync::Mutex<HotkeyManager>>,
) -> Option<String> {
    let manager = state.lock().ok()?;
    manager.get_current_hotkey().map(|s| s.to_string())
}
