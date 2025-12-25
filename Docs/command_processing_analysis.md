# 命令處理系統架構分析

## 執行日期
2025-12-25

## 問題描述

目前專案中存在三個類似的命令處理函數：
1. `CommandProcessor::parse_command()` (command_processor.rs)
2. `InputHandler::handle_command()` (input.rs)
3. `GameEngine::process_command()` (game_engine.rs)

這些函數看起來在做類似的事情，但實際使用情況和功能範圍不同。

## 詳細分析

### 1. CommandProcessor::parse_command()

**檔案位置**: `src/command_processor.rs`

**功能**:
- 純文本命令解析器
- 不依賴 Crossterm 或終端 UI
- 支援基本命令（約 30 個）

**使用者**:
- ✅ `GameEngine::process_command()` (game_engine.rs)
- ✅ FFI 介面 (ffi.rs)
- ❌ 主程式 (app.rs) **不使用**

**支援的命令**:
```
基本命令：
- exit, quit, help
- 移動：up, down, left, right, move
- 查看：look, status
- 介面：map, minimap, hidemap, log, world, clear
- 物品：get, drop, eat
- 睡眠：sleep, dream, wakeup
- NPC：summon, name, create, destroy, set
- NPC 互動：npcs, ctrl, trade, buy, sell
- 對話：setdialogue, seteagerness
- 其他：typewriter
```

**缺少的命令**（與 input.rs 相比）:
```
❌ give <npc> <item> [qty]      - 給予物品給 NPC
❌ re / repeat                   - 重複上一次命令
❌ talk <npc> [topic]            - 與 NPC 對話
❌ check <npc>                   - 查看 NPC 詳細資訊
❌ sdl / setdialogue (進階版)    - 設置帶條件的對話
❌ setrelationship               - 設置 NPC 好感度
❌ changerelationship            - 改變 NPC 好感度
❌ quest 相關命令                - 任務系統（7+ 個命令）
❌ 以及其他最新新增的命令
```

**特點**:
- 簡化版實作
- 功能相對完整但不是最新
- 程式碼較乾淨
- 標記為 `#[allow(dead_code)]`

---

### 2. InputHandler::handle_command()

**檔案位置**: `src/input.rs`

**功能**:
- 完整的命令解析器
- 主程式的實際命令處理器
- 支援所有最新命令（40+ 個）

**使用者**:
- ✅ 主程式 (app.rs) **主要使用**
- ✅ 透過 `handle_input_events()` 間接調用

**執行流程**:
```
使用者輸入
  ↓
InputHandler::handle_input_events()
  ↓
InputHandler::parse_input()
  ↓
InputHandler::handle_command() [此函數]
  ↓
CommandResult
  ↓
app.rs::handle_command_result()
  ↓
執行對應的 handle_xxx() 函數
```

**支援的命令**:
```
包含 CommandProcessor 的所有命令，外加：

新增命令：
✅ give <npc> <item> [qty]       - 給予物品
✅ re / repeat                    - 重複命令
✅ talk <npc> [topic]             - 對話
✅ check / inspect <npc>          - 查看 NPC
✅ sdl <npc> set/add ...          - 進階對話設置
✅ setrelationship <npc> <val>    - 設置好感度
✅ changerelationship <npc> <delta> - 改變好感度
✅ quest list/active/available... - 任務系統
✅ 以及持續新增的命令
```

**特殊功能**:
```rust
// 重複命令功能
"re" | "repeat" => {
    if let Some(ref last_cmd) = self.last_command {
        return self.handle_command(last_cmd.clone());
    }
}

// 儲存成功的命令
if self.input != "re" && self.input != "repeat" {
    if !matches!(result, CommandResult::Error(_)) {
        self.last_command = Some(self.input.clone());
    }
}
```

**特點**:
- 最完整、最新的實作
- 持續更新維護
- 包含特殊功能（如命令歷史）
- 程式碼較長（600+ 行）

---

### 3. GameEngine::process_command()

**檔案位置**: `src/game_engine.rs`

**功能**:
- 無頭遊戲引擎的命令處理入口
- **內部調用** `CommandProcessor::parse_command()`
- 管理輸出緩衝區

**使用者**:
- ✅ FFI 介面 (ffi.rs)
- ❌ 主程式 (app.rs) **不使用**

