# Audio Assistant - Complete Setup Instructions 🎙️

## What We Built

A Rust desktop application that:
1. **Captures system audio** in real-time (meetings, videos, calls)
2. **Transcribes audio** automatically using OpenAI Whisper API
3. **Generates summaries** with action items using GPT-4o-mini
4. **Saves everything** to local JSON files for later review

---

## Project Overview

### Technology Stack
- **Language**: Rust
- **GUI**: egui (immediate-mode GUI framework)
- **Audio**: cpal (cross-platform audio library)
- **APIs**: OpenAI Whisper (transcription) + GPT (summarization)
- **Async Runtime**: Tokio
- **Platform**: Linux (with PulseAudio/PipeWire)

### Project Structure
```
audio-assistant/
├── src/
│   ├── main.rs              # GUI + orchestration
│   ├── config.rs            # Settings management
│   ├── audio_capture.rs     # Audio recording
│   ├── transcription.rs     # Whisper API
│   └── summarization.rs     # GPT API
├── Cargo.toml               # Dependencies
├── README.md                # Full documentation
├── QUICKSTART.md            # 5-minute guide
├── TROUBLESHOOTING.md       # Problem solving
├── DEVELOPMENT.md           # Developer guide
└── setup-audio.sh           # Audio setup helper
```

---

## Complete Setup Process

### Step 1: OpenAI API Key Setup

This is the MOST IMPORTANT step - you need this to run the app!

#### A. Create OpenAI Account
1. Go to https://platform.openai.com/
2. Sign up or log in
3. You may need to verify your email/phone

#### B. Add Payment Method
1. Go to https://platform.openai.com/account/billing
2. Click "Add payment method"
3. Add a credit/debit card
4. **Optional**: Set a spending limit (recommended: $10-20/month)

#### C. Generate API Key
1. Go to https://platform.openai.com/api-keys
2. Click "Create new secret key"
3. Give it a name (e.g., "Audio Assistant")
4. Click "Create secret key"
5. **IMPORTANT**: Copy the key immediately (starts with `sk-`)
6. Store it safely - you won't be able to see it again!

#### D. Verify Your Setup
Test your API key with curl:
```bash
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer YOUR_API_KEY_HERE"
```

If you see a JSON response with model names, you're good to go!

#### E. Understanding Costs
- **Whisper API**: $0.006 per minute of audio
- **GPT-4o-mini**: ~$0.15 per 1M tokens (very cheap for summaries)
- **Example**: 1-hour meeting ≈ $0.40-0.60 total

Check usage anytime at: https://platform.openai.com/usage

---

### Step 2: Build the Application

```bash
# Navigate to project directory
cd rust-course/audio-assistant

# Install system dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libasound2-dev libssl-dev

# For Fedora:
# sudo dnf install gcc pkg-config alsa-lib-devel openssl-devel

# For Arch:
# sudo pacman -S base-devel alsa-lib openssl

# Build in release mode (optimized)
cargo build --release

# This will take 2-3 minutes on first build
```

---

### Step 3: Configure the Application

```bash
# Run the application
cargo run --release
```

When the GUI opens:

1. Look for the **"⚙️ Configuration"** section (click to expand)
2. In the **"OpenAI API Key"** field, paste your API key (starts with `sk-`)
3. Leave other settings as default for now:
   - Chunk Duration: 30 seconds
   - Keep audio files: unchecked
   - Real-time processing: checked
4. Click **"💾 Save Configuration"**
5. You should see "Configuration saved" in the status

Your API key is now stored in: `~/.config/audio-assistant/config.json`

**Security Note**: Keep this file private! It contains your API key.
```bash
chmod 600 ~/.config/audio-assistant/config.json
```

---

### Step 4: Linux Audio Setup (CRITICAL!)

By default, the app captures from your **microphone**. To capture **system audio** (what your computer plays), you need to set up audio loopback.

#### Option A: Use the Setup Script (Easiest)
```bash
./setup-audio.sh
```

Follow the on-screen instructions.

#### Option B: Manual Setup (PulseAudio)
```bash
# 1. Load loopback module
pactl load-module module-loopback latency_msec=1

# 2. Install audio control panel
sudo apt install pavucontrol

# 3. Run pavucontrol
pavucontrol
```

