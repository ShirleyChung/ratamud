# RataMUD Framework 範例說明

## 概述

`example.c` 展示了如何在無 UI 模式下使用 RataMUD 遊戲核心，通過回調函數用 `printf` 直接印出遊戲輸出，不依賴 ratatui 或 crossterm。

**✅ 支援平台**：
- macOS (ARM64/Intel)
- iOS (真機 ARM64)
- iOS Simulator (ARM64/Intel)

## 編譯和執行

### 方法一：使用 SCons (推薦，最簡單)

```bash
# 一鍵編譯運行
scons run-framework

# 或分步執行
scons framework           # 建立 macOS Framework
scons example-framework   # 編譯範例
scons run-framework       # 運行
```

### 方法二：使用 build_frameworks.sh (建立所有平台)

```bash
# 1. 建立所有 Frameworks (macOS + iOS + iOS Simulator)
./build_frameworks.sh

# 2. 編譯 macOS 範例
cd dist
make example-framework

# 3. 執行
make run-framework
```

### 方法三：使用動態庫 (dylib)

```bash
# 1. 建立動態庫
cargo build --release --lib

# 2. 複製到 dist 目錄
cp target/release/libratamud.dylib dist/

# 3. 編譯並執行
cd dist
make example
make run-c
```

## 輸出範例

執行後會看到以下輸出：

```
╔══════════════════════════════════════╗
║   RataMUD 遊戲核心範例 (無 UI 模式)  ║
╚══════════════════════════════════════╝

🔧 註冊輸出回調函數...
✅ 回調已註冊

🎮 啟動遊戲核心測試...
─────────────────────────────────────

💬 歡迎來到 RataMUD！
💬 你站在一個廣場中央。
📝 遊戲初始化完成
📝 載入地圖: town_square
⚡ 遊戲時間: Day 1 09:00
ℹ️  NPC: 商人
   等級: 10
   生命: 100/100
💬 一隻野豬向你衝來！

─────────────────────────────────────
📊 測試完成！共收到 7 條訊息
```

## 訊息類型

範例使用四種訊息類型，每種有不同的顏色和圖示：

| 類型 | 圖示 | 顏色 | 用途 |
|------|------|------|------|
| MAIN | 💬 | 亮綠色 | 主遊戲訊息（移動、戰鬥、對話） |
| LOG | 📝 | 青色 | 系統日誌（帶時間戳） |
| STATUS | ⚡ | 亮黃色 | 狀態欄訊息（臨時訊息） |
| SIDE | ℹ️ | 紫色 | 側邊面板（NPC 資訊等） |

## 核心 API 說明

### 回調函數類型

```c
typedef void (*OutputCallback)(const char* msg_type, const char* content);
```

### 關鍵函數

1. **註冊回調**
   ```c
   void ratamud_register_output_callback(OutputCallback callback);
   ```
   註冊一個回調函數來接收遊戲輸出。

2. **清除回調**
   ```c
   void ratamud_clear_output_callback(void);
   ```
   清除已註冊的回調函數。

3. **測試輸出**
   ```c
   void ratamud_test_output_callback(void);
   ```
   觸發測試輸出，用於驗證回調系統是否正常工作。

## 自定義範例

你可以修改 `example.c` 中的 `game_output_callback` 函數來自定義輸出格式：

```c
void game_output_callback(const char* msg_type, const char* content) {
    // 自定義處理邏輯
    if (strcmp(msg_type, "MAIN") == 0) {
        // 處理主遊戲訊息
    } else if (strcmp(msg_type, "LOG") == 0) {
        // 處理日誌訊息
    }
    // ...
}
```

## Framework 結構

### macOS Framework
macOS Framework 位於 `frameworks/RataMUD.framework/`：

```
RataMUD.framework/
├── RataMUD              # 靜態庫 (ARM64)
├── Headers/
│   └── ratamud.h        # C 頭文件
├── Resources/
│   └── Info.plist       # Framework 元數據
└── Versions/
    └── A/               # 版本 A
```

### iOS Frameworks

**iOS 真機** (`frameworks/RataMUD-iOS.framework/`):
```
RataMUD-iOS.framework/
├── RataMUD              # 靜態庫 (ARM64)
├── Headers/
│   └── ratamud.h
└── Info.plist
```

**iOS 模擬器** (`frameworks/RataMUD-iOS-Simulator.framework/`):
```
RataMUD-iOS-Simulator.framework/
├── RataMUD              # Universal Binary (ARM64 + x86_64)
├── Headers/
│   └── ratamud.h
└── Info.plist
```

### Framework 大小

| Framework | 大小 | 架構 |
|-----------|------|------|
| macOS | 21 MB | ARM64 |
| iOS | 18 MB | ARM64 |
| iOS Simulator | 35 MB | ARM64 + x86_64 |

## 注意事項

1. **CoreFoundation 依賴**：macOS Framework 版本需要鏈接 CoreFoundation：
   ```bash
   gcc example.c -framework RataMUD -framework CoreFoundation
   ```

2. **rpath 設定**：需要設定正確的 rpath 以找到 Framework：
   ```bash
   -Wl,-rpath,../frameworks
   ```

3. **無 UI 模式**：這個範例使用的是無 UI 的核心功能，不包含 ratatui/crossterm。

4. **iOS 支援**：iOS Frameworks 已成功建立，可用於整合到 iOS 應用中。使用 `--no-default-features` 編譯以排除終端 UI 依賴。

5. **Rust 工具鏈**：需要使用 rustup 管理的 Rust（不要用 Homebrew 安裝的 Rust），才能編譯 iOS targets。詳見 `RUST_ENV_FIX.md`。

## 疑難排解

### 找不到 Framework
```
ld: framework not found RataMUD
```
**解決方案**：先執行 `./build_frameworks.sh` 建立 Framework。

### 缺少符號
```
Undefined symbols: "_CFRelease"
```
**解決方案**：加入 `-framework CoreFoundation` 到鏈接選項。

### 執行時找不到 Framework
```
dyld: Library not loaded
```
**解決方案**：確保 rpath 設定正確，或從專案根目錄執行 `dist/example`。
