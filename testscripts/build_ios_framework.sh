#!/bin/bash
set -e

echo "Building iOS Framework for ratamud..."

# 確保使用 rustup 的 toolchain
export PATH="$HOME/.cargo/bin:$PATH"

# 檢查 Xcode 設置
XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "")
if [[ "$XCODE_PATH" == *"CommandLineTools"* ]]; then
    echo "⚠️  檢測到使用 Command Line Tools，但編譯 iOS framework 需要完整的 Xcode"
    echo ""
    echo "請執行以下命令切換到 Xcode："
    echo "  sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer"
    echo ""
    echo "然後重新執行此腳本"
    exit 1
fi

if [[ "$XCODE_PATH" != *"Xcode.app"* ]]; then
    echo "❌ 未找到 Xcode，請先安裝 Xcode"
    exit 1
fi

echo "✅ Using Xcode at: $XCODE_PATH"

# 檢查並安裝 iOS targets
echo "Adding iOS targets..."
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# 檢查並安裝 cargo-lipo (用於創建通用庫)
if ! command -v cargo-lipo &> /dev/null; then
    echo "Installing cargo-lipo..."
    cargo install cargo-lipo
fi

# 創建輸出目錄
mkdir -p dist/ios

# 編譯各個架構
echo "Building for iOS device (arm64)..."
cargo build --lib --release --target aarch64-apple-ios

echo "Building for iOS simulator (arm64)..."
cargo build --lib --release --target aarch64-apple-ios-sim

echo "Building for iOS simulator (x86_64)..."
cargo build --lib --release --target x86_64-apple-ios

# 創建 iOS Simulator 的通用二進制文件
echo "Creating universal simulator library..."
mkdir -p target/universal-sim/release
lipo -create \
    target/aarch64-apple-ios-sim/release/libratamud.a \
    target/x86_64-apple-ios/release/libratamud.a \
    -output target/universal-sim/release/libratamud.a

# 創建 XCFramework
echo "Creating XCFramework..."
rm -rf dist/ios/ratamud.xcframework

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libratamud.a \
    -headers src/ratamud.h \
    -library target/universal-sim/release/libratamud.a \
    -headers src/ratamud.h \
    -output dist/ios/ratamud.xcframework

echo "✅ iOS XCFramework created at: dist/ios/ratamud.xcframework"
echo ""
echo "使用方法："
echo "1. 將 dist/ios/ratamud.xcframework 拖入 Xcode 專案"
echo "2. 在 Xcode 中: Target -> General -> Frameworks, Libraries, and Embedded Content"
echo "3. 點擊 + 號，選擇 ratamud.xcframework"
echo "4. 在 Swift/Objective-C 中導入使用"
