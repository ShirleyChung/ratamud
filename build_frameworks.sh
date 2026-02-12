#!/bin/bash

set -e

PROJECT_NAME="ratamud"
FRAMEWORK_NAME="RataMUD"

echo "🔨 Building macOS and iOS Frameworks..."

# 创建输出目录
mkdir -p frameworks/macos
mkdir -p frameworks/ios
mkdir -p frameworks/ios-simulator

# 1. 构建 macOS Framework (当前架构)
echo "📦 Building macOS Framework..."

# 检测当前架构
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    echo "Building for Apple Silicon (ARM64)..."
    cargo build --release --target aarch64-apple-darwin --lib
    cp target/aarch64-apple-darwin/release/lib${PROJECT_NAME}.a frameworks/macos/lib${PROJECT_NAME}.a
else
    echo "Building for Intel (x86_64)..."
    cargo build --release --target x86_64-apple-darwin --lib
    cp target/x86_64-apple-darwin/release/lib${PROJECT_NAME}.a frameworks/macos/lib${PROJECT_NAME}.a
fi

# 创建 macOS Framework 结构
MACOS_FRAMEWORK="frameworks/${FRAMEWORK_NAME}.framework"
rm -rf "$MACOS_FRAMEWORK"
mkdir -p "$MACOS_FRAMEWORK/Versions/A/Headers"
mkdir -p "$MACOS_FRAMEWORK/Versions/A/Resources"

# 复制静态库
cp frameworks/macos/lib${PROJECT_NAME}.a "$MACOS_FRAMEWORK/Versions/A/${FRAMEWORK_NAME}"

# 复制头文件
cp src/ratamud.h "$MACOS_FRAMEWORK/Versions/A/Headers/"

# 创建 Info.plist
cat > "$MACOS_FRAMEWORK/Versions/A/Resources/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.ratamud.framework</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>
EOF

# 创建符号链接
cd "$MACOS_FRAMEWORK"
ln -sf Versions/A/Headers Headers
ln -sf Versions/A/Resources Resources
ln -sf Versions/A/${FRAMEWORK_NAME} ${FRAMEWORK_NAME}
ln -sf A Versions/Current
cd - > /dev/null

echo "✅ macOS Framework created at: $MACOS_FRAMEWORK"

# 2. 构建 iOS Framework (ARM64 真机)
echo "📱 Building iOS Framework..."

# 安装 iOS targets (如果尚未安装)
rustup target add aarch64-apple-ios 2>/dev/null || true
rustup target add aarch64-apple-ios-sim 2>/dev/null || true
rustup target add x86_64-apple-ios 2>/dev/null || true

# iOS 真机 (ARM64) - 无 UI 模式
cargo build --release --target aarch64-apple-ios --lib --no-default-features

# iOS 模拟器 (Apple Silicon) - 无 UI 模式
cargo build --release --target aarch64-apple-ios-sim --lib --no-default-features

# iOS 模拟器 (Intel) - 无 UI 模式
cargo build --release --target x86_64-apple-ios --lib --no-default-features

# 创建模拟器 Universal Binary
lipo -create \
    target/aarch64-apple-ios-sim/release/lib${PROJECT_NAME}.a \
    target/x86_64-apple-ios/release/lib${PROJECT_NAME}.a \
    -output frameworks/ios-simulator/lib${PROJECT_NAME}.a

# 创建 iOS Framework 结构 (真机)
IOS_FRAMEWORK="frameworks/${FRAMEWORK_NAME}-iOS.framework"
rm -rf "$IOS_FRAMEWORK"
mkdir -p "$IOS_FRAMEWORK/Headers"

cp target/aarch64-apple-ios/release/lib${PROJECT_NAME}.a "$IOS_FRAMEWORK/${FRAMEWORK_NAME}"
cp src/ratamud.h "$IOS_FRAMEWORK/Headers/"

cat > "$IOS_FRAMEWORK/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.ratamud.framework.ios</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>MinimumOSVersion</key>
    <string>13.0</string>
</dict>
</plist>
EOF

echo "✅ iOS Framework created at: $IOS_FRAMEWORK"

# 创建 iOS 模拟器 Framework
IOS_SIM_FRAMEWORK="frameworks/${FRAMEWORK_NAME}-iOS-Simulator.framework"
rm -rf "$IOS_SIM_FRAMEWORK"
mkdir -p "$IOS_SIM_FRAMEWORK/Headers"

cp frameworks/ios-simulator/lib${PROJECT_NAME}.a "$IOS_SIM_FRAMEWORK/${FRAMEWORK_NAME}"
cp src/ratamud.h "$IOS_SIM_FRAMEWORK/Headers/"

cat > "$IOS_SIM_FRAMEWORK/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.ratamud.framework.ios-simulator</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${FRAMEWORK_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>MinimumOSVersion</key>
    <string>13.0</string>
</dict>
</plist>
EOF

echo "✅ iOS Simulator Framework created at: $IOS_SIM_FRAMEWORK"

# 3. 创建 XCFramework (可选，同时支持真机和模拟器)
echo "📦 Creating XCFramework..."

xcodebuild -create-xcframework \
    -framework "$IOS_FRAMEWORK" \
    -framework "$IOS_SIM_FRAMEWORK" \
    -output "frameworks/${FRAMEWORK_NAME}.xcframework"

echo "✅ XCFramework created at: frameworks/${FRAMEWORK_NAME}.xcframework"

echo ""
echo "🎉 All frameworks built successfully!"
echo ""
echo "📍 Outputs:"
echo "  - macOS: frameworks/${FRAMEWORK_NAME}.framework"
echo "  - iOS (Device): frameworks/${FRAMEWORK_NAME}-iOS.framework"
echo "  - iOS (Simulator): frameworks/${FRAMEWORK_NAME}-iOS-Simulator.framework"
echo "  - XCFramework: frameworks/${FRAMEWORK_NAME}.xcframework"
