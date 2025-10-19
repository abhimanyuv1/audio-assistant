# Development Guide 👨‍💻

A guide for developers who want to understand, modify, or extend Audio Assistant.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Project Structure](#project-structure)
- [Key Components](#key-components)
- [Extending the Application](#extending-the-application)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Common Modifications](#common-modifications)

---

## Architecture Overview

Audio Assistant follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                         GUI Layer (egui)                     │
│                      main.rs - Event Loop                    │
└──────────────┬──────────────────────────────┬────────────────┘
               │                              │
               ↓                              ↓
┌──────────────────────────┐    ┌──────────────────────────┐
│   Audio Capture Thread   │    │   Async Task Pool        │
│   (audio_capture.rs)     │    │   (tokio runtime)        │
└────────────┬─────────────┘    └─────────┬────────────────┘
             │                            │
             ↓                            ↓
┌──────────────────────┐    ┌──────────────────────────────┐
│   Audio Chunks       │    │   API Services               │
│   (WAV files)        │───→│  - Transcription (Whisper)   │
└──────────────────────┘    │  - Summarization (GPT)       │
                             └──────────────────────────────┘
                                          │
                                          ↓
                             ┌──────────────────────────────┐
                             │   Storage Layer              │
                             │  - Transcriptions (JSON)     │
                             │  - Summaries (JSON)          │
                             └──────────────────────────────┘
```

### Design Patterns

1. **Message Passing**: Uses channels (mpsc) for thread communication
2. **Async/Await**: Tokio for non-blocking I/O operations
3. **Immediate Mode GUI**: egui for responsive interface
4. **Configuration Management**: JSON-based persistent settings

---

## Project Structure

```
audio-assistant/
├── src/
│   ├── main.rs              # GUI application and orchestration
│   ├── config.rs            # Configuration management
│   ├── audio_capture.rs     # System audio recording
│   ├── transcription.rs     # Whisper API integration
│   └── summarization.rs     # GPT API integration
├── Cargo.toml               # Dependencies and metadata
├── README.md                # User documentation
├── QUICKSTART.md            # Quick start guide
├── TROUBLESHOOTING.md       # Problem solutions
├── DEVELOPMENT.md           # This file
├── setup-audio.sh           # Linux audio setup helper
└── config.example.json      # Example configuration
```

---

## Key Components

### 1. Main Application (`main.rs`)

**Responsibilities**:
- GUI rendering with egui
- Message queue processing
- Orchestrating audio capture, transcription, and summarization
- State management

**Key structures**:
```rust
struct AudioAssistantApp {
    config: Config,
    audio_capture: Option<AudioCapture>,
    is_listening: bool,
    message_tx: Sender<AppMessage>,
    message_rx: Arc<Mutex<Receiver<AppMessage>>>,
    transcriptions: Vec<TranscriptionResult>,
    summaries: Vec<SummaryResult>,
    // ... UI state
}

enum AppMessage {
    AudioChunkReady(PathBuf),
    TranscriptionReady(TranscriptionResult),
    SummaryReady(SummaryResult),
    Error(String),
}
```

**Message Flow**:
1. Audio chunk ready → spawn transcription task
2. Transcription ready → optionally spawn summarization task
3. Summary ready → update UI

### 2. Audio Capture (`audio_capture.rs`)

**Responsibilities**:
- Initialize audio device using cpal
- Capture audio samples in real-time
- Buffer and chunk audio data
- Write chunks to WAV files

**Key features**:
- Cross-platform audio with `cpal`
- Sample format conversion (i16, u16, f32 → f32)
- Configurable sample rate and chunk duration
- Thread-safe recording state

**Thread Architecture**:
```
Main Thread              Capture Thread         Chunk Writer Thread
    |                         |                        |
    |---start_recording()---->|                        |
    |                         |----spawn thread------->|
    |                         |                        |
    |                    [Capture loop]                |
    |                         |----samples------------>|
    |                         |                        |--write_wav()
    |                         |                        |--on_chunk_ready()
    |<-------------------callback--------------------|
```

### 3. Transcription (`transcription.rs`)

**Responsibilities**:
- Upload audio files to OpenAI Whisper API
- Handle API authentication and errors
- Parse transcription responses
- Save transcriptions to JSON

**API Flow**:
```rust
async fn transcribe(audio_file: PathBuf) -> Result<TranscriptionResult> {
    // 1. Read audio file
    // 2. Create multipart form with file
    // 3. POST to OpenAI Whisper endpoint
    // 4. Parse JSON response
    // 5. Return structured result
}
```

**Error Handling**:
- Network errors
- Authentication failures
- Rate limiting
- Invalid audio format

### 4. Summarization (`summarization.rs`)

**Responsibilities**:
- Send transcribed text to OpenAI GPT API
- Extract summaries and action items
- Handle JSON parsing from GPT responses
- Save summaries to files

**Prompt Engineering**:
```rust
const SYSTEM_PROMPT: &str = r#"
You are an AI assistant that summarizes conversations and extracts action items.
Your task:
1. Provide a concise summary of the conversation
2. Extract any action items, tasks, or to-dos mentioned
3. Return the result in JSON format
"#;
```

**API Request Structure**:
```json
{
  "model": "gpt-4o-mini",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "Transcription text..."}
  ],
  "temperature": 0.3
}
```

### 5. Configuration (`config.rs`)

**Responsibilities**:
- Load/save configuration from JSON
- Provide sensible defaults
- Validate settings
- Ensure directory structure exists

**Storage Locations**:
- Config: `~/.config/audio-assistant/config.json`
- Data: `~/.local/share/audio-assistant/`

---

## Extending the Application

### Adding a New Feature

**Example**: Add speaker diarization (identifying different speakers)

1. **Update data structures**:
```rust
// In transcription.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub speaker: Option<String>,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub segments: Vec<TranscriptionSegment>,
    pub audio_file: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

2. **Update API integration**:
```rust
// Use a different API that supports diarization
async fn transcribe_with_diarization(audio_file: PathBuf) -> Result<TranscriptionResult> {
    // Implementation
}
```

3. **Update UI**:
```rust
// In main.rs update() method
ui.collapsing("Speakers", |ui| {
    for segment in &transcription.segments {
        ui.horizontal(|ui| {
            if let Some(speaker) = &segment.speaker {
                ui.label(format!("{}: ", speaker));
            }
            ui.label(&segment.text);
        });
    }
});
```

### Adding a New API Service

**Example**: Add local Whisper model support

1. **Create new module** `src/local_transcription.rs`:
```rust
use whisper_rs::WhisperContext;

pub struct LocalTranscriber {
    model: WhisperContext,
}

impl LocalTranscriber {
    pub fn new(model_path: &str) -> Result<Self> {
        let model = WhisperContext::new(model_path)?;
        Ok(Self { model })
    }

    pub async fn transcribe(&self, audio_file: PathBuf) -> Result<TranscriptionResult> {
        // Implementation using whisper.cpp bindings
    }
}
```

2. **Update config**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionBackend {
    OpenAI,
    Local { model_path: String },
}

pub struct Config {
    pub transcription_backend: TranscriptionBackend,
    // ... other fields
}
```

3. **Update main.rs** to switch between backends:
```rust
match &self.config.transcription_backend {
    TranscriptionBackend::OpenAI => {
        let transcriber = Transcriber::new(api_key);
        // Use OpenAI
    }
    TranscriptionBackend::Local { model_path } => {
        let transcriber = LocalTranscriber::new(model_path)?;
        // Use local model
    }
}
```

### Adding Export Functionality

**Example**: Export summaries as Markdown

1. **Add export module** `src/export.rs`:
```rust
use std::path::PathBuf;
use anyhow::Result;

pub fn export_to_markdown(
    summary: &SummaryResult,
    output_path: PathBuf,
) -> Result<()> {
    let mut content = String::new();
    
    content.push_str(&format!("# Summary - {}\n\n", 
        summary.timestamp.format("%Y-%m-%d %H:%M:%S")));
    content.push_str(&summary.summary);
    content.push_str("\n\n## Action Items\n\n");
    
    for (i, item) in summary.action_items.iter().enumerate() {
        content.push_str(&format!("{}. {}\n", i + 1, item));
    }
    
    std::fs::write(output_path, content)?;
    Ok(())
}
```

2. **Add UI button**:
```rust
if let Some(summary) = &self.current_summary {
    if ui.button("📄 Export as Markdown").clicked() {
        let path = self.config.summaries_dir.join("export.md");
        if let Err(e) = export_to_markdown(summary, path) {
            self.error_message = format!("Export failed: {}", e);
        }
    }
}
```

---

## Development Workflow

### Setting Up Development Environment

```bash
# Clone the repository
cd rust-course/audio-assistant

# Install dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libasound2-dev libssl-dev

# Build in debug mode (faster compilation)
cargo build

# Run with logging
RUST_LOG=debug cargo run
```

### Development Mode Features

**Enable more verbose logging**:
```rust
// In main.rs
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

**Add debug UI elements**:
```rust
// In main.rs update() method
if cfg!(debug_assertions) {
    ui.separator();
    ui.label("Debug Info");
    ui.label(format!("Frame time: {:?}", ctx.input(|i| i.stable_dt)));
    ui.label(format!("Memory usage: {:?}", ctx.memory()));
}
```

### Code Style Guidelines

1. **Formatting**: Use `rustfmt`
   ```bash
   cargo fmt
   ```

2. **Linting**: Use `clippy`
   ```bash
   cargo clippy -- -W clippy::all
   ```

3. **Documentation**: Add doc comments
   ```rust
   /// Transcribe an audio file using OpenAI Whisper API.
   ///
   /// # Arguments
   /// * `audio_file` - Path to the audio file to transcribe
   ///
   /// # Returns
   /// * `Result<TranscriptionResult>` - The transcription result or an error
   ///
   /// # Errors
   /// This function will return an error if:
   /// - The audio file cannot be read
   /// - The API request fails
   /// - The response cannot be parsed
   pub async fn transcribe(&self, audio_file: PathBuf) -> Result<TranscriptionResult> {
       // Implementation
   }
   ```

---

## Testing

### Unit Tests

Add tests to each module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.chunk_duration_secs, 30);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_config_save_load() {
        let config = Config::default();
        config.save().unwrap();
        let loaded = Config::load().unwrap();
        assert_eq!(config.chunk_duration_secs, loaded.chunk_duration_secs);
    }
}
```

Run tests:
```bash
cargo test
```

### Integration Tests

Create `tests/integration_test.rs`:
```rust
use audio_assistant::config::Config;