In pavucontrol:
- Click "Start Listening" in the Audio Assistant app first
- Switch to the **"Recording"** tab in pavucontrol
- Find **"ALSA plug-in [audio-assistant]"** or similar
- Click the dropdown menu (currently shows "Built-in Audio")
- Select **"Monitor of [your output device]"** (e.g., "Monitor of Built-in Audio Analog Stereo")
- Now your app will record what your computer plays!

#### Option C: Manual Setup (PipeWire)
PipeWire is compatible with PulseAudio tools, so use Option A or B above.

#### Verify Audio Setup
1. Keep the Audio Assistant app running
2. Click "Start Listening"
3. Play a YouTube video or music
4. You should see audio chunks being created
5. After 30 seconds, transcription should appear

---

### Step 5: First Test Run

Let's verify everything works!

1. **Start the app** (if not already running):
   ```bash
   cargo run --release
   ```

2. **Click "🎤 Start Listening"** (button turns red)

3. **Play test audio**:
   - Open YouTube
   - Search for "short motivational speech" or "ted talk"
   - Play for 1-2 minutes

4. **Watch for results**:
   - Status shows "Listening..."
   - After ~30 seconds, you'll see "Processing audio chunks..."
   - Soon after, transcription appears in "📝 Transcriptions" section
   - If real-time processing is enabled, summary appears automatically

5. **Click "⏹ Stop Listening"** when done

6. **Review output**:
   - Check transcriptions (should match the video content)
   - Check summary and action items
   - Look in status messages for any errors

---

## Understanding the Workflow

### What Happens When You "Start Listening"?

```
1. Audio Capture (continuous)
   └─→ System audio captured at 16kHz
   └─→ Buffered in 30-second chunks
   └─→ Saved as WAV files

2. Transcription (per chunk)
   └─→ WAV file sent to OpenAI Whisper API
   └─→ Returns text transcription
   └─→ Saved to JSON file
   └─→ Displayed in GUI

3. Summarization (real-time or on-demand)
   └─→ All transcriptions combined
   └─→ Sent to GPT-4o-mini
   └─→ Returns summary + action items
   └─→ Saved to JSON file
   └─→ Displayed in GUI
```

### File Locations

All files are saved to `~/.local/share/audio-assistant/`:

```bash
# View your files
ls -lh ~/.local/share/audio-assistant/audio_chunks/
ls -lh ~/.local/share/audio-assistant/transcriptions/
ls -lh ~/.local/share/audio-assistant/summaries/

# Example transcription file
cat ~/.local/share/audio-assistant/transcriptions/transcription_20240115_143022.json
```

---

## Configuration Options Explained

### Chunk Duration (seconds)
- **What it does**: Length of each audio segment before transcription
- **Default**: 30 seconds
- **Recommendations**:
  - 30s: Good for real-time, frequent updates
  - 60s: Better context, fewer API calls, lower cost
  - 120s: Maximum context, cheapest, but delayed updates

### Keep Audio Files
- **What it does**: Saves raw WAV files after transcription
- **Default**: Off (disabled)
- **Enable if**: You want to review original audio later
- **Disable if**: You want to save disk space (recommended)

### Real-time Processing
- **What it does**: Auto-generates summaries as transcriptions complete
- **Default**: On (enabled)
- **Enable for**: Live meeting notes, immediate insights
- **Disable for**: Batch processing, manual control, cost savings

### Summarization Model
- **Default**: gpt-4o-mini (fast and cheap)
- **Alternatives**:
  - `gpt-3.5-turbo` - Even cheaper, slightly lower quality
  - `gpt-4o` - More expensive, better quality
  - `gpt-4-turbo` - Highest quality, highest cost

---

## Common Use Cases

### 1. Recording Meeting Notes
```bash
# Before meeting:
1. Start Audio Assistant
2. Click "Start Listening"
3. Join your meeting (Zoom, Teams, etc.)

# During meeting:
- Transcriptions appear automatically
- Summaries generated in real-time

# After meeting:
1. Click "Stop Listening"
2. Review summary and action items
3. Find detailed transcriptions in JSON files
```

