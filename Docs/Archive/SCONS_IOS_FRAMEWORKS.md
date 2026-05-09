# ✅ SCons 支援 iOS Frameworks 建構

## 概述

現在 SCons 已經完全支援 iOS Framework 的編譯，包括：
- iOS 真機 (ARM64)
- iOS 模擬器 (ARM64 + x86_64)
- XCFramework (整合真機和模擬器)

## 使用方法

### 建構 iOS Frameworks

```bash
# 建構所有 iOS frameworks（真機 + 模擬器 + XCFramework）
scons ios-frameworks

# 或使用別名
scons all-ios
```

### 輸出結果

執行後會產生以下檔案：

```
frameworks/
├── RataMUDiOS.framework/              # iOS 真機 (ARM64)
│   ├── Headers/
│   │   └── ratamud.h
│   ├── Info.plist
│   └── RataMUDiOS                     # 18 MB 靜態庫
│
├── RataMUDiOSSimulator.framework/     # iOS 模擬器 (ARM64 + x86_64)
│   ├── Headers/
│   │   └── ratamud.h
│   ├── Info.plist
│   └── RataMUDiOSSimulator            # 35 MB 靜態庫（universal binary）
│
└── RataMUD.xcframework/               # XCFramework (整合版)
    ├── Info.plist
    ├── ios-arm64/
    │   └── RataMUDiOS.framework/
    └── ios-arm64_x86_64-simulator/
        └── RataMUDiOSSimulator.framework/
```

## 建構過程

SCons 自動執行以下步驟：

1. **安裝 iOS targets**（如果尚未安裝）
   ```bash
   rustup target add aarch64-apple-ios
   rustup target add aarch64-apple-ios-sim
   rustup target add x86_64-apple-ios
   ```

2. **編譯 iOS 真機版本**
   ```bash
   cargo build --release --target aarch64-apple-ios --lib --no-default-features
   ```

3. **編譯 iOS 模擬器版本**（兩個架構）
   ```bash
   cargo build --release --target aarch64-apple-ios-sim --lib --no-default-features
   cargo build --release --target x86_64-apple-ios --lib --no-default-features
   ```

4. **建立模擬器 Universal Binary**
   ```bash
   lipo -create \
       target/aarch64-apple-ios-sim/release/libratamud.a \
       target/x86_64-apple-ios/release/libratamud.a \
       -output frameworks/ios-simulator/libratamud.a
   ```

5. **建立 Framework 結構**
   - 複製靜態庫到 framework 中
   - 複製標頭檔案
   - 建立 Info.plist

6. **建立 XCFramework**
   ```bash
   xcodebuild -create-xcframework \
       -framework frameworks/RataMUDiOS.framework \
       -framework frameworks/RataMUDiOSSimulator.framework \
       -output frameworks/RataMUD.xcframework
   ```

## 關鍵技術要點

### 1. 為什麼不支援 ratatui 和 crossterm？

**crossterm** 和 **ratatui** 都依賴終端 API，而這些 API 在 iOS 上不存在：

- **crossterm** 依賴：
  - Unix: `libc`, `termios`, `ioctl`
  - Windows: Windows Console API
  - ❌ iOS: 沒有標準終端設備

- **ratatui** 依賴：
  - 需要 crossterm 或其他終端後端
  - ❌ iOS: 沒有終端後端可用

### 2. 解決方案：Features 系統

在 `Cargo.toml` 中：

```toml
[features]
default = ["terminal-ui"]
terminal-ui = ["ratatui", "crossterm"]

[dependencies]
# Terminal UI dependencies (optional)
ratatui = { version = "0.26", optional = true }
crossterm = { version = "0.27", optional = true }
```

iOS 編譯時使用 `--no-default-features`，這樣就不會包含 ratatui 和 crossterm。

### 3. Framework 命名規則

為了與 xcodebuild 兼容：
- Framework 名稱：`RataMUDiOS.framework`
- 二進制名稱：`RataMUDiOS`（必須與 framework 名稱一致）
- Info.plist 中的 `CFBundleExecutable` 也必須匹配

## SCons 整合

### 新增的目標

在 `dist/SConscript` 中新增了 `build_ios_frameworks` 函數：

```python
def build_ios_frameworks(target, source, env):
    # 1. 確保 iOS targets 已安裝
    # 2. 編譯 iOS 真機和模擬器版本
    # 3. 建立 universal binary
    # 4. 建立 framework 結構
    # 5. 建立 XCFramework
    ...

ios_frameworks = local_env.Command(
    ios_fw_marker,
    rust_sources + [cargo_toml],
    build_ios_frameworks
)

local_env.Alias('ios-frameworks', ios_frameworks)
```

### 別名設定

在 `SConstruct` 中：

```python
if env.get('LIB_EXT') == 'dylib':
    Alias('ios-frameworks', 'frameworks/RataMUD.xcframework/Info.plist')
    Alias('all-ios', 'ios-frameworks')
```

## 使用範例

### 在 iOS 專案中使用

1. **將 XCFramework 拖入 Xcode 專案**
   - 選擇 `frameworks/RataMUD.xcframework`
   - Xcode 會自動選擇正確的版本（真機或模擬器）

2. **引入標頭檔案**
   ```objective-c
   #import <RataMUD/ratamud.h>
   ```

3. **註冊回調並使用**
   ```objective-c
   // 註冊輸出回調
   ratamud_register_output_callback(my_callback_function);
   
   // 發送指令
   ratamud_input_command("look");
   ```

## 與 build_frameworks.sh 的比較

| 功能 | SCons | build_frameworks.sh |
|-----|-------|-------------------|
| 建構 macOS Framework | ✅ | ✅ |
| 建構 iOS Frameworks | ✅ | ✅ |
| 建構 XCFramework | ✅ | ✅ |
| 自動依賴追蹤 | ✅ | ❌ |
| 增量建構 | ✅ | ❌ |
| 整合 C/C++ 範例 | ✅ | ❌ |
| 一鍵運行測試 | ✅ | ❌ |

## 完整工作流程

```bash
# 1. 建構所有（dylib + 範例）
scons

# 2. 建構 macOS Framework
scons framework

# 3. 建構 iOS Frameworks
scons ios-frameworks

# 4. 運行測試
scons run-c              # 使用 dylib
scons run-framework      # 使用 macOS Framework

# 5. 清理
scons -c
```

## 總結

✅ SCons 已完全支援 iOS Framework 建構  
✅ 自動處理所有平台和架構  
✅ 建立 XCFramework 用於 iOS 整合  
✅ ratatui 和 crossterm 確認不支援 iOS（已透過 features 排除）  
✅ 核心遊戲邏輯可成功編譯為 iOS framework  
✅ 適合整合到任何 iOS/Swift 應用程式中  

所有建構工具（SCons 和 build_frameworks.sh）都已就緒！