#[test]
fn test_full_workflow() {
    // Test complete flow from config to audio capture
}
```

### Manual Testing Checklist

- [ ] Start/stop listening works
- [ ] Audio chunks are created
- [ ] Transcription produces text
- [ ] Summaries are generated
- [ ] Config is saved/loaded
- [ ] UI is responsive
- [ ] Errors are displayed properly
- [ ] Files are created in correct locations

---

## Common Modifications

### Change Sample Rate

In `config.rs`:
```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            sample_rate: 44100,  // Change from 16000
            // ...
        }
    }
}
```

### Add Custom Summarization Prompt

In `summarization.rs`:
```rust
let system_prompt = r#"
You are a technical meeting assistant.
Focus on extracting:
1. Technical decisions made
2. Implementation details
3. Blockers or issues
4. Action items with assignees
"#;
```

### Support Different Audio Formats

Add `mp3lame` or similar encoder:
```toml
# Cargo.toml
[dependencies]
lame = "0.1"
```

```rust
// In audio_capture.rs
fn write_mp3_file(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<()> {
    // Implementation
}
```

### Add Keyboard Shortcuts

In `main.rs`:
```rust
// In update() method
ctx.input(|i| {
    if i.key_pressed(egui::Key::L) && i.modifiers.ctrl {
        // Toggle listening
        if self.is_listening {
            self.stop_listening();
        } else {
            self.start_listening();
        }
    }
});
```

### Add Database Storage

Replace JSON files with SQLite:

```toml
# Cargo.toml
[dependencies]
rusqlite = "0.30"
```

```rust
// Create new module src/database.rs
use rusqlite::Connection;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                audio_file TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn save_transcription(&self, result: &TranscriptionResult) -> Result<()> {
        // Implementation
    }
}
```

---

## Performance Optimization

### Profile the Application

```bash
# Install flamegraph
cargo install flamegraph

