# RataMUD

RataMUD 是一個以 Rust 為主體的文字冒險 / MUD 遊戲引擎。專案的主要程式碼在 `src/`，遊戲資料在 `worlds/`。C/C++、iOS framework、SCons、shell script 主要是外部介面、建置包裝或測試輔助，不是遊戲邏輯的核心。

## 專案定位

本專案可以分成三層：

1. **Rust 遊戲核心**
   - 負責世界狀態、地圖、角色、NPC、物品、任務、事件、交易、戰鬥、時間推進與指令處理。
   - 這是專案的主要程式。

2. **使用者介面 / Client**
   - Terminal UI 使用 Ratatui + Crossterm。
   - C ABI / FFI 讓 C/C++、iOS 或其他平台可以呼叫 Rust core。
   - 這些 client 不應該放遊戲規則，只應該負責輸入、輸出、畫面呈現與平台整合。

3. **資料與輔助資源**
   - `worlds/` 放遊戲世界資料。
   - `Docs/` 放設計、開發記錄與功能文件。
   - `dist/`、`frameworks/` 放建置輸出或發佈用檔案。
   - `scripts/`、`testscripts/` 多數是 Copilot 生成的測試腳本或建置輔助，只是輔助，不是必要程式。

## 主要目錄

```text
.
├── src/                  # Rust 主程式與遊戲核心
├── worlds/               # 遊戲世界資料，JSON 格式
├── Docs/                 # 架構、功能、bugfix、開發記錄
├── Docs/Archive/         # 從根目錄整理進來的歷史 markdown
├── scripts/              # 建置、測試、resume Codex 等輔助腳本
├── dist/                 # 發佈輸出與範例 client 產物
├── frameworks/           # iOS/macOS framework 產物
├── testscripts/          # 舊的輔助測試腳本，不是核心程式
├── Cargo.toml            # Rust crate 設定
├── SConstruct            # SCons 建置包裝，主要給 native/client 發佈用
└── README.md             # 專案入口說明
```

## Rust Core

核心模組都在 `src/`：

| 檔案 | 責任 |
|------|------|
| `main.rs` | Terminal 版本入口 |
| `lib.rs` | Library 版本匯出 |
| `app.rs` | Terminal UI 主迴圈、事件處理、畫面更新 |
| `world.rs` | GameWorld，管理地圖、時間、NPC、任務、事件、互動與戰鬥狀態 |
| `map.rs` | 100x100 地圖、地形、格點、物品掉落 |
| `person.rs` | 玩家 / NPC 資料、屬性、對話、關係、物品、時間更新 |
| `npc_manager.rs` | NPC 載入、查找、儲存、位置查詢 |
| `npc_ai.rs`、`npc_view.rs`、`npc_action.rs` | NPC AI 快照與行動決策 |
| `command_handler.rs` | 字串指令解析成 `CommandResult` |
| `command_executor.rs` | 無 UI / FFI 模式可用的指令執行 |
| `input.rs` | Terminal 鍵盤輸入與命令歷史 |
| `output.rs` | Terminal 輸出管理 |
| `core_output.rs` | 無 UI / FFI 模式輸出 callback |
| `ffi.rs` | C ABI 介面 |
| `event*.rs` | 事件載入、排程、執行 |
| `quest.rs` | 任務系統 |
| `trade.rs` | 交易系統 |
| `item.rs`、`item_registry.rs` | 物品模型與物品資料庫 |
| `time_thread.rs`、`time_updatable.rs` | 時間推進與可更新物件 |
| `ui.rs` | Ratatui 畫面元件 |

## Client / Interface

### Terminal UI

預設執行方式是 terminal 版本：

```bash
cargo run
```

或：

```bash
cargo run --bin main
```

Terminal UI 是目前最完整的 client，包含互動選單、地圖顯示、戰鬥、交易、任務、log、狀態欄等畫面邏輯。

### C ABI / Library Mode

`Cargo.toml` 會把 crate 編成：

```toml
crate-type = ["staticlib", "cdylib", "rlib"]
```

