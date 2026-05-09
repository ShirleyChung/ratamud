# Rust 環境修復記錄

## 問題
之前使用 Homebrew 安裝的 Rust 無法編譯 iOS targets，因為 Homebrew 版本的 Rust 缺少 iOS 標準庫。

## 解決方案
移除 Homebrew 的 Rust，改用 rustup 管理的 Rust。

## 執行的操作

```bash
# 1. 移除 Homebrew Rust
brew uninstall rust

# 2. 驗證現在使用 rustup 版本
which rustc
# 輸出: /Users/shirleychung/.cargo/bin/rustc

rustc --version
# 輸出: rustc 1.92.0 (ded5c06cf 2025-12-08)
```

## 當前配置

### Rust 工具鏈
- **rustc**: 1.92.0 (rustup 管理)
- **cargo**: 1.92.0 (rustup 管理)
- **位置**: `~/.cargo/bin/`

### 已安裝的 targets
```bash
rustup target list --installed
```
- aarch64-apple-darwin (macOS ARM64)
- aarch64-apple-ios (iOS 真機)
- aarch64-apple-ios-sim (iOS 模擬器 ARM64)
- x86_64-apple-ios (iOS 模擬器 Intel)

## 驗證

### macOS 編譯
```bash
cargo build --release --lib
# ✅ 成功
```

### iOS 編譯
```bash
cargo build --release --target aarch64-apple-ios --lib --no-default-features
# ✅ 成功（之前失敗）
```

### Framework 建立
```bash
./build_frameworks.sh
# ✅ macOS Framework 成功
# ✅ iOS Framework 成功
# ✅ iOS Simulator Framework 成功
```

## 優勢

使用 rustup 而非 Homebrew 管理 Rust 的優勢：

1. **跨平台 targets**: 支援 iOS、Android 等多平台
2. **版本管理**: 可輕鬆切換 Rust 版本
3. **工具鏈管理**: 統一管理所有 Rust 工具
4. **標準庫完整**: 包含所有平台的標準庫

## 未來更新 Rust

```bash
# 更新 Rust
rustup update

# 查看版本
rustup show

# 添加新 target
rustup target add <target-name>
```

## 注意事項

- 不要再用 `brew install rust`
- 使用 `rustup` 管理所有 Rust 相關工具
- PATH 會自動使用 `~/.cargo/bin` 中的工具
