use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

const CORE_NAME: &str = "voice-input-core";
const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Default, Serialize)]
struct ToolPaths {
    ffmpeg_path: Option<String>,
    coli_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeStatus {
    name: &'static str,
    version: &'static str,
    ffmpeg_path: Option<String>,
    coli_path: Option<String>,
    ffmpeg_exists: bool,
    coli_exists: bool,
}

static TOOL_PATHS: OnceLock<RwLock<ToolPaths>> = OnceLock::new();

fn tool_paths() -> &'static RwLock<ToolPaths> {
    TOOL_PATHS.get_or_init(|| RwLock::new(ToolPaths::default()))
}

fn c_string_from(value: String) -> *mut c_char {
    CString::new(value)
        .expect("CString::new failed")
        .into_raw()
}

fn optional_string_from_ptr(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let string = unsafe { CStr::from_ptr(value) }.to_string_lossy().trim().to_string();
    if string.is_empty() {
        None
    } else {
        Some(string)
    }
}

fn path_exists(path: Option<&String>) -> bool {
    path.map(PathBuf::from).is_some_and(|value| value.exists())
}

#[no_mangle]
pub extern "C" fn voice_input_core_version() -> *mut c_char {
    c_string_from(CORE_VERSION.to_string())
}

#[no_mangle]
pub extern "C" fn voice_input_core_configure_tools(
    ffmpeg_path: *const c_char,
    coli_path: *const c_char,
) -> bool {
    let updated = ToolPaths {
        ffmpeg_path: optional_string_from_ptr(ffmpeg_path),
        coli_path: optional_string_from_ptr(coli_path),
    };

    if let Ok(mut guard) = tool_paths().write() {
        *guard = updated;
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn voice_input_core_smoke_status_json() -> *mut c_char {
    let tool_paths = tool_paths()
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default();

    let status = SmokeStatus {
        name: CORE_NAME,
        version: CORE_VERSION,
        ffmpeg_exists: path_exists(tool_paths.ffmpeg_path.as_ref()),
        coli_exists: path_exists(tool_paths.coli_path.as_ref()),
        ffmpeg_path: tool_paths.ffmpeg_path,
        coli_path: tool_paths.coli_path,
    };

    c_string_from(serde_json::to_string(&status).expect("serialize smoke status"))
}

#[no_mangle]
pub extern "C" fn voice_input_core_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{tool_paths, ToolPaths};

    #[test]
    fn stores_tool_configuration() {
        let mut guard = tool_paths().write().unwrap();
        *guard = ToolPaths {
            ffmpeg_path: Some("/tmp/ffmpeg".into()),
            coli_path: Some("/tmp/coli".into()),
        };

        assert_eq!(guard.ffmpeg_path.as_deref(), Some("/tmp/ffmpeg"));
        assert_eq!(guard.coli_path.as_deref(), Some("/tmp/coli"));
    }
}
