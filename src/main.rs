use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

mod audio_capture;
mod config;
mod summarization;
mod transcription;

use audio_capture::AudioCapture;
use config::Config;
use summarization::{Summarizer, SummaryResult};
use transcription::{Transcriber, TranscriptionResult};

#[derive(Debug, Clone)]
enum AppMessage {
    AudioChunkReady(PathBuf),
    TranscriptionReady(TranscriptionResult),
    SummaryReady(SummaryResult),
    Error(String),
}

struct AudioAssistantApp {
    config: Config,
    audio_capture: Option<AudioCapture>,
    is_listening: bool,

    // Communication channels
    message_tx: Sender<AppMessage>,
    message_rx: Arc<Mutex<Receiver<AppMessage>>>,

    // Transcription state
    transcriptions: Vec<TranscriptionResult>,
    pending_transcriptions: usize,

    // Summary state
    summaries: Vec<SummaryResult>,
    current_summary: Option<SummaryResult>,

    // UI state
    api_key_input: String,
    chunk_duration_input: String,
    status_message: String,
    error_message: String,

    // Live streaming display state
    auto_scroll_enabled: bool,
    show_timestamps: bool,
    show_statistics: bool,
    last_transcription_time: Option<std::time::Instant>,

    // Search/filter state
    search_query: String,
    highlight_search: bool,
}

