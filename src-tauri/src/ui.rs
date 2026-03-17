use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, Window, WindowEvent,
};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const MAIN_TRAY_ID: &str = "main";

const TRAY_MENU_SHOW_PANEL: &str = "show-panel";
const TRAY_MENU_TOGGLE_RECORDING: &str = "toggle-recording";
const TRAY_MENU_QUIT: &str = "quit";
const TRAY_MENU_STATUS: &str = "status";

pub fn reveal_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn trigger_recording_toggle<R: Runtime>(app: &AppHandle<R>) {
    reveal_main_window(app);

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.emit("toggle-recording", ());
    }
}

pub fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        WindowEvent::Focused(false) => {
            let _ = window.hide();
        }
        _ => {}
    }
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        TRAY_MENU_SHOW_PANEL => reveal_main_window(app),
        TRAY_MENU_TOGGLE_RECORDING => trigger_recording_toggle(app),
        TRAY_MENU_QUIT => app.exit(0),
        _ => {}
    }
}

pub fn handle_tray_icon_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    if event.id().as_ref() != MAIN_TRAY_ID {
        return;
    }

    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        toggle_main_window(app);
    }
}

pub fn configure_main_tray<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<()> {
    let status_item = MenuItemBuilder::with_id(TRAY_MENU_STATUS, "Voice Input Ready")
        .enabled(false)
        .build(app_handle)?;

    let toggle_recording_item = MenuItemBuilder::with_id(TRAY_MENU_TOGGLE_RECORDING, "Start Listening")
        .accelerator("CmdOrControl+D")
        .build(app_handle)?;

    let show_panel_item = MenuItemBuilder::with_id(TRAY_MENU_SHOW_PANEL, "Open Voice Input Panel")
        .build(app_handle)?;

    let quit_item = MenuItemBuilder::with_id(TRAY_MENU_QUIT, "Quit Voice Input")
        .accelerator("CmdOrControl+Q")
        .build(app_handle)?;

    let tray_menu = MenuBuilder::new(app_handle)
        .item(&status_item)
        .separator()
        .item(&toggle_recording_item)
        .item(&show_panel_item)
        .separator()
        .item(&quit_item)
        .build()?;

    if let Some(tray) = app_handle.tray_by_id(MAIN_TRAY_ID) {
        tray.set_menu(Some(tray_menu))?;
        let _ = tray.set_show_menu_on_left_click(false);
        let _ = tray.set_tooltip(Some("Voice Input"));
    }

    Ok(())
}

#[tauri::command]
pub fn update_tray_state<R: Runtime>(
    app_handle: AppHandle<R>,
    status_label: String,
    is_recording: bool,
) -> Result<(), String> {
    if let Some(app_menu) = app_handle.menu() {
        if let Some(status_item) = app_menu.get(TRAY_MENU_STATUS) {
            status_item
                .as_menuitem()
                .ok_or("Tray status item missing")?
                .set_text(format!("Voice Input {}", status_label))
                .map_err(|error| error.to_string())?;
        }

        if let Some(toggle_item) = app_menu.get(TRAY_MENU_TOGGLE_RECORDING) {
            toggle_item
                .as_menuitem()
                .ok_or("Tray toggle item missing")?
                .set_text(if is_recording {
                    "Stop And Transcribe"
                } else {
                    "Start Listening"
                })
                .map_err(|error| error.to_string())?;
        }
    }

    if let Some(tray) = app_handle.tray_by_id(MAIN_TRAY_ID) {
        let tooltip = if is_recording {
            format!("Voice Input: {}", status_label)
        } else {
            format!("Voice Input {}", status_label)
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    Ok(())
}