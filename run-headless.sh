#!/usr/bin/env bash
# Launch claude_crt with GPU-accelerated headless rendering via gamescope
#
# PURPOSE:
#   Enable visual debugging and iteration without a physical display.
#   Uses gamescope to provide a virtual Wayland compositor with full GPU acceleration.
#
# FEATURES:
#   - Real GPU rendering (same code path as windowed mode)
#   - BRP server on http://localhost:15702
#   - Screenshot support via BRP (works in headless!)
#   - Automatic GPU detection and Vulkan ICD configuration
#
# REQUIREMENTS:
#   - gamescope installed (pacman -S gamescope on Arch)
#   - GPU with Vulkan support
#   - User in 'video' group for DRM access
#
# USAGE:
#   ./run-headless.sh
#
# For full development workflow, see AGENT_GUIDE.md

set -euo pipefail

# Settings
WESTON_LOG="/tmp/weston-headless.log"
WESTON_SOCKET="wayland-1" # Use a non-default socket to avoid conflicts

# Detect GPU and set appropriate ICD
detect_vulkan_icd() {
    local icd_dir="/usr/share/vulkan/icd.d"

    # Check environment variable first
    if [ -n "${VK_ICD_FILENAMES:-}" ]; then
        echo "$VK_ICD_FILENAMES"
        return
    fi

    if [ -f "$icd_dir/radeon_icd.x86_64.json" ]; then
        echo "$icd_dir/radeon_icd.x86_64.json"
    elif [ -f "$icd_dir/nvidia_icd.json" ]; then
        echo "$icd_dir/nvidia_icd.json"
    elif [ -f "$icd_dir/intel_icd.x86_64.json" ]; then
        echo "$icd_dir/intel_icd.x86_64.json"
    else
        echo ""
    fi
}

# Check if we already have a display
has_display() {
    [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]
}

# Start gamescope for headless GPU rendering
start_gamescope() {
    if ! command -v gamescope &> /dev/null; then
        echo "❌ gamescope not found. Please install gamescope." >&2
        exit 1
    fi

    echo "🎮 Starting gamescope (GPU-accelerated virtual display)..." >&2
    echo "   Logs: $WESTON_LOG" >&2

    # Set up XDG_RUNTIME_DIR
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    mkdir -p "$XDG_RUNTIME_DIR"

    # Gamescope will create a Wayland display for our app to render to
    # No cleanup trap needed - gamescope will be parent of our app
}

# Main
main() {
    local icd_file

    icd_file=$(detect_vulkan_icd)

    echo "🚀 Starting claude_crt with GPU rendering..."

    if [ -n "$icd_file" ]; then
        echo "📍 Using Vulkan ICD: $icd_file"
        export VK_ICD_FILENAMES="$icd_file"
    fi

    export WGPU_BACKEND=vulkan

    echo "🎮 BRP server: http://localhost:15702"
    echo "🔧 Press Ctrl+C to stop"
    echo ""

    # Run with gamescope if no display available
    if ! has_display; then
        echo "🔍 No display detected, using gamescope for GPU-accelerated rendering..."
        start_gamescope

        # Run app inside gamescope
        # Gamescope creates a virtual display and renders headlessly with GPU
        exec gamescope -w 1920 -h 1080 -W 1920 -H 1080 --backend headless -- \
            cargo run -p claude_crt --quiet -- --fullscreen
    else
        echo "✅ Display available, using existing display"
        exec cargo run -p claude_crt --quiet -- --fullscreen
    fi
}

main "$@"
