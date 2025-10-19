# Audio Assistant 🎙️

A Rust desktop application that captures system audio, transcribes it using OpenAI Whisper, and generates summaries with action items using GPT.

## Features

- 🎤 **Real-time Audio Capture**: Records system audio in configurable chunks
- 📝 **Automatic Transcription**: Uses OpenAI Whisper API to convert audio to text
- 🤖 **AI Summarization**: Generates conversation summaries and extracts action items
- 💾 **Persistent Storage**: Saves transcriptions and summaries to local files
- ⚙️ **Configurable**: Adjustable chunk duration, real-time vs batch processing
- 🖥️ **Simple GUI**: Easy-to-use desktop interface built with egui

## Prerequisites

- Rust (1.70 or later)
- OpenAI API Key (for Whisper and GPT access)
- Linux system with audio support (PulseAudio or PipeWire)

## Installation

1. Clone or navigate to the project directory:
```bash
cd rust-course/audio-assistant
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

## Configuration

### Setting up OpenAI API Key

You need an OpenAI API key to use this application:

1. Go to [OpenAI Platform](https://platform.openai.com/)
2. Sign up or log in to your account
3. Navigate to API Keys section
4. Create a new API key
5. Copy the key (it starts with `sk-`)

**In the application:**
1. Open the "Configuration" section
2. Paste your API key in the "OpenAI API Key" field
3. Click "Save Configuration"

Your API key will be stored locally in: `~/.config/audio-assistant/config.json`

### Configuration Options

- **Chunk Duration**: Length of audio segments in seconds (default: 30)
  - Shorter chunks: More frequent updates, higher API costs
  - Longer chunks: Better context, fewer API calls
  - Recommended: 30-60 seconds

- **Keep Audio Files**: Whether to save raw audio chunks after transcription
  - Enable if you want to review original audio
  - Disable to save disk space

- **Real-time Processing**: Generate summaries automatically as transcriptions complete
  - Enable for live meeting notes
  - Disable to manually trigger summarization when needed

### Storage Locations

By default, files are stored in:
- **Config**: `~/.config/audio-assistant/config.json`
- **Audio Chunks**: `~/.local/share/audio-assistant/audio_chunks/`
- **Transcriptions**: `~/.local/share/audio-assistant/transcriptions/`
- **Summaries**: `~/.local/share/audio-assistant/summaries/`

## Usage

### Basic Workflow

1. **Start the Application**
   ```bash
   cargo run --release
   ```

2. **Configure API Key** (first time only)
   - Open the "Configuration" section
   - Enter your OpenAI API key
   - Click "Save Configuration"

3. **Start Listening**
   - Click the "🎤 Start Listening" button
   - The app will begin capturing system audio
   - Audio is automatically chunked, transcribed, and (optionally) summarized

4. **Monitor Progress**
   - Watch the "Transcriptions" section for real-time text
   - View summaries in the "Latest Summary" section
   - Check action items as they're identified

5. **Stop Listening**
   - Click "⏹ Stop Listening" when done
   - If not in real-time mode, click "📝 Generate Summary" to create final summary

### Use Cases

- **Meeting Notes**: Capture and summarize video calls
- **Lecture Recording**: Transcribe educational content with key points
- **Interview Transcription**: Record conversations with automatic action items
- **Podcast Analysis**: Get summaries and takeaways from audio content

## Linux Audio Setup

### Capturing System Audio

By default, `cpal` captures from the default input device (microphone). To capture system audio, you need to set up an audio loopback:

#### For PulseAudio:

1. Load the loopback module:
```bash
pactl load-module module-loopback latency_msec=1
```

2. Use `pavucontrol` to route audio:
```bash
pavucontrol
```
   - Go to "Recording" tab
   - Find "audio-assistant" and set it to "Monitor of [your output device]"

3. To make it permanent, add to `/etc/pulse/default.pa`:
```
load-module module-loopback latency_msec=1
```

#### For PipeWire:

1. Create a virtual sink:
```bash
pw-cli create-node adapter '{ factory.name=support.null-audio-sink node.name=my-mic media.class=Audio/Sink audio.position=FL,FR }'
```

2. Use `helvum` or `qpwgraph` to route audio visually

3. Or use `pavucontrol` (PipeWire is compatible with PulseAudio tools)

#### Alternative: Use Monitor Device

You can also modify the code to explicitly select a monitor device. Run this to list devices:

```bash
cargo run --release 2>&1 | grep "audio device"
```

Then update `audio_capture.rs` to use the monitor device name.

## Troubleshooting

### "No default input device found"

**Problem**: The application can't find an audio input device.

**Solution**:
- Ensure PulseAudio or PipeWire is running: `systemctl --user status pulseaudio` or `systemctl --user status pipewire`
- Check available devices: `pactl list sources` (PulseAudio) or `pw-cli list-objects` (PipeWire)
- Set up audio loopback as described above

### "OpenAI API key is not set"

**Problem**: No API key configured.

**Solution**:
1. Get an API key from [OpenAI Platform](https://platform.openai.com/)
2. Enter it in the Configuration section
3. Click "Save Configuration"

### "Transcription failed" or "Summarization failed"

**Problem**: API request errors.

**Possible causes**:
- Invalid API key
- Insufficient API credits
- Network connection issues
- API rate limits

**Solution**:
- Check your API key is correct
- Verify you have credits: [OpenAI Usage](https://platform.openai.com/usage)
- Check error details in terminal output
- Wait a moment and try again if rate limited

### Audio Quality Issues

**Problem**: Poor transcription quality.

**Solutions**:
- Increase chunk duration for better context
- Ensure system audio is being captured (not microphone)
- Check audio levels aren't too low or distorted
- Use a higher sample rate (edit `config.json`)

### High API Costs

**Problem**: Too many API calls.

**Solutions**:
- Increase chunk duration (fewer chunks = fewer API calls)
- Disable real-time processing and summarize manually
- Use `gpt-4o-mini` instead of `gpt-4` (already default)
- Only run during important meetings

## Project Structure

```
audio-assistant/
├── src/
│   ├── main.rs              # GUI and main application logic
│   ├── config.rs            # Configuration management
│   ├── audio_capture.rs     # Audio recording functionality
│   ├── transcription.rs     # OpenAI Whisper integration
│   └── summarization.rs     # OpenAI GPT summarization
├── Cargo.toml               # Dependencies
└── README.md                # This file
```

## API Costs

Approximate costs per hour of audio (as of 2024):

- **Whisper API**: $0.006 per minute = ~$0.36 per hour
- **GPT-4o-mini**: ~$0.15 per 1M input tokens (varies by conversation length)

**Example**: A 1-hour meeting in 30-second chunks:
- 120 audio chunks × $0.003 = $0.36 (Whisper)
- ~$0.05-0.20 for summarization (GPT)
- **Total**: ~$0.40-0.60 per hour

## Dependencies

Key Rust crates used:
- `eframe`/`egui` - GUI framework
- `cpal` - Cross-platform audio I/O
- `hound` - WAV file encoding
- `reqwest` - HTTP client for OpenAI API
- `tokio` - Async runtime
- `serde` - Serialization

## Security Notes

- API keys are stored in plain text in `~/.config/audio-assistant/config.json`
- Ensure this file has appropriate permissions: `chmod 600 ~/.config/audio-assistant/config.json`
- Never commit your config file to version control
- Audio files may contain sensitive information - handle appropriately

## Future Enhancements

Potential improvements:
- [ ] Select specific audio input device in GUI
- [ ] Export summaries to different formats (PDF, Markdown, etc.)
- [ ] Local Whisper model support (no API costs)
- [ ] Support for macOS and Windows
- [ ] Custom prompt templates for summarization
- [ ] Speaker diarization (identify different speakers)
- [ ] Live streaming transcription display

## Contributing

This is a learning project. Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Fork and modify for your needs

## License

[Add your license here]

## Acknowledgments

- OpenAI for Whisper and GPT APIs
- egui for the excellent immediate-mode GUI framework
- The Rust audio community for cpal and related tools