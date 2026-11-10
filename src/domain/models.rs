use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct VoiceInfo {
    pub index: usize,
    pub name: String,
    pub language: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub title: String,
    pub path: PathBuf,
    pub format: String,
    pub characters: usize,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub rate: i32,
    pub volume: u32,
    pub bitrate_kbps: u32,
    pub output_format: String,
    pub selected_voice: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rate: 0,
            volume: 100,
            bitrate_kbps: 192,
            output_format: "MP3".to_string(),
            selected_voice: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Mp3,
    Wav,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mp3 => "MP3",
            Self::Wav => "WAV",
        }
    }
}

impl From<&str> for OutputFormat {
    fn from(value: &str) -> Self {
        if value.eq_ignore_ascii_case("wav") {
            Self::Wav
        } else {
            Self::Mp3
        }
    }
}
