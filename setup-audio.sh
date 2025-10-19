#!/bin/bash
# Audio Assistant - Linux Audio Setup Helper Script
# This script helps configure your system to capture audio

set -e

echo "================================"
echo "Audio Assistant - Audio Setup"
echo "================================"
echo ""

# Detect audio system
detect_audio_system() {
    if command -v pw-cli &> /dev/null; then
        if pgrep -x pipewire &> /dev/null; then
            echo "pipewire"
            return
        fi
    fi

    if command -v pactl &> /dev/null; then
        if pgrep -x pulseaudio &> /dev/null; then
            echo "pulseaudio"
            return
        fi
    fi

    echo "unknown"
}

AUDIO_SYSTEM=$(detect_audio_system)

echo "Detected audio system: $AUDIO_SYSTEM"
echo ""

case $AUDIO_SYSTEM in
    pulseaudio)
        echo "Setting up PulseAudio for system audio capture..."
        echo ""

        # Check if loopback module is already loaded
        if pactl list modules short | grep -q module-loopback; then
            echo "✓ Loopback module already loaded"
        else
            echo "Loading loopback module..."
            pactl load-module module-loopback latency_msec=1
            echo "✓ Loopback module loaded"
        fi

        echo ""
        echo "Next steps:"
        echo "1. Install pavucontrol if not already installed:"
        echo "   sudo apt install pavucontrol    # Ubuntu/Debian"
        echo "   sudo dnf install pavucontrol    # Fedora"
        echo "   sudo pacman -S pavucontrol      # Arch"
        echo ""
        echo "2. Run pavucontrol:"
        echo "   pavucontrol"
        echo ""
        echo "3. In pavucontrol:"
        echo "   - Start the Audio Assistant app and click 'Start Listening'"
        echo "   - Go to the 'Recording' tab"
        echo "   - Find the Audio Assistant entry"
        echo "   - Change the device to 'Monitor of [your output device]'"
        echo ""
        echo "To make this permanent, add this to /etc/pulse/default.pa:"
        echo "   load-module module-loopback latency_msec=1"
        ;;

    pipewire)
        echo "PipeWire detected. PipeWire is compatible with PulseAudio tools."
        echo ""
        echo "Setting up audio capture..."

        # PipeWire usually works with PulseAudio commands
        if command -v pactl &> /dev/null; then
            if pactl list modules short | grep -q module-loopback; then
                echo "✓ Loopback module already loaded"
            else
                echo "Loading loopback module..."
                pactl load-module module-loopback latency_msec=1
                echo "✓ Loopback module loaded"
            fi
        fi

        echo ""
        echo "Next steps:"
        echo "1. Install pavucontrol if not already installed:"
        echo "   sudo apt install pavucontrol    # Ubuntu/Debian"
        echo "   sudo dnf install pavucontrol    # Fedora"
        echo "   sudo pacman -S pavucontrol      # Arch"
        echo ""
        echo "2. Run pavucontrol:"
        echo "   pavucontrol"
        echo ""
        echo "3. In pavucontrol:"
        echo "   - Start the Audio Assistant app and click 'Start Listening'"
        echo "   - Go to the 'Recording' tab"
        echo "   - Find the Audio Assistant entry"
        echo "   - Change the device to 'Monitor of [your output device]'"
        echo ""
        echo "Alternative: Use PipeWire-specific tools:"
        echo "   - helvum (GUI): sudo apt install helvum"
        echo "   - qpwgraph (GUI): sudo apt install qpwgraph"
        echo "   - pw-cli (CLI): pw-cli ls Node"
        ;;

    *)
        echo "⚠ Could not detect audio system (PulseAudio or PipeWire)"
        echo ""
        echo "Please ensure one of the following is installed and running:"
        echo "  - PulseAudio: sudo apt install pulseaudio"
        echo "  - PipeWire: sudo apt install pipewire pipewire-pulse"
        echo ""
        echo "Then run this script again."
        exit 1
        ;;
esac

echo ""
echo "================================"
echo "Additional Information"
echo "================================"
echo ""
echo "Available audio devices:"
echo ""

if command -v pactl &> /dev/null; then
    echo "--- Input Sources ---"
    pactl list sources short | nl
    echo ""
    echo "--- Output Sinks ---"
    pactl list sinks short | nl
fi

echo ""
echo "To test your audio setup:"
echo "1. Start the Audio Assistant application"
echo "2. Click 'Start Listening'"
echo "3. Play some audio (YouTube, music, etc.)"
echo "4. Check if transcriptions appear in the app"
echo ""
echo "If you still hear audio in your speakers/headphones while recording,"
echo "that's normal! The loopback allows both listening and recording."
echo ""
echo "Setup complete! 🎤"
