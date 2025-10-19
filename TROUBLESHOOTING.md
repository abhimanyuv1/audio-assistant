# Troubleshooting Guide 🔧

Common issues and their solutions for Audio Assistant.

## Table of Contents
- [Installation Issues](#installation-issues)
- [Audio Capture Issues](#audio-capture-issues)
- [API Issues](#api-issues)
- [Performance Issues](#performance-issues)
- [General Issues](#general-issues)

---

## Installation Issues

### Build Fails with Missing Dependencies

**Error**: `error: linker 'cc' not found` or similar

**Solution**:
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libasound2-dev libssl-dev

# Fedora
sudo dnf install gcc pkg-config alsa-lib-devel openssl-devel

# Arch
sudo pacman -S base-devel alsa-lib openssl
```

### Rust Version Too Old

**Error**: `error: package requires rustc 1.70 or newer`

**Solution**:
```bash
# Update Rust
rustup update stable
```

---

## Audio Capture Issues

### "No default input device found"

**Problem**: Can't find any audio input device.

**Diagnosis**:
```bash
# Check if PulseAudio is running
systemctl --user status pulseaudio

# Or check for PipeWire
systemctl --user status pipewire
```

**Solutions**:

1. **Start audio server**:
   ```bash
   # PulseAudio
   systemctl --user start pulseaudio
   
   # PipeWire
   systemctl --user start pipewire pipewire-pulse
   ```

2. **Check available devices**:
   ```bash
   pactl list sources short
   ```

3. **Run the setup script**:
   ```bash
   ./setup-audio.sh
   ```

### Recording Only Captures Microphone, Not System Audio

**Problem**: App records your voice but not computer audio (YouTube, meetings, etc.)

**Cause**: Not using a monitor/loopback device

**Solution**:

1. **Load loopback module**:
   ```bash
   pactl load-module module-loopback latency_msec=1
   ```

2. **Use pavucontrol to route audio**:
   ```bash
   sudo apt install pavucontrol
   pavucontrol
   ```
   
   Steps in pavucontrol:
   - Start the app and click "Start Listening"
   - Go to "Recording" tab
   - Find "audio-assistant" or "ALSA plug-in"
   - Change from "Built-in Audio" to "**Monitor of [output device]**"

3. **Verify it's working**:
   - Play some audio (YouTube video)
   - You should see transcriptions appear
   - If not, try a different monitor source in pavucontrol

### Audio Sounds Choppy or Distorted

**Problem**: Poor quality recordings

**Solutions**:

1. **Increase buffer size** - Edit `src/audio_capture.rs`:
   ```rust
   buffer_size: cpal::BufferSize::Fixed(2048), // Instead of Default
   ```

2. **Adjust latency**:
   ```bash
   pactl load-module module-loopback latency_msec=50  # Higher latency
   ```

3. **Check CPU usage** - Close other applications

4. **Try different sample rate** - In config, try 44100 or 48000 Hz

### No Audio Being Captured (Empty WAV Files)

**Problem**: Files are created but contain no audio

**Diagnosis**:
```bash
# Check file size
ls -lh ~/.local/share/audio-assistant/audio_chunks/

# If files are very small (< 1KB), no audio is being captured
```

**Solutions**:

1. **Check audio is actually playing** through your system

2. **Verify the correct device is selected** in pavucontrol

3. **Check PulseAudio/PipeWire logs**:
   ```bash
   journalctl --user -u pulseaudio -n 50
   ```

4. **Test with arecord**:
   ```bash
   arecord -f cd -d 5 test.wav
   aplay test.wav
   ```

---

## API Issues

### "OpenAI API key is not set"

**Problem**: App won't start listening without API key

**Solutions**:

1. **Set API key in GUI**:
   - Open "Configuration" section
   - Paste your API key (starts with `sk-`)
   - Click "Save Configuration"

2. **Verify it's saved**:
   ```bash
   cat ~/.config/audio-assistant/config.json
   ```

3. **Get a new key** if needed: https://platform.openai.com/api-keys

### "Transcription failed" - Authentication Error

**Error**: `401 Unauthorized` or similar

**Solutions**:

1. **Check API key is valid**:
   ```bash
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer YOUR_API_KEY"
   ```

2. **Regenerate API key** on OpenAI platform

3. **Check for extra spaces** in the key field

### "Transcription failed" - Insufficient Quota

**Error**: `429 Too Many Requests` or quota exceeded

**Problem**: No credits remaining or rate limited

**Solutions**:

1. **Check your usage**: https://platform.openai.com/usage

2. **Add payment method**: https://platform.openai.com/account/billing

3. **Wait if rate limited** (usually 1 minute)

4. **Increase chunk duration** to reduce API calls:
   - Change from 30s to 60s chunks
   - Fewer chunks = fewer API calls

### "Summarization failed" - Model Not Available

**Error**: `model not found` or similar

**Solution**: Change model in config:
```json
{
  "summarization_model": "gpt-4o-mini"
}
```

Or try:
- `gpt-3.5-turbo` (cheaper)
- `gpt-4o` (more expensive, better quality)
- `gpt-4-turbo` (if you have access)

### Network/Connection Errors

**Error**: `Failed to send request` or timeout

**Solutions**:

1. **Check internet connection**:
   ```bash
   ping api.openai.com
   ```

2. **Check firewall**:
   ```bash
   # Test HTTPS access
   curl https://api.openai.com/v1/models
   ```

3. **Proxy settings** - If behind corporate proxy, set:
   ```bash
   export HTTPS_PROXY=http://proxy:port
   cargo run --release
   ```

---

## Performance Issues

### High CPU Usage

**Problem**: App uses too much CPU

**Solutions**:

1. **Build in release mode**:
   ```bash
   cargo build --release
   cargo run --release  # NOT just 'cargo run'
   ```

2. **Increase chunk duration** - Fewer chunks to process

3. **Disable real-time processing**:
   - Uncheck "Real-time processing" in config
   - Generate summaries manually

4. **Close other applications**

### High Memory Usage

**Problem**: Memory usage keeps growing

**Solutions**:

1. **Clear transcriptions periodically** - Click "Clear All" button

2. **Restart the app** for long sessions

3. **Don't keep audio files** - Uncheck "Keep audio files" in config

### Slow Transcription

**Problem**: Takes too long to transcribe

**Cause**: OpenAI API latency (usually 2-10 seconds per chunk)

**Solutions**:

1. **This is normal** - Whisper API takes time

2. **Use batch mode** instead of real-time:
   - Disable "Real-time processing"
   - Let all chunks accumulate
   - Generate summary once at the end

3. **Increase chunk duration** - Fewer, longer chunks

---

## General Issues

### App Crashes on Startup

**Diagnosis**: Run with debug output:
```bash
RUST_LOG=debug cargo run --release 2>&1 | tee debug.log
```

**Common causes**:

1. **Missing audio system**:
   ```bash
   systemctl --user start pulseaudio
   ```

2. **Permission issues**:
   ```bash
   # Check config directory permissions
   ls -la ~/.config/audio-assistant/
   chmod 700 ~/.config/audio-assistant/
   ```

3. **Corrupted config**:
   ```bash
   # Backup and reset
   mv ~/.config/audio-assistant/config.json ~/.config/audio-assistant/config.json.bak
   # Restart app to generate new config
   ```

### "Failed to create directories"

**Problem**: Can't write to file system

**Solution**:
```bash
# Check permissions
mkdir -p ~/.local/share/audio-assistant
chmod 755 ~/.local/share/audio-assistant

# Check disk space
df -h ~
```

### Empty Transcriptions

**Problem**: Transcription completes but text is empty

**Possible causes**:

1. **No audio captured** - See [Audio Capture Issues](#audio-capture-issues)

2. **Audio too quiet** - Increase volume in pavucontrol

3. **Non-speech audio** - Whisper works best with speech

4. **Wrong language** - Whisper auto-detects, but may fail on very short clips

### GUI Not Responding

**Problem**: Interface freezes or doesn't update

**Solutions**:

1. **Wait** - Heavy processing may cause temporary freezes

2. **Check terminal output** for errors

3. **Restart the app**

4. **Build in release mode** (debug mode is very slow)

### Config Changes Not Saved

**Problem**: Settings reset after restart

**Solutions**:

1. **Click "Save Configuration"** button after changes

2. **Check file permissions**:
   ```bash
   ls -la ~/.config/audio-assistant/config.json
   ```

3. **Manually edit config**:
   ```bash
   nano ~/.config/audio-assistant/config.json
   ```

---

## Getting More Help

### Enable Debug Logging

Run with detailed logs:
```bash
RUST_LOG=debug cargo run --release
```

Save logs to file:
```bash
RUST_LOG=debug cargo run --release 2>&1 | tee audio-assistant.log
```

### Check Audio System Status

```bash
# For PulseAudio
pactl info
pactl list sources
pactl list modules | grep loopback

# For PipeWire
pw-cli info
pw-cli list-objects
```

### Test Individual Components

1. **Test audio capture only**:
   ```bash
   arecord -f cd -d 10 test.wav && aplay test.wav
   ```

2. **Test OpenAI API**:
   ```bash
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer YOUR_API_KEY"
   ```

3. **Test file writing**:
   ```bash
   touch ~/.local/share/audio-assistant/test.txt
   ls -la ~/.local/share/audio-assistant/
   ```

### Still Having Issues?

1. **Read the full README.md** for detailed setup instructions

2. **Check GitHub Issues** (if applicable)

3. **Provide debug info**:
   - OS and version: `uname -a`
   - Rust version: `rustc --version`
   - Audio system: `pactl info` or `pw-cli info`
   - Error messages from terminal
   - Content of debug log

---

## Common Error Messages Reference

| Error | Likely Cause | Quick Fix |
|-------|--------------|-----------|
| `No default input device found` | Audio system not running | `systemctl --user start pulseaudio` |
| `OpenAI API key is not set` | No API key configured | Add key in Configuration panel |
| `401 Unauthorized` | Invalid API key | Regenerate key on OpenAI platform |
| `429 Too Many Requests` | Rate limited or no credits | Wait or add credits |
| `Failed to open audio file` | File permissions | Check `~/.local/share/audio-assistant/` permissions |
| `Failed to parse transcription response` | API response format changed | Update the app |
| `Failed to send transcription request` | Network/internet issue | Check connection, proxy settings |
| `Stream error` | Audio device disconnected | Replug device, restart app |

---

**Remember**: Most issues are related to audio configuration on Linux. The `setup-audio.sh` script can help automate the setup!
