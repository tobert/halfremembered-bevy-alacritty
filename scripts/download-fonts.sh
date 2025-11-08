#!/bin/bash
# Download Cascadia Code fonts from Microsoft

set -e

echo "📥 Downloading Cascadia Code fonts..."

FONTS_DIR="bevy-terminal/assets/fonts"
mkdir -p "$FONTS_DIR"

CASCADIA_VERSION="2404.23"
CASCADIA_URL="https://github.com/microsoft/cascadia-code/releases/download/v${CASCADIA_VERSION}/CascadiaCode-${CASCADIA_VERSION}.zip"

wget --quiet -O cascadia-temp.zip "$CASCADIA_URL"
unzip -q cascadia-temp.zip -d cascadia-temp

# Copy Regular (MVP), Bold and Italic (future)
cp cascadia-temp/ttf/static/CascadiaMono-Regular.ttf "$FONTS_DIR/"
cp cascadia-temp/ttf/static/CascadiaMono-Bold.ttf "$FONTS_DIR/"
cp cascadia-temp/ttf/static/CascadiaMono-Italic.ttf "$FONTS_DIR/"

rm -rf cascadia-temp cascadia-temp.zip

echo "✅ Fonts downloaded:"
ls -lh "$FONTS_DIR"