**實作**:
```rust
pub fn process_command(&mut self, command: &str) -> (bool, String) {
    // 1. 解析命令（調用 CommandProcessor）
    let cmd_result = self.processor.parse_command(command);
    
    // 2. 檢查退出
    if matches!(cmd_result, CommandResult::Exit) {
        return (false, "退出遊戲".to_string());
    }
    
    // 3. 執行命令
    let result_msg = self.execute_command(cmd_result);
    self.add_output(result_msg.clone());
    
    (true, result_msg)
}
```

**執行流程**:
```
外部調用（FFI）
  ↓
GameEngine::process_command() [此函數]
  ↓
CommandProcessor::parse_command()
  ↓
CommandResult
  ↓
GameEngine::execute_command()
  ↓
返回結果給 FFI
```

**特點**:
- 薄封裝層
- 主要邏輯委派給 CommandProcessor
- 維護輸出緩衝區
- 標記為 `#[allow(dead_code)]`

---

## 使用情況對比表

| 模組/函數 | 主程式 | FFI | 命令數量 | 最新功能 | 維護狀態 |
|----------|--------|-----|---------|---------|---------|
| CommandProcessor::parse_command | ❌ | ✅ | ~30 | ❌ | 停滯 |
| InputHandler::handle_command | ✅ | ❌ | 40+ | ✅ | 活躍 |
| GameEngine::process_command | ❌ | ✅ | ~30 | ❌ | 停滯 |

## 架構圖

```
┌─────────────────────────────────────────────────────────────┐
│                        主程式流程                             │
│                                                               │
│  使用者輸入                                                   │
│      ↓                                                        │
│  InputHandler::handle_input_events()                         │
│      ↓                                                        │
│  InputHandler::handle_command()      [input.rs]              │
│      ↓                                                        │
│  CommandResult (40+ 命令)                                     │
│      ↓                                                        │
│  handle_command_result()             [app.rs]                │
│      ↓                                                        │
│  handle_give(), handle_buy(), ...    [app.rs]                │
│                                                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                        FFI 流程                               │
│                                                               │
│  外部調用（C API）                                            │
│      ↓                                                        │
│  GameEngine::process_command()       [game_engine.rs]        │
│      ↓                                                        │
│  CommandProcessor::parse_command()   [command_processor.rs]  │
│      ↓                                                        │
│  CommandResult (約 30 命令)                                   │
│      ↓                                                        │
│  GameEngine::execute_command()       [game_engine.rs]        │
│      ↓                                                        │
│  返回結果                                                     │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## 問題分析

### 1. 程式碼重複

**重複內容**:
- 命令字串解析邏輯
- 參數提取邏輯
- 錯誤訊息

**重複範圍**:
- `CommandProcessor::parse_command` 和 `InputHandler::handle_command` 約有 70-80% 的程式碼重複

**範例**（buy 命令）:

```rust
// CommandProcessor::parse_command (command_processor.rs)
"buy" => {
    if parts.len() >= 3 {
        let npc = parts[1].to_string();
        let item = parts[2].to_string();
        let quantity = if parts.len() > 3 {
            parts[3].parse::<u32>().unwrap_or(1)
        } else {
            1
        };
        CommandResult::Buy(npc, item, quantity)
    } else {
        CommandResult::Error("用法: buy <NPC> <物品> [數量]".to_string())
    }
}

// InputHandler::handle_command (input.rs)
"buy" => {
    if parts.len() < 3 {
        CommandResult::Error("Usage: buy <npc> <item> [quantity]".to_string())
    } else {
        let npc = parts[1].to_string();
        let item = parts[2].to_string();
        let quantity = if parts.len() > 3 {
            parts[3].parse::<u32>().unwrap_or(1)
        } else {
            1
        };
        CommandResult::Buy(npc, item, quantity)
    }
}
```

### 2. 功能不同步

**CommandProcessor 缺少的功能**:
- 最新命令（give, re, talk, check, quest...）
- 進階對話設置（sdl 的 set/add 語法）
- 好感度系統命令
- 任務系統命令

**影響**:
- FFI 使用者無法使用最新功能
- 維護時需要同步修改兩處
- 容易遺漏更新

### 3. 維護負擔

**當前狀況**:
- 新增命令時，開發者通常只更新 `input.rs`
- `command_processor.rs` 逐漸過時
- 沒有自動化測試確保同步

**證據**:
- `give` 命令（2025-12-25 新增）只在 `input.rs` 中
- `re` 命令（2025-12-25 新增）只在 `input.rs` 中
- quest 相關命令只在 `input.rs` 中

### 4. 設計不一致

**InputHandler 的特殊設計**:
```rust
// 命令歷史功能
pub last_command: Option<String>

