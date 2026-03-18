use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
use std::env;
#[cfg(test)]
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub lang: Option<String>,
    pub duration: Option<f64>,
}

pub struct AsrClient {
    model: String,
    polish: bool,
}

impl AsrClient {
    pub fn new(model: String, polish: bool) -> Self {
        Self { model, polish }
    }

    pub async fn transcribe(&self, audio_path: &Path) -> Result<TranscriptionResult, String> {
        let model = self.model.clone();
        let polish = self.polish;
        let audio_path = audio_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            voice_input_core::transcribe_audio(audio_path, Some(model), Some(polish)).map(
                |result| TranscriptionResult {
                    text: result.text,
                    lang: result.lang,
                    duration: result.duration,
                },
            )
        })
        .await
        .map_err(|error| format!("Failed to join transcription task: {}", error))?
    }

    pub fn check_availability() -> bool {
        voice_input_core::check_coli_available()
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn set_polish(&mut self, polish: bool) {
        self.polish = polish;
    }
}

#[cfg(test)]
fn resolve_coli_from_path_var(path_var: Option<std::ffi::OsString>) -> Option<PathBuf> {
    let Some(path_var) = path_var else {
        return None;
    };

    env::split_paths(&path_var)
        .map(|entry| entry.join("coli"))
        .find(|candidate| candidate.is_file())
}

impl Default for AsrClient {
    fn default() -> Self {
        Self::new("sensevoice".to_string(), true)
    }
}

// Tauri commands
#[tauri::command]
pub async fn transcribe_audio(
    audio_path: String,
    model: Option<String>,
    polish: Option<bool>,
    state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<AsrClient>>>,
) -> Result<TranscriptionResult, String> {
    let client = {
        let mut client = state.lock().map_err(|e| e.to_string())?;

        if let Some(m) = model {
            client.set_model(m);
        }
        if let Some(p) = polish {
            client.set_polish(p);
        }

        AsrClient::new(client.model.clone(), client.polish)
    };

    let path = Path::new(&audio_path);
    client.transcribe(path).await
}

#[tauri::command]
pub fn check_coli_available() -> bool {
    AsrClient::check_availability()
}

#[cfg(test)]
mod tests {
    use super::resolve_coli_from_path_var;
    use std::fs;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("voice-input-mac-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn resolves_coli_from_path_entries() {
        let temp_dir = unique_temp_dir("coli-path");
        let coli_path = temp_dir.join("coli");
        fs::write(&coli_path, b"#!/bin/sh\nexit 0\n").unwrap();

        let path_var = OsString::from(temp_dir.into_os_string());
        let resolved = resolve_coli_from_path_var(Some(path_var)).unwrap();

        assert_eq!(resolved, coli_path);
    }

    #[test]
    fn returns_none_when_path_is_missing() {
        assert!(resolve_coli_from_path_var(None).is_none());
    }
}
