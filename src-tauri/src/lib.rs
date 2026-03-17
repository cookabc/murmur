// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod asr;
mod clipboard;
mod health;
mod hotkey;
mod settings;
mod ui;

use audio::AudioRecorder;
use asr::AsrClient;
use hotkey::HotkeyManager;
use settings::SettingsManager;
use std::sync::Arc;
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_manager = SettingsManager::new();
    let initial_hotkey = settings_manager
        .load_settings()
        .unwrap_or_default()
        .hotkey;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app.emit("hotkey-pressed", shortcut.to_string());
                    }
                })
                .build(),
        )
        .on_window_event(ui::handle_window_event)
        .on_menu_event(|app, event| ui::handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(ui::handle_tray_icon_event)
        .setup(move |app| {
            // Initialize state
            app.manage(AudioRecorder::new());
            app.manage(Arc::new(std::sync::Mutex::new(AsrClient::default())));
            let mut hotkey_manager = HotkeyManager::new();
            if let Err(error) = hotkey_manager.register_hotkey(&app.handle().clone(), &initial_hotkey)
            {
                eprintln!("Failed to restore hotkey '{}': {}", initial_hotkey, error);
                hotkey_manager.register_hotkey(&app.handle().clone(), "Command+Shift+V")?;
            }
            app.manage(std::sync::Mutex::new(hotkey_manager));
            app.manage(Arc::new(std::sync::Mutex::new(settings_manager)));

            let window = app
                .get_webview_window(ui::MAIN_WINDOW_LABEL)
                .ok_or("Main window is missing")?;
            ui::configure_main_tray(&app.handle())?;

            let app_handle_for_hotkey = app.handle().clone();
            app.listen("hotkey-pressed", move |_| {
                eprintln!("Hotkey pressed, toggling recording state");

                ui::trigger_recording_toggle(&app_handle_for_hotkey);
            });

            let _ = window.hide();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            audio::start_recording,
            audio::stop_recording,
            audio::is_recording,
            audio::check_audio_deps,
            // ASR commands
            asr::transcribe_audio,
            asr::check_coli_available,
            health::get_runtime_health,
            // Clipboard commands
            clipboard::paste_transcription,
            clipboard::copy_to_clipboard_cmd,
            clipboard::get_clipboard,
            // Hotkey commands
            hotkey::register_hotkey,
            hotkey::unregister_hotkey,
            hotkey::get_current_hotkey,
            ui::update_tray_state,
            // Settings commands
            settings::get_settings,
            settings::update_settings,
            settings::get_history,
            settings::add_history_entry,
            settings::clear_history,
            settings::delete_history_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