// re 命令的遞迴實作
"re" | "repeat" => {
    if let Some(ref last_cmd) = self.last_command {
        return self.handle_command(last_cmd.clone());
    }
}
```

這些功能在 `CommandProcessor` 中沒有對應實作。

## 重構建議

### 方案 1: 統一到 CommandProcessor（推薦）

**步驟**:
1. 將 `InputHandler::handle_command` 的所有命令移植到 `CommandProcessor`
2. 將命令歷史功能抽取為可選特性
3. `InputHandler` 改為調用 `CommandProcessor`
4. 保持兩個流程使用相同的命令解析邏輯

**優點**:
- ✅ 消除程式碼重複
- ✅ 統一維護點
- ✅ FFI 自動獲得最新功能
- ✅ 更好的測試覆蓋

**缺點**:
- ⚠️ 需要重構現有程式碼
- ⚠️ 可能影響 FFI 介面相容性

**實作範例**:
```rust
// command_processor.rs
pub struct CommandProcessor {
    last_command: Option<String>,  // 可選特性
}

impl CommandProcessor {
    pub fn parse_command(&mut self, input: &str) -> CommandResult {
        // 統一的命令解析邏輯
        // 包含所有最新命令
    }
}

// input.rs
impl InputHandler {
    fn handle_command(&mut self, input: String) -> CommandResult {
        // 簡單委派
        self.command_processor.parse_command(&input)
    }
}
```

### 方案 2: 廢棄 CommandProcessor

**步驟**:
1. 標記 `CommandProcessor` 為 deprecated
2. FFI 直接使用 `InputHandler`
3. 或為 FFI 創建專用的簡化介面

**優點**:
- ✅ 簡化架構
- ✅ 單一真實來源
- ✅ 減少維護負擔

**缺點**:
- ⚠️ 打破 FFI 相容性
- ⚠️ `InputHandler` 依賴 Crossterm，可能不適合 FFI

### 方案 3: 抽取共用模組（最佳但最複雜）

**步驟**:
1. 創建 `command_parser.rs` 模組
2. 定義 `CommandParser` trait
3. 實作基本解析邏輯為共用函數
4. `CommandProcessor` 和 `InputHandler` 都使用這些函數

**優點**:
- ✅ 完全消除重複
- ✅ 保持介面獨立
- ✅ 最佳的設計

**缺點**:
- ⚠️ 重構工作量大
- ⚠️ 需要仔細設計介面

**實作範例**:
```rust
// command_parser.rs (新檔案)
pub fn parse_buy_command(parts: &[&str]) -> Result<CommandResult, String> {
    if parts.len() < 3 {
        return Err("Usage: buy <npc> <item> [quantity]".to_string());
    }
    let npc = parts[1].to_string();
    let item = parts[2].to_string();
    let quantity = if parts.len() > 3 {
        parts[3].parse::<u32>().unwrap_or(1)
    } else {
        1
    };
    Ok(CommandResult::Buy(npc, item, quantity))
}

// command_processor.rs 和 input.rs 都使用
"buy" => parse_buy_command(&parts).unwrap_or_else(CommandResult::Error),
```

## 立即行動建議

### 短期（已完成）
- ✅ 加入詳細註解說明各模組的用途
- ✅ 標記哪些函數實際被使用
- ✅ 記錄功能差異

### 中期（建議）
1. 同步缺少的命令到 `CommandProcessor`
   - 至少同步 `give`, `re` 等常用新命令
2. 添加測試確保兩者行為一致
3. 在 CI/CD 中加入檢查

### 長期（待討論）
1. 選擇並執行重構方案（建議方案 1 或 3）
2. 統一命令處理邏輯
3. 改善 FFI 介面設計

## 總結

### 當前狀態
```
✅ 主程式使用 InputHandler::handle_command (最新、完整)
✅ FFI 使用 CommandProcessor::parse_command (基本、過時)
❌ 兩者功能不同步
❌ 存在大量程式碼重複
⚠️ GameEngine::process_command 是薄封裝層
```

### 建議優先級
1. **高**: 同步缺少的命令（give, re, talk, check）
2. **中**: 添加測試確保功能一致性
3. **低**: 執行完整重構（方案 1 或 3）

### 技術債務評估
- **嚴重度**: 中等
- **影響範圍**: FFI 使用者、程式碼維護性
- **解決難度**: 中等
- **建議時程**: 3-6 個月內完成重構

---

**分析日期**: 2025-12-25  
**分析者**: AI Assistant  
**狀態**: ✅ 已加註解，待重構