FFI 相關檔案：

- `src/ffi.rs`
- `src/ratamud.h`
- `test_callback.cpp`
- `Docs/Archive/LIB_MODE_GUIDE.md`
- `Docs/Archive/OUTPUT_CALLBACK_USAGE.md`
- `Docs/C_ABI_README.md`
- `Docs/C_ABI_GUIDE.md`

Library 模式用來讓 C/C++、iOS 或其他平台接入 Rust core。這些 client 應透過 C ABI 傳入指令、接收 callback 輸出，不應複製遊戲規則。

### iOS / macOS Framework

`frameworks/` 裡的檔案是建置產物或發佈包，目標是讓 iOS/macOS client 使用 Rust core。它們不是遊戲邏輯來源。

相關文件：

- `Docs/Archive/IOS_FRAMEWORK_SUCCESS.md`
- `Docs/Archive/SCONS_IOS_FRAMEWORKS.md`
- `Docs/IOS_FRAMEWORK_README.md`
- `scripts/build_frameworks.sh`
- `scripts/test_ios_build.sh`

## 遊戲資料

主要資料在：

```text
worlds/
├── settings.json
├── person_descriptions.json
└── beginWorld/
    ├── world.json
    ├── time.json
    ├── maps/
    ├── persons/
    ├── quests/
    └── events/
```

`worlds/beginWorld/` 是目前預設世界。程式會讀寫其中的 `time.json`、`maps/*.json`、`persons/*.json` 等檔案，所以執行遊戲後資料可能會改變。

## 建置與執行

### 檢查 Rust core

```bash
cargo check --lib
```

### 執行單元測試

```bash
cargo test --lib
```

目前專案內有少量 Rust 單元測試。部分 shell script 是 Copilot 生成的整合測試草稿，不代表必要測試流程。

### 執行遊戲

```bash
cargo run
```

### 建置 library

```bash
cargo build --release --lib
```

若要同時編譯 C++ callback 範例，可使用：

```bash
scripts/build_and_test.sh
```

這是 client / FFI 驗證，不是 Rust core 的必要建置步驟。

## 常用指令

遊戲內支援的指令由 `src/command_handler.rs` 定義。常見指令包括：

| 指令 | 說明 |
|------|------|
| `help` | 顯示可用指令 |
| `look` / `l` | 查看目前位置或 NPC |
| `up` / `down` / `left` / `right` | 移動 |
| `get` / `drop` / `use` | 物品操作 |
| `npcs` | 列出 NPC |
| `talk` | 與 NPC 對話 |
| `trade` / `buy` / `sell` | 交易 |
| `quest ...` | 任務操作 |
| `punch` / `kick` / `escape` | 戰鬥 |
| `show map` / `show minimap` / `show log` | UI 顯示控制 |
| `exit` / `quit` | 離開 |

完整內容以 `CommandResult::get_help_info()` 和 `parse_command()` 為準。

## 文件整理原則

`Docs/` 中有很多文件是功能開發、重構、bugfix 或 Copilot 協作過程留下的紀錄。閱讀時建議優先看：

- `Docs/README.md`：文件索引
- `Docs/Development/CODE_RULES.md`：開發規範
- `Docs/EVENT_DRIVEN_RULE.md`：事件驅動架構原則
- `Docs/C_ABI_README.md`：C ABI 使用說明
- `Docs/README_DIALOGUE.md`、`Docs/SDL_SYNTAX.md`：對話系統

其他 `*_SUMMARY.md`、`*_FIX.md`、`*_COMPLETE.md` 多半是歷史記錄，不一定代表目前最佳入口。

## 目前整理重點

- Rust code 是主要程式。
- C/C++ 與 iOS 是 client/interface。
- `dist/`、`frameworks/` 是建置或發佈產物。
- `scripts/` 和 `testscripts/` 裡的 shell 測試腳本只是輔助，不是核心架構。
- 真正需要維護的主線是 `src/`、`worlds/`、`Cargo.toml` 和必要文件。
