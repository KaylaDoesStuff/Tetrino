#!/bin/bash
set -e

echo "=== Tetrino Windows Build ==="
echo ""

# Ensure target is installed
rustup target add x86_64-pc-windows-msvc 2>/dev/null

echo "Building for Windows (x86_64-pc-windows-msvc)..."
cargo xwin build --release --target x86_64-pc-windows-msvc

EXE_PATH="target/x86_64-pc-windows-msvc/release/tetrino.exe"

if [ -f "$EXE_PATH" ]; then
    SIZE=$(du -h "$EXE_PATH" | cut -f1)
    echo ""
    echo "Build successful!"
    echo "Output: $EXE_PATH ($SIZE)"
    echo ""
    echo "To distribute: copy tetrino.exe + settings.toml to the target Windows machine."
    echo "The target machine needs Visual C++ Redistributable installed."
else
    echo ""
    echo "Build failed - exe not found at $EXE_PATH"
    exit 1
fi
