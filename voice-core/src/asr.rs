use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub lang: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ColiResponse {
    text: String,
    #[serde(rename = "text_clean")]
    text_clean: Option<String>,
    lang: Option<String>,
    duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ColiError {
    error: String,
}

pub fn check_coli_available(configured_path: Option<String>) -> bool {
    resolve_executable_path(configured_path).is_some()
}

pub fn transcribe_audio(
    configured_path: Option<String>,
    audio_path: &Path,
    model: Option<String>,
    polish: Option<bool>,
) -> Result<TranscriptionResult, String> {
    if !audio_path.exists() {
        return Err(format!("Audio file not found: {}", audio_path.display()));
    }

    let audio_path_str = audio_path.to_str().ok_or("Invalid audio path")?;
    let model = model.unwrap_or_else(|| "sensevoice".to_string());
    let polish = polish.unwrap_or(true);
    let coli_executable = resolve_executable_path(configured_path)
        .ok_or("Failed to locate the coli executable. Bundle it into the app or install @marswave/coli.")?;

    let mut child = Command::new(&coli_executable)
        .args(["asr", "-j", "--model", model.as_str(), audio_path_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to execute coli via {}: {}", coli_executable.display(), error))?;

    match child
        .wait_timeout(Duration::from_secs(120))
        .map_err(|error| format!("Failed while waiting for coli: {}", error))?
    {
        Some(_) => {}
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("coli asr timed out after 120 seconds".to_string());
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to collect coli output: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("coli asr failed with status {}", output.status)
        } else {
            format!("coli asr failed: {stderr}")
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(response) = serde_json::from_str::<ColiResponse>(&stdout) {
        let text = if polish {
            response.text_clean.unwrap_or(response.text)
        } else {
            response.text
        };

        Ok(TranscriptionResult {
            text,
            lang: response.lang,
            duration: response.duration,
        })
    } else if let Ok(error) = serde_json::from_str::<ColiError>(&stdout) {
        Err(error.error)
    } else {
        Err(format!("Failed to parse coli response: {}", stdout.trim()))
    }
}

fn resolve_executable_path(configured_path: Option<String>) -> Option<PathBuf> {
    if let Some(path) = configured_path.filter(|value| !value.trim().is_empty()) {
        let candidate = PathBuf::from(path);
        if coli_responds(&candidate) {
            return Some(candidate);
        }
    }

    if let Some(path) = resolve_from_path_var("coli", env::var_os("PATH")) {
        return Some(path);
    }

    if let Some(nvm_bin) = env::var_os("NVM_BIN") {
        let candidate = PathBuf::from(nvm_bin).join("coli");
        if coli_responds(&candidate) {
            return Some(candidate);
        }
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    if let Some(home_dir) = home.as_ref() {
        if let Some(path) = search_nvm_node_bins(home_dir) {
            return Some(path);
        }

        for relative in [".npm-global/bin/coli", ".volta/bin/coli", ".local/bin/coli"] {
            let candidate = home_dir.join(relative);
            if coli_responds(&candidate) {
                return Some(candidate);
            }
        }
    }

    for candidate in ["/opt/homebrew/bin/coli", "/usr/local/bin/coli", "/usr/bin/coli"] {
        let path = PathBuf::from(candidate);
        if coli_responds(&path) {
            return Some(path);
        }
    }

    None
}

fn resolve_from_path_var(binary: &str, path_var: Option<OsString>) -> Option<PathBuf> {
    let path_var = path_var?;

    env::split_paths(&path_var)
        .map(|entry| entry.join(binary))
        .find(|candidate| coli_responds(candidate))
}

fn search_nvm_node_bins(home_dir: &Path) -> Option<PathBuf> {
    let versions_dir = home_dir.join(".nvm/versions/node");
    let entries = fs::read_dir(versions_dir).ok()?;

    let mut candidates = entries
        .filter_map(|entry| entry.ok().map(|item| item.path().join("bin/coli")))
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| right.cmp(left));
    candidates.into_iter().find(|candidate| coli_responds(candidate))
}

fn coli_responds(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    Command::new(path)
        .arg("-h")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{resolve_from_path_var, transcribe_audio};
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("voice-input-core-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn resolves_coli_from_path_entries() {
        let temp_dir = unique_temp_dir("coli-path");
        let coli_path = temp_dir.join("coli");
        fs::write(&coli_path, b"#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&coli_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&coli_path, permissions).unwrap();
        }

        let path_var = OsString::from(temp_dir.into_os_string());
        let resolved = resolve_from_path_var("coli", Some(path_var)).unwrap();

        assert_eq!(resolved, coli_path);
    }

    #[test]
    fn missing_audio_file_is_reported() {
        let audio_path = PathBuf::from("/tmp/voice-input-core-missing.wav");
        let error = transcribe_audio(None, &audio_path, None, None).unwrap_err();
        assert!(error.contains("Audio file not found"));
    }
}