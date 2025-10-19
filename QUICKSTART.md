# Quick Start Guide 🚀

Get up and running with Audio Assistant in 5 minutes!

## Step 1: Get Your OpenAI API Key

1. Go to https://platform.openai.com/
2. Sign in or create an account
3. Navigate to **API Keys** (in left sidebar)
4. Click **"Create new secret key"**
5. Copy the key (starts with `sk-`)
6. Keep it safe! You'll need it in Step 3

**Note**: Make sure you have credits in your OpenAI account. Check at https://platform.openai.com/usage

## Step 2: Build the Application

```bash
cd rust-course/audio-assistant
cargo build --release
```

This will take a few minutes the first time.

## Step 3: Run and Configure

```bash
cargo run --release
```

When the app opens:

1. Click on **"⚙️ Configuration"** to expand it
2. Paste your OpenAI API key in the **"OpenAI API Key"** field
3. Click **"💾 Save Configuration"**
4. You should see "Configuration saved" in the status

## Step 4: Set Up System Audio Capture (Linux)

To capture system audio instead of just your microphone:

### Quick Setup (PulseAudio)

```bash
# Load loopback module
pactl load-module module-loopback latency_msec=1

# Install and run audio mixer
sudo apt install pavucontrol  # Ubuntu/Debian
pavucontrol
```

In `pavucontrol`:
- Go to the **"Recording"** tab
- Find **"ALSA plug-in [audio-assistant]"** 
- Change from "Built-in Audio" to **"Monitor of [your output device]"**

### Quick Setup (PipeWire)

```bash
# Install control panel
sudo apt install pavucontrol  # Works with PipeWire too!
pavucontrol
```

Same steps as above - PipeWire is compatible with PulseAudio tools.

## Step 5: Start Listening!

1. Click the **"🎤 Start Listening"** button (turns red)
2. Play some audio (YouTube video, podcast, meeting call, etc.)
3. Watch the **"📺 Live Transcript"** panel - transcriptions appear in real-time!
   - New segments fade in with a green highlight
   - Panel auto-scrolls to show the latest content
   - See live statistics: word count, character count, and duration
4. Wait for summaries to appear in the **Latest Summary** section (if real-time processing is enabled)

## Step 6: Use Live Features

While listening, try these interactive features:

**Search & Filter** 🔍
- Type in the search box to filter transcriptions
- Matching segments are highlighted in yellow
- See match count in real-time

**Statistics** 📊
- Toggle the "Stats" checkbox to show/hide live metrics
- View total words, characters, and session duration

**Timestamps** 🕐
- Toggle timestamps on/off
- Shows segment number and time for each transcription

**Auto-scroll** ⬇
- Uncheck to manually scroll and review earlier content
- Re-enable to jump back to latest transcriptions

## Step 7: Stop and Review

1. Click **"⏹ Stop Listening"** when done
2. Review your transcriptions using the search feature
3. Click **"📋 Copy All"** to copy entire transcript to clipboard
4. Use **"💾 Export Transcript"** to save as:
   - Plain Text (.txt) - simple format with statistics
   - Markdown (.md) - formatted with headers and sections
5. Check out any action items in the summary section

## Files Location

All your data is saved in:
```
~/.local/share/audio-assistant/
├── audio_chunks/        # Audio files (if kept)
├── transcriptions/      # JSON files with transcribed text
└── summaries/          # JSON files with summaries and action items
```

## Test It Out

**Simple Test**:
1. Start Listening
2. Play a YouTube video: "Ted Talk" (any short one)
3. Wait 30-60 seconds
4. Watch the **Live Transcript** panel fill up in real-time
5. Try searching for a word from the video
6. Click "Copy All" or "Export Transcript" to save
7. Look for the auto-generated summary (if real-time processing is enabled)

## Troubleshooting Quick Fixes

### "No default input device found"
```bash
# Check if PulseAudio is running
systemctl --user status pulseaudio

# Restart if needed
systemctl --user restart pulseaudio
```

### "OpenAI API key is not set"
- Make sure you clicked "Save Configuration" after entering your key
- Check: `cat ~/.config/audio-assistant/config.json`

### Not capturing system audio (only microphone)
- You need to set up audio loopback (see Step 4)
- Use `pavucontrol` to route audio properly

### "Transcription failed" errors
- Check you have OpenAI credits: https://platform.openai.com/usage
- Verify your API key is valid
- Check your internet connection

## Usage Tips

### For Meetings
- **Chunk Duration**: 30-60 seconds works best
- **Real-time Processing**: Enable for live notes
- **Live Transcript**: Keep auto-scroll enabled to follow along
- Use search during breaks to find specific topics discussed
- Export as Markdown at the end for sharing
- Start listening before the meeting begins

### For Podcasts/Videos
- **Chunk Duration**: 60 seconds for better context
- **Real-time Processing**: Can disable and summarize at the end
- **Search Feature**: Great for finding specific quotes or topics
- Quality depends on audio clarity
- Export for later reference or note-taking

### For Research Interviews
- **Live Search**: Find topics mentioned earlier without stopping
- **Timestamps**: Track when specific topics were discussed
- **Statistics**: Monitor interview length and content density
- **Export**: Save transcripts for analysis

### To Save Money
- Increase chunk duration (fewer API calls)
- Disable real-time processing
- Only run during important content

## Cost Estimation

**For a 1-hour meeting**:
- Transcription: ~$0.36
- Summarization: ~$0.10-0.20
- **Total**: ~$0.50/hour

Much cheaper than manual transcription services!

## Next Steps

Once you're comfortable:
- Adjust chunk duration in Configuration
- Try batch mode (disable real-time processing)
- Explore the JSON files in `~/.local/share/audio-assistant/`
- Modify the code to suit your needs!

## Need Help?

Check the full [README.md](README.md) for:
- Detailed troubleshooting
- Advanced configuration
- Project architecture
- Contributing guidelines

---

**Happy transcribing! 🎤✨**