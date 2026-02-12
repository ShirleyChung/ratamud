#!/bin/bash

set -e

echo "🧪 Testing iOS build compatibility..."

# 安装 iOS targets
rustup target add aarch64-apple-ios 2>/dev/null || true

# 尝试构建 iOS 库
echo "Testing iOS ARM64 build..."
cargo build --target aarch64-apple-ios --lib 2>&1 | tee /tmp/ios_build.log

if [ $? -eq 0 ]; then
    echo "✅ iOS build successful!"
else
    echo "❌ iOS build failed. Check /tmp/ios_build.log for details"
    echo ""
    echo "💡 Possible solutions:"
    echo "1. Some dependencies (crossterm, ratatui) may not support iOS"
    echo "2. Consider using conditional compilation in Cargo.toml"
    echo "3. Or create a separate iOS-specific crate"
fi