impl AudioAssistantApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        let api_key_input = config.openai_api_key.clone();
        let chunk_duration_input = config.chunk_duration_secs.to_string();

        let (tx, rx) = channel();

        Self {
            config,
            audio_capture: None,
            is_listening: false,
            message_tx: tx,
            message_rx: Arc::new(Mutex::new(rx)),
            transcriptions: Vec::new(),
            pending_transcriptions: 0,
            summaries: Vec::new(),
            current_summary: None,
            api_key_input,
            chunk_duration_input,
            status_message: "Ready".to_string(),
            error_message: String::new(),
            auto_scroll_enabled: true,
            show_timestamps: true,
            show_statistics: true,
            last_transcription_time: None,
            search_query: String::new(),
            highlight_search: true,
        }
    }

    fn start_listening(&mut self) {
        // Validate config
        if let Err(e) = self.config.validate() {
            self.error_message = format!("Configuration error: {}", e);
            return;
        }

        // Ensure directories exist
        if let Err(e) = self.config.ensure_directories() {
            self.error_message = format!("Failed to create directories: {}", e);
            return;
        }

        // Create audio capture
        let mut capture = match AudioCapture::new(
            self.config.sample_rate,
            self.config.chunk_duration_secs,
            self.config.audio_chunks_dir.clone(),
        ) {
            Ok(c) => c,
            Err(e) => {
                self.error_message = format!("Failed to initialize audio capture: {}", e);
                return;
            }
        };

        let tx = self.message_tx.clone();

        // Start recording
        if let Err(e) = capture.start_recording(move |audio_file| {
            let _ = tx.send(AppMessage::AudioChunkReady(audio_file));
        }) {
            self.error_message = format!("Failed to start recording: {}", e);
            return;
        }

        self.audio_capture = Some(capture);
        self.is_listening = true;
        self.status_message = "Listening...".to_string();
        self.error_message.clear();

        println!("Started listening for audio");
    }

    fn stop_listening(&mut self) {
        if let Some(mut capture) = self.audio_capture.take() {
            if let Err(e) = capture.stop_recording() {
                self.error_message = format!("Error stopping recording: {}", e);
            }
        }

        self.is_listening = false;
        self.status_message = "Stopped".to_string();
        println!("Stopped listening");
    }

    fn process_messages(&mut self) {
        // Collect messages first to avoid borrow checker issues
        let mut messages = Vec::new();
        {
            let rx = self.message_rx.lock().unwrap();
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        // Process messages after releasing the lock
        for msg in messages {
            match msg {
                AppMessage::AudioChunkReady(audio_file) => {
                    self.handle_audio_chunk(audio_file);
                }
                AppMessage::TranscriptionReady(result) => {
                    self.handle_transcription(result);
                }
                AppMessage::SummaryReady(result) => {
                    self.handle_summary(result);
                }
                AppMessage::Error(error) => {
                    self.error_message = error;
                }
            }
        }
    }

    fn handle_audio_chunk(&mut self, audio_file: PathBuf) {
        println!("Processing audio chunk: {:?}", audio_file);
        self.pending_transcriptions += 1;
        self.status_message = format!("Processing {} audio chunks...", self.pending_transcriptions);

        let api_key = self.config.openai_api_key.clone();
        let transcriptions_dir = self.config.transcriptions_dir.clone();
        let keep_audio = self.config.keep_audio_files;
        let tx = self.message_tx.clone();

        // Spawn async task for transcription
        tokio::spawn(async move {
            let transcriber = Transcriber::new(api_key);

            match transcriber.transcribe(audio_file.clone()).await {
                Ok(result) => {
                    // Save transcription
                    if let Err(e) = transcriber
                        .save_transcription(&result, &transcriptions_dir)
                        .await
                    {
                        let _ = tx.send(AppMessage::Error(format!(
                            "Failed to save transcription: {}",
                            e
                        )));
                    }

                    // Delete audio file if configured
                    if !keep_audio {
                        let _ = tokio::fs::remove_file(&audio_file).await;
                    }

                    let _ = tx.send(AppMessage::TranscriptionReady(result));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Transcription failed: {}", e)));
                }
            }
        });
    }

    fn handle_transcription(&mut self, result: TranscriptionResult) {
        self.pending_transcriptions = self.pending_transcriptions.saturating_sub(1);
        self.transcriptions.push(result.clone());
        self.last_transcription_time = Some(std::time::Instant::now());

        println!("Transcription received: {}", result.text);

        // If real-time processing is enabled, summarize immediately
        if self.config.realtime_processing {
            self.generate_summary();
        } else {
            self.status_message = format!("Transcribed {} segments", self.transcriptions.len());
        }
    }

    fn handle_summary(&mut self, result: SummaryResult) {
        self.summaries.push(result.clone());
        self.current_summary = Some(result);
        self.status_message = "Summary generated".to_string();
    }

    fn generate_summary(&mut self) {
        if self.transcriptions.is_empty() {
            self.error_message = "No transcriptions to summarize".to_string();
            return;
        }

        let api_key = self.config.openai_api_key.clone();
        let model = self.config.summarization_model.clone();
        let summaries_dir = self.config.summaries_dir.clone();
        let tx = self.message_tx.clone();

        let texts: Vec<String> = self.transcriptions.iter().map(|t| t.text.clone()).collect();

        self.status_message = "Generating summary...".to_string();

        tokio::spawn(async move {
            let summarizer = Summarizer::new(api_key, model);

            match summarizer.summarize_conversation(&texts).await {
                Ok(result) => {
                    // Save summary
                    if let Err(e) = summarizer.save_summary(&result, &summaries_dir).await {
                        let _ =
                            tx.send(AppMessage::Error(format!("Failed to save summary: {}", e)));
                    }

                    let _ = tx.send(AppMessage::SummaryReady(result));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Summarization failed: {}", e)));
                }
            }
        });
    }

    fn save_config(&mut self) {
        // Parse chunk duration
        if let Ok(duration) = self.chunk_duration_input.parse::<u64>() {
            self.config.chunk_duration_secs = duration;
        }

        self.config.openai_api_key = self.api_key_input.clone();

        if let Err(e) = self.config.save() {
            self.error_message = format!("Failed to save config: {}", e);
        } else {
            self.status_message = "Configuration saved".to_string();
        }
    }

    fn export_transcript_txt(&mut self) {
        if self.transcriptions.is_empty() {
            self.error_message = "No transcriptions to export".to_string();
            return;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("transcript_{}.txt", timestamp);
        let filepath = self.config.transcriptions_dir.join(&filename);

        let mut content = String::new();
        content.push_str("=== AUDIO ASSISTANT TRANSCRIPT ===\n");
        content.push_str(&format!(
            "Generated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        content.push_str(&format!("Total segments: {}\n", self.transcriptions.len()));

        // Add statistics
        let total_text: String = self
            .transcriptions
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let word_count = total_text.split_whitespace().count();
        let char_count = total_text.chars().count();
        content.push_str(&format!("Word count: {}\n", word_count));
        content.push_str(&format!("Character count: {}\n\n", char_count));
        content.push_str("=====================================\n\n");

        for (i, trans) in self.transcriptions.iter().enumerate() {
            content.push_str(&format!(
                "[Segment {}] {}\n",
                i + 1,
                trans.timestamp.format("%H:%M:%S")
            ));
            content.push_str(&trans.text);
            content.push_str("\n\n");
        }

        match std::fs::write(&filepath, content) {
            Ok(_) => {
                self.status_message = format!("Transcript exported to: {:?}", filename);
                println!("Transcript exported to: {:?}", filepath);
            }
            Err(e) => {
                self.error_message = format!("Failed to export transcript: {}", e);
            }
        }
    }

    fn export_transcript_markdown(&mut self) {
        if self.transcriptions.is_empty() {
            self.error_message = "No transcriptions to export".to_string();
            return;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("transcript_{}.md", timestamp);
        let filepath = self.config.transcriptions_dir.join(&filename);

        let mut content = String::new();
        content.push_str("# Audio Assistant Transcript\n\n");
        content.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Add statistics
        let total_text: String = self
            .transcriptions
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let word_count = total_text.split_whitespace().count();
        let char_count = total_text.chars().count();

        content.push_str("## Statistics\n\n");
        content.push_str(&format!("- **Segments:** {}\n", self.transcriptions.len()));
        content.push_str(&format!("- **Words:** {}\n", word_count));
        content.push_str(&format!("- **Characters:** {}\n\n", char_count));

        if let Some(first) = self.transcriptions.first() {
            if let Some(last) = self.transcriptions.last() {
                let duration = last.timestamp.signed_duration_since(first.timestamp);
                let minutes = duration.num_minutes();
                let seconds = duration.num_seconds() % 60;
                content.push_str(&format!("- **Duration:** {}m {}s\n\n", minutes, seconds));
            }
        }

        content.push_str("---\n\n");
        content.push_str("## Transcript\n\n");

        for (i, trans) in self.transcriptions.iter().enumerate() {
            content.push_str(&format!(
                "### Segment {} `{}`\n\n",
                i + 1,
                trans.timestamp.format("%H:%M:%S")
            ));
            content.push_str(&trans.text);
            content.push_str("\n\n");
        }

        match std::fs::write(&filepath, content) {
            Ok(_) => {
                self.status_message = format!("Transcript exported to: {:?}", filename);
                println!("Transcript exported to: {:?}", filepath);
            }
            Err(e) => {
                self.error_message = format!("Failed to export transcript: {}", e);
            }
        }
    }
}

impl eframe::App for AudioAssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending messages
        self.process_messages();

        // Request continuous repaint to process messages
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎙️ Audio Assistant");
            ui.add_space(10.0);

            // Configuration section
            ui.collapsing("⚙️ Configuration", |ui| {
                ui.horizontal(|ui| {
                    ui.label("OpenAI API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.api_key_input)
                            .password(true)
                            .hint_text("sk-..."),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Chunk Duration (seconds):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.chunk_duration_input).hint_text("30"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.config.keep_audio_files, "Keep audio files");
                    ui.checkbox(&mut self.config.realtime_processing, "Real-time processing");
                });

                if ui.button("💾 Save Configuration").clicked() {
                    self.save_config();
                }

                ui.add_space(5.0);
                ui.label(format!("Audio chunks: {:?}", self.config.audio_chunks_dir));
                ui.label(format!(
                    "Transcriptions: {:?}",
                    self.config.transcriptions_dir
                ));
                ui.label(format!("Summaries: {:?}", self.config.summaries_dir));
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Main control
            ui.horizontal(|ui| {
                let button_text = if self.is_listening {
                    "⏹ Stop Listening"
                } else {
                    "🎤 Start Listening"
                };
                let button_color = if self.is_listening {
                    egui::Color32::from_rgb(220, 50, 50)
                } else {
                    egui::Color32::from_rgb(50, 150, 50)
                };

                let button = egui::Button::new(button_text)
                    .fill(button_color)
                    .min_size(egui::vec2(200.0, 40.0));

                if ui.add(button).clicked() {
                    if self.is_listening {
                        self.stop_listening();
                    } else {
                        self.start_listening();
                    }
                }

                if !self.is_listening && !self.transcriptions.is_empty() {
                    if ui.button("📝 Generate Summary").clicked() {
                        self.generate_summary();
                    }
                }

                if ui.button("🗑 Clear All").clicked() {
                    self.transcriptions.clear();
                    self.summaries.clear();
                    self.current_summary = None;
                    self.status_message = "Cleared".to_string();
                }

                if !self.transcriptions.is_empty() {
                    ui.menu_button("💾 Export Transcript", |ui| {
                        if ui.button("📄 Plain Text (.txt)").clicked() {
                            self.export_transcript_txt();
                            ui.close_menu();
                        }
                        if ui.button("📝 Markdown (.md)").clicked() {
                            self.export_transcript_markdown();
                            ui.close_menu();
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Status
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
            });

            if !self.error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", self.error_message));
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Live Streaming Transcription Display
            if self.is_listening || !self.transcriptions.is_empty() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("📺 Live Transcript");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut self.show_statistics, "📊 Stats");
                            ui.checkbox(&mut self.show_timestamps, "🕐 Timestamps");
                            ui.checkbox(&mut self.auto_scroll_enabled, "⬇ Auto-scroll");
                        });
                    });

                    // Search bar
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search transcript...")
                                .desired_width(300.0),
                        );

                        if !self.search_query.is_empty() {
                            if ui.button("✖").clicked() {
                                self.search_query.clear();
                            }
                            ui.checkbox(&mut self.highlight_search, "Highlight");

                            // Count matches
                            let match_count: usize = self
                                .transcriptions
                                .iter()
                                .filter(|t| {
                                    t.text
                                        .to_lowercase()
                                        .contains(&self.search_query.to_lowercase())
                                })
                                .count();

                            if match_count > 0 {
                                ui.label(
                                    egui::RichText::new(format!("{} matches", match_count))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(50, 150, 50)),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("No matches")
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(120)),
                                );
                            }
                        }
                    });

                    // Statistics display
                    if self.show_statistics && !self.transcriptions.is_empty() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            let total_text: String = self
                                .transcriptions
                                .iter()
                                .map(|t| t.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");
                            let word_count = total_text.split_whitespace().count();
                            let char_count = total_text.chars().count();

                            ui.label(
                                egui::RichText::new(format!("📝 {} words", word_count))
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(100)),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!("🔤 {} characters", char_count))
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(100)),
                            );

                            if let Some(first) = self.transcriptions.first() {
                                if let Some(last) = self.transcriptions.last() {
                                    let duration =
                                        last.timestamp.signed_duration_since(first.timestamp);
                                    let minutes = duration.num_minutes();
                                    let seconds = duration.num_seconds() % 60;
                                    ui.separator();
                                    ui.label(
                                        egui::RichText::new(format!("⏱ {}m {}s", minutes, seconds))
                                            .size(12.0)
                                            .color(egui::Color32::from_gray(100)),
                                    );
                                }
                            }
                        });
                    }

                    ui.separator();

                    let scroll_area = egui::ScrollArea::vertical()
                        .max_height(350.0)
                        .auto_shrink([false, false])
                        .stick_to_bottom(self.auto_scroll_enabled);

                    scroll_area.show(ui, |ui| {
                        if self.transcriptions.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(100.0);
                                ui.label(
                                    egui::RichText::new("🎙️ Waiting for transcriptions...")
                                        .size(16.0)
                                        .color(egui::Color32::GRAY),
                                );
                            });
                        } else {
                            // Filter transcriptions based on search query
                            let search_lower = self.search_query.to_lowercase();
                            let filtered: Vec<(usize, &TranscriptionResult)> = self
                                .transcriptions
                                .iter()
                                .enumerate()
                                .filter(|(_, t)| {
                                    self.search_query.is_empty()
                                        || t.text.to_lowercase().contains(&search_lower)
                                })
                                .collect();

                            if filtered.is_empty() && !self.search_query.is_empty() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(100.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "🔍 No results found for \"{}\"",
                                            self.search_query
                                        ))
                                        .size(14.0)
                                        .color(egui::Color32::GRAY),
                                    );
                                });
                            } else {
                                for (i, trans) in filtered {
                                    // Calculate fade-in effect for recent transcriptions
                                    let is_new =
                                        if let Some(last_time) = self.last_transcription_time {
                                            let elapsed = last_time.elapsed().as_secs_f32();
                                            i == self.transcriptions.len() - 1 && elapsed < 2.0
                                        } else {
                                            false
                                        };

                                    // Check if this matches the search
                                    let matches_search = !self.search_query.is_empty()
                                        && trans.text.to_lowercase().contains(&search_lower);

                                    let frame = if is_new {
                                        egui::Frame::none()
                                            .fill(egui::Color32::from_rgba_unmultiplied(
                                                50, 150, 50, 30,
                                            ))
                                            .inner_margin(8.0)
                                            .rounding(4.0)
                                    } else if matches_search && self.highlight_search {
                                        egui::Frame::none()
                                            .fill(egui::Color32::from_rgba_unmultiplied(
                                                255, 255, 150, 100,
                                            ))
                                            .inner_margin(8.0)
                                            .rounding(4.0)
                                    } else {
                                        egui::Frame::none()
                                            .fill(egui::Color32::from_gray(240))
                                            .inner_margin(8.0)
                                            .rounding(4.0)
                                    };

                                    frame.show(ui, |ui| {
                                        if self.show_timestamps {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("#{}", i + 1))
                                                        .size(11.0)
                                                        .color(egui::Color32::from_gray(120)),
                                                );
                                                ui.label(
                                                    egui::RichText::new(
                                                        trans
                                                            .timestamp
                                                            .format("%H:%M:%S")
                                                            .to_string(),
                                                    )
                                                    .size(11.0)
                                                    .color(egui::Color32::from_gray(120))
                                                    .monospace(),
                                                );
                                            });
                                        }

                                        // Display text with search highlighting
                                        if matches_search
                                            && self.highlight_search
                                            && !self.search_query.is_empty()
                                        {
                                            // Simple highlighting by making matched text bold
                                            ui.label(
                                                egui::RichText::new(&trans.text)
                                                    .size(14.0)
                                                    .strong(),
                                            );
                                        } else {
                                            ui.label(egui::RichText::new(&trans.text).size(14.0));
                                        }
                                    });

                                    ui.add_space(6.0);
                                }
                            }
                        }
                    });

                    // Status bar with copy button
                    ui.separator();
                    ui.horizontal(|ui| {
                        if self.is_listening {
                            ui.label(
                                egui::RichText::new("● LIVE")
                                    .color(egui::Color32::from_rgb(220, 50, 50))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("● STOPPED")
                                    .color(egui::Color32::from_gray(120)),
                            );
                        }
                        ui.separator();
                        ui.label(format!("{} segments", self.transcriptions.len()));

                        if self.pending_transcriptions > 0 {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!(
                                    "⏳ Processing: {}",
                                    self.pending_transcriptions
                                ))
                                .color(egui::Color32::from_rgb(200, 150, 50)),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !self.transcriptions.is_empty() {
                                if ui.button("📋 Copy All").clicked() {
                                    let full_text: String = self
                                        .transcriptions
                                        .iter()
                                        .map(|t| t.text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("\n\n");
                                    ui.output_mut(|o| o.copied_text = full_text);
                                    self.status_message =
                                        "Transcript copied to clipboard".to_string();
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);
            }

            // Collapsible detailed transcriptions section
            ui.collapsing(
                format!("📝 Detailed Transcriptions ({})", self.transcriptions.len()),
                |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, trans) in self.transcriptions.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.label(format!(
                                        "Segment {} - {}",
                                        i + 1,
                                        trans.timestamp.format("%H:%M:%S")
                                    ));
                                    ui.label(&trans.text);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "File: {:?}",
                                            trans.audio_file.file_name().unwrap_or_default()
                                        ))
                                        .size(10.0)
                                        .color(egui::Color32::from_gray(120)),
                                    );
                                });
                                ui.add_space(5.0);
                            }
                        });
                },
            );

            ui.add_space(10.0);

            // Summary section
            if let Some(summary) = &self.current_summary {
                ui.collapsing("📊 Latest Summary", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            ui.group(|ui| {
                                ui.heading("Summary");
                                ui.label(&summary.summary);
                            });

                            ui.add_space(10.0);

                            if !summary.action_items.is_empty() {
                                ui.group(|ui| {
                                    ui.heading("Action Items");
                                    for (i, item) in summary.action_items.iter().enumerate() {
                                        ui.label(format!("{}. {}", i + 1, item));
                                    }
                                });
                            }
                        });
                });
            }

            ui.add_space(20.0);

            // Help text
            ui.collapsing("ℹ️ Help", |ui| {
                ui.label("How to use:");
                ui.label("1. Set your OpenAI API key in the configuration section");
                ui.label("2. Click 'Start Listening' to begin recording system audio");
                ui.label("3. Audio will be captured in chunks and transcribed automatically");
                ui.label(
                    "4. If real-time processing is enabled, summaries are generated automatically",
                );
                ui.label("5. Click 'Stop Listening' when done");
                ui.add_space(5.0);
                ui.label("Note: On Linux, you may need to configure PulseAudio or PipeWire");
                ui.label("to capture system audio (loopback device).");
            });
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 1000.0])
            .with_title("Audio Assistant"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "Audio Assistant",
        options,
        Box::new(|cc| Box::new(AudioAssistantApp::new(cc))),
    );

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
