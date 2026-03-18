use crate::audio;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealth {
    pub ffmpeg_available: bool,
    pub coli_available: bool,
    pub osascript_available: bool,
    pub ready: bool,
    pub issues: Vec<String>,
}

fn command_available(binary: &str) -> bool {
    Command::new("/usr/bin/which")
        .arg(binary)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn build_runtime_health(
    ffmpeg_available: bool,
    coli_available: bool,
    osascript_available: bool,
) -> RuntimeHealth {
    let mut issues = Vec::new();
    if !ffmpeg_available {
        issues.push("ffmpeg is missing. Install it with: brew install ffmpeg".to_string());
    }
    if !coli_available {
        issues.push("coli is missing or unavailable. Install it before starting transcription.".to_string());
    }
    if !osascript_available {
        issues.push("osascript is unavailable, so simulated paste may not work.".to_string());
    }

    RuntimeHealth {
        ffmpeg_available,
        coli_available,
        osascript_available,
        ready: ffmpeg_available && coli_available,
        issues,
    }
}

#[tauri::command]
pub fn get_runtime_health() -> RuntimeHealth {
    let ffmpeg_available = audio::check_ffmpeg_available();
    let coli_available = voice_input_core::check_coli_available();
    let osascript_available = command_available("osascript");

    build_runtime_health(ffmpeg_available, coli_available, osascript_available)
}

#[cfg(test)]
mod tests {
    use super::build_runtime_health;

    #[test]
    fn ready_depends_on_ffmpeg_and_coli() {
        let health = build_runtime_health(true, true, false);

        assert!(health.ready);
        assert_eq!(health.issues.len(), 1);
        assert_eq!(
            health.issues[0],
            "osascript is unavailable, so simulated paste may not work."
        );
    }

    #[test]
    fn missing_dependencies_are_reported() {
        let health = build_runtime_health(false, false, true);

        assert!(!health.ready);
        assert_eq!(health.issues.len(), 2);
        assert!(health.issues.iter().any(|issue| issue.contains("ffmpeg is missing")));
        assert!(health.issues.iter().any(|issue| issue.contains("coli is missing")));
    }
}