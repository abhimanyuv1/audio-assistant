use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    pub summary: String,
    pub action_items: Vec<String>,
    pub original_text: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct Summarizer {
    api_key: String,
    client: Client,
    model: String,
}

impl Summarizer {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            model,
        }
    }

    /// Generate summary and extract action items from transcribed text
    pub async fn summarize(&self, text: &str) -> Result<SummaryResult> {
        println!("Generating summary for text of length: {}", text.len());

        let system_prompt = r#"You are an AI assistant that summarizes conversations and extracts action items.

Your task:
1. Provide a concise summary of the conversation
2. Extract any action items, tasks, or to-dos mentioned
3. Return the result in the following JSON format:

{
  "summary": "Brief summary of the conversation...",
  "action_items": ["Action item 1", "Action item 2", ...]
}

If there are no action items, return an empty array."#;

        let user_prompt = format!(
            "Please summarize the following conversation and extract any action items:\n\n{}",
            text
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: 0.3,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send summarization request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "OpenAI API request failed with status {}: {}",
                status,
                error_text
            );
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse chat response")?;

        let content = &chat_response
            .choices
            .first()
            .context("No response from GPT")?
            .message
            .content;

        // Parse the JSON response from GPT
        #[derive(Deserialize)]
        struct GptOutput {
            summary: String,
            action_items: Vec<String>,
        }

        let gpt_output: GptOutput =
            serde_json::from_str(content).context("Failed to parse GPT JSON output")?;

        println!("Summary generated: {}", gpt_output.summary);
        println!("Action items found: {}", gpt_output.action_items.len());

        Ok(SummaryResult {
            summary: gpt_output.summary,
            action_items: gpt_output.action_items,
            original_text: text.to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Save summary result to a file
    pub async fn save_summary(
        &self,
        result: &SummaryResult,
        output_dir: &PathBuf,
    ) -> Result<PathBuf> {
        let timestamp = result.timestamp.format("%Y%m%d_%H%M%S");
        let filename = format!("summary_{}.json", timestamp);
        let filepath = output_dir.join(filename);

        let json = serde_json::to_string_pretty(result)?;
        tokio::fs::write(&filepath, json).await?;

        println!("Summary saved to: {:?}", filepath);
        Ok(filepath)
    }

    /// Generate a cumulative summary from multiple transcription chunks
    pub async fn summarize_conversation(&self, transcriptions: &[String]) -> Result<SummaryResult> {
        let combined_text = transcriptions.join("\n\n--- Next segment ---\n\n");
        self.summarize(&combined_text).await
    }
}
