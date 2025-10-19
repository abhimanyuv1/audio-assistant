use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct AudioCapture {
    host: Host,
    device: Option<Device>,
    stream: Option<Stream>,
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
    chunk_duration: Duration,
    output_dir: PathBuf,
}

impl AudioCapture {
    #[allow(dead_code)]
    pub fn new(sample_rate: u32, chunk_duration_secs: u64, output_dir: PathBuf) -> Result<Self> {
        let host = cpal::default_host();

        Ok(Self {
            host,
            device: None,
            stream: None,
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate,
            chunk_duration: Duration::from_secs(chunk_duration_secs),
            output_dir,
        })
    }

    /// Get the default input device (microphone or system audio)
    pub fn get_default_device(&mut self) -> Result<()> {
        // Try to get default input device
        let device = self
            .host
            .default_input_device()
            .context("No default input device found. You may need to configure PulseAudio/PipeWire to capture system audio.")?;

        println!("Using audio device: {}", device.name()?);
        self.device = Some(device);
        Ok(())
    }

    /// List all available audio devices
    #[allow(dead_code)]
    pub fn list_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();

        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    /// Start recording audio in chunks
    pub fn start_recording<F>(&mut self, on_chunk_ready: F) -> Result<()>
    where
        F: Fn(PathBuf) + Send + 'static,
    {
        if self.is_recording.load(Ordering::SeqCst) {
            anyhow::bail!("Already recording");
        }

        if self.device.is_none() {
            self.get_default_device()?;
        }

        let device = self.device.as_ref().unwrap();

        // Get supported config
        let config = device.default_input_config()?;
        println!("Default input config: {:?}", config);

        // Create stream config with our desired sample rate
        let stream_config = StreamConfig {
            channels: 1, // Mono audio
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let is_recording = Arc::clone(&self.is_recording);
        is_recording.store(true, Ordering::SeqCst);

        let sample_rate = self.sample_rate;
        let chunk_duration = self.chunk_duration;
        let output_dir = self.output_dir.clone();

        // Shared buffer for collecting samples
        let samples_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = Arc::clone(&samples_buffer);

        // Spawn a thread to handle chunk writing
        let is_recording_clone = Arc::clone(&is_recording);
        thread::spawn(move || {
            let chunk_samples = (sample_rate as u64 * chunk_duration.as_secs()) as usize;

            while is_recording_clone.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));

                let mut buffer = samples_clone.lock().unwrap();
                if buffer.len() >= chunk_samples {
                    // Extract chunk
                    let chunk: Vec<f32> = buffer.drain(..chunk_samples).collect();
                    drop(buffer); // Release lock

                    // Write chunk to file
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let filename = format!("chunk_{}.wav", timestamp);
                    let filepath = output_dir.join(filename);

                    if let Err(e) = write_wav_file(&filepath, &chunk, sample_rate) {
                        eprintln!("Error writing audio chunk: {}", e);
                    } else {
                        println!("Audio chunk saved: {:?}", filepath);
                        on_chunk_ready(filepath);
                    }
                }
            }

            // Write remaining samples when stopped
            let mut buffer = samples_clone.lock().unwrap();
            if !buffer.is_empty() {
                let chunk: Vec<f32> = buffer.drain(..).collect();
                drop(buffer);

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let filename = format!("chunk_{}_final.wav", timestamp);
                let filepath = output_dir.join(filename);

                if let Err(e) = write_wav_file(&filepath, &chunk, sample_rate) {
                    eprintln!("Error writing final audio chunk: {}", e);
                } else {
                    println!("Final audio chunk saved: {:?}", filepath);
                    on_chunk_ready(filepath);
                }
            }
        });

        // Build the input stream
        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                self.build_stream::<i16>(device, &stream_config, samples_buffer)?
            }
            SampleFormat::U16 => {
                self.build_stream::<u16>(device, &stream_config, samples_buffer)?
            }
            SampleFormat::F32 => {
                self.build_stream::<f32>(device, &stream_config, samples_buffer)?
            }
            format => anyhow::bail!("Unsupported sample format: {:?}", format),
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<()> {
        if !self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        // Give time for the final chunk to be written
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    /// Check if currently recording
    #[allow(dead_code)]
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Build input stream for a specific sample type
    fn build_stream<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        samples_buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let err_fn = |err| eprintln!("Stream error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buffer = samples_buffer.lock().unwrap();
                for &sample in data {
                    buffer.push(sample.to_sample::<f32>());
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}

/// Write samples to a WAV file
fn write_wav_file(path: &PathBuf, samples: &[f32], sample_rate: u32) -> Result<()> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)?;

    for &sample in samples {
        // Convert f32 (-1.0 to 1.0) to i16
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }

    writer.finalize()?;
    Ok(())
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        let _ = self.stop_recording();
    }
}