# Run with profiling
cargo flamegraph

# Open flamegraph.svg in browser
```

### Optimize Audio Processing

1. **Use ring buffers** instead of Vec for audio samples
2. **Reduce lock contention** with lock-free data structures
3. **Batch API calls** if processing multiple chunks

### Reduce Memory Usage

1. **Clear old transcriptions** automatically
2. **Use Arc<str>** instead of String for large text
3. **Stream large files** instead of loading entirely

---

## Debugging Tips

### Debug Audio Issues

```rust
// Add logging in audio_capture.rs
println!("Captured {} samples", buffer.len());
println!("Chunk size: {} bytes", chunk.len() * 4);
```

### Debug API Issues

```rust
// Log full request/response
println!("Request body: {:?}", request);
let response_text = response.text().await?;
println!("Response: {}", response_text);
```

### Debug GUI Issues

```rust
// Show debug window
egui::Window::new("Debug").show(ctx, |ui| {
    ui.label(format!("Messages pending: {}", self.message_rx.lock().unwrap().len()));
    ui.label(format!("Is listening: {}", self.is_listening));
});
```

---

## Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Test** thoroughly
5. **Document** your changes
6. **Submit** a pull request

### Code Review Checklist

- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] No breaking changes (or documented)

---

## Useful Resources

### Rust Audio
- [cpal documentation](https://docs.rs/cpal/)
- [hound documentation](https://docs.rs/hound/)

### GUI
- [egui documentation](https://docs.rs/egui/)
- [egui examples](https://github.com/emilk/egui/tree/master/examples)

### Async Rust
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
- [Async Rust book](https://rust-lang.github.io/async-book/)

### OpenAI API
- [Whisper API docs](https://platform.openai.com/docs/guides/speech-to-text)
- [Chat API docs](https://platform.openai.com/docs/guides/chat)

---

**Happy developing! 🦀**