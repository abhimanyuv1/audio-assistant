use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub audio_file: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct Transcriber {
    api_key: String,
    client: reqwest::Client,
}

impl Transcriber {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Transcribe an audio file using OpenAI Whisper API
    pub async fn transcribe(&self, audio_file: PathBuf) -> Result<TranscriptionResult> {
        println!("Transcribing audio file: {:?}", audio_file);

        // Read the audio file
        let mut file = File::open(&audio_file)
            .await
            .context("Failed to open audio file")?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .context("Failed to read audio file")?;

        // Get filename
        let filename = audio_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        // Create multipart form
        let file_part = Part::bytes(buffer)
            .file_name(filename)
            .mime_str("audio/wav")?;

        let form = Form::new()
            .part("file", file_part)
            .text("model", "whisper-1")
            .text("response_format", "json");

        // Send request to OpenAI
        let response = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .context("Failed to send transcription request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Whisper API request failed with status {}: {}",
                status,
                error_text
            );
        }

        let transcription: TranscriptionResponse = response
            .json()
            .await
            .context("Failed to parse transcription response")?;

        println!("Transcription: {}", transcription.text);

        Ok(TranscriptionResult {
            text: transcription.text,
            audio_file,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Save transcription result to a file
    pub async fn save_transcription(
        &self,
        result: &TranscriptionResult,
        output_dir: &PathBuf,
    ) -> Result<PathBuf> {
        let timestamp = result.timestamp.format("%Y%m%d_%H%M%S");
        let filename = format!("transcription_{}.json", timestamp);
        let filepath = output_dir.join(filename);

        let json = serde_json::to_string_pretty(result)?;
        tokio::fs::write(&filepath, json).await?;

        println!("Transcription saved to: {:?}", filepath);
        Ok(filepath)
    }
}
