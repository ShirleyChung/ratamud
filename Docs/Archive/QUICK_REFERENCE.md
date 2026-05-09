# 🎯 RataMUD 跨平台 Framework 快速參考

## ✅ 當前狀態

**Rust 工具鏈**: rustup 1.92.0 (已從 Homebrew 遷移)  
**支援平台**: macOS, iOS, iOS Simulator  
**Framework 狀態**: 全部成功建立

## 📦 一鍵建立所有 Frameworks

```bash
./build_frameworks.sh
```

## 🚀 快速測試 macOS 範例

```bash
scons run-framework
```

## 📊 Framework 資訊

```
frameworks/
├── RataMUD.framework (21 MB)           # macOS ARM64
├── RataMUD-iOS.framework (18 MB)       # iOS ARM64
└── RataMUD-iOS-Simulator.framework     # iOS Sim (ARM64+x86_64)
    (35 MB)
```

## 🔧 關鍵命令

### 編譯
```bash
# macOS
cargo build --release --lib

# iOS (無 UI)
cargo build --release --target aarch64-apple-ios --lib --no-default-features

# iOS Simulator
cargo build --release --target aarch64-apple-ios-sim --lib --no-default-features
```

### SCons
```bash
scons framework          # 建立 macOS Framework
scons example-framework  # 編譯範例
scons run-framework      # 運行
scons -c                 # 清理
```

## 📖 詳細文檔

| 文件 | 內容 |
|------|------|
| `IOS_FRAMEWORK_SUCCESS.md` | iOS Framework 完整說明 |
| `RUST_ENV_FIX.md` | Rust 環境修復記錄 |
| `dist/FRAMEWORK_EXAMPLE.md` | Framework 使用範例 |
| `dist/SCONS_BUILD.md` | SCons 建構系統 |
| `dist/QUICKSTART.md` | 快速開始指南 |

## 💡 重要提醒

- ✅ 使用 rustup 管理 Rust（不要用 Homebrew）
- ✅ iOS 編譯需要 `--no-default-features`
- ✅ macOS Framework 需要鏈接 CoreFoundation
- ✅ 所有輸出用 printf，無需 TUI

## 🎮 範例輸出

執行 `scons run-framework` 會看到：

```
💬 歡迎來到 RataMUD！
📝 遊戲初始化完成
⚡ 遊戲時間: Day 1 09:00
ℹ️  NPC: 商人
```

使用彩色 emoji，直接 printf 輸出！