### 2. Transcribing Podcast/Video
```bash
# Set longer chunks for better context
1. Change "Chunk Duration" to 60 seconds
2. Disable "Real-time Processing" (optional)
3. Click "Start Listening"
4. Play your video/podcast
5. When done, click "Stop Listening"
6. Click "Generate Summary" for final summary
```

### 3. Lecture Notes
```bash
# Optimized for long-form content
1. Set "Chunk Duration" to 60-120 seconds
2. Keep "Real-time Processing" enabled
3. Start listening before lecture
4. Get automatic summaries of key points
```

---

## Troubleshooting Common Issues

### "No default input device found"
**Problem**: Can't find audio device

**Fix**:
```bash
# Check if audio server is running
systemctl --user status pulseaudio

# Restart if needed
systemctl --user restart pulseaudio

# List available devices
pactl list sources short
```

### "OpenAI API key is not set"
**Problem**: App won't start without API key

**Fix**: Enter your API key in Configuration section and click "Save Configuration"

### Only Recording Microphone, Not System Audio
**Problem**: App records your voice but not computer audio

**Fix**: You need to set up audio loopback (Step 4 above)
```bash
# Quick fix
pactl load-module module-loopback latency_msec=1
pavucontrol  # Then change recording device to Monitor
```

### "Transcription failed: 401 Unauthorized"
**Problem**: Invalid API key

**Fix**: 
1. Verify your API key at https://platform.openai.com/api-keys
2. Regenerate if necessary
3. Update in app configuration

### "Transcription failed: 429 Rate Limit"
**Problem**: Too many requests or no credits

**Fix**:
1. Check usage: https://platform.openai.com/usage
2. Add credits if needed
3. Wait a moment and try again
4. Increase chunk duration to reduce API calls

### Empty or Incorrect Transcriptions
**Problem**: Transcription doesn't match audio

**Fix**:
1. Verify audio is being captured (check WAV file sizes)
2. Ensure volume is adequate (not too quiet)
3. Check you're using Monitor device, not microphone
4. Audio must contain speech (Whisper doesn't work on music)

---

## Next Steps

### Learn More
- Read **QUICKSTART.md** for a 5-minute tutorial
- Check **TROUBLESHOOTING.md** for detailed problem solving
- See **DEVELOPMENT.md** if you want to modify the code
- Review **README.md** for complete documentation

### Customize Your Experience
1. **Adjust chunk duration** based on your use case
2. **Try different GPT models** for summaries
3. **Modify prompts** in `src/summarization.rs`
4. **Add export features** (Markdown, PDF, etc.)

### Cost Optimization Tips
1. Increase chunk duration (fewer API calls)
2. Use batch mode instead of real-time
3. Only record important meetings
4. Set spending limits in OpenAI dashboard

---

## Quick Reference

### Commands
```bash
# Build
cargo build --release

# Run
cargo run --release

# Run with debug logs
RUST_LOG=debug cargo run --release

# Audio setup
./setup-audio.sh

# Check config
cat ~/.config/audio-assistant/config.json

# View transcriptions
ls ~/.local/share/audio-assistant/transcriptions/

# View summaries
ls ~/.local/share/audio-assistant/summaries/
```

### Keyboard Shortcuts
- Currently none (GUI-only)
- Can be added in `src/main.rs` (see DEVELOPMENT.md)

### File Formats
- **Audio**: WAV (16-bit, 16kHz, mono)
- **Transcriptions**: JSON
- **Summaries**: JSON
- **Config**: JSON

---

## Getting Help

If you encounter issues:

1. **Check logs**: Run with `RUST_LOG=debug`
2. **Verify audio setup**: Use `./setup-audio.sh`
3. **Test API key**: Use curl to verify it works
4. **Read troubleshooting**: See TROUBLESHOOTING.md
5. **Check OpenAI status**: https://status.openai.com/

---

## Success Checklist

- [ ] Rust installed and up to date
- [ ] System dependencies installed
- [ ] OpenAI account created
- [ ] Payment method added to OpenAI
- [ ] API key generated and tested
- [ ] Project builds successfully
- [ ] API key configured in app
- [ ] Audio loopback set up
- [ ] Test recording successful
- [ ] Transcription working
- [ ] Summary generated

---

**You're all set! Start capturing and transcribing audio! 🎤✨**

For questions or issues, refer to the documentation files in this directory.