use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// OpenAI API key for Whisper and GPT
    pub openai_api_key: String,

    /// Duration of each audio chunk in seconds
    pub chunk_duration_secs: u64,

    /// Sample rate for audio capture
    pub sample_rate: u32,

    /// Directory to store audio chunks (temporary)
    pub audio_chunks_dir: PathBuf,

    /// Directory to store transcriptions
    pub transcriptions_dir: PathBuf,

    /// Directory to store summaries
    pub summaries_dir: PathBuf,

    /// Whether to keep audio files after transcription
    pub keep_audio_files: bool,

    /// Process in real-time or batch mode
    pub realtime_processing: bool,

    /// OpenAI model for summarization
    pub summarization_model: String,
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("audio-assistant");

        Self {
            openai_api_key: String::new(),
            chunk_duration_secs: 30, // 30 second chunks by default
            sample_rate: 16000,      // 16kHz is good for speech
            audio_chunks_dir: base_dir.join("audio_chunks"),
            transcriptions_dir: base_dir.join("transcriptions"),
            summaries_dir: base_dir.join("summaries"),
            keep_audio_files: false,
            realtime_processing: true,
            summarization_model: "gpt-4o-mini".to_string(),
        }
    }
}

impl Config {
    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    /// Get the config file path
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("audio-assistant")
            .join("config.json")
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.audio_chunks_dir)?;
        fs::create_dir_all(&self.transcriptions_dir)?;
        fs::create_dir_all(&self.summaries_dir)?;
        Ok(())
    }

    /// Validate that the config is ready to use
    pub fn validate(&self) -> Result<()> {
        if self.openai_api_key.is_empty() {
            anyhow::bail!("OpenAI API key is not set");
        }
        Ok(())
    }
}
