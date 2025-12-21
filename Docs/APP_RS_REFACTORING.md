# app.rs 使用 GameEngine 重構指南

## 當前狀態

✅ **example.c 已完成**
- 使用新的 GameEngine API
- 所有遊戲命令正常工作（up/down/left/right, status, map, help等）
- 回調系統整合完成

## app.rs 重構方案

由於 app.rs 已經是完整的遊戲，重構需要謹慎。建議採用**漸進式重構**：

### 選項 A: 保持現狀（推薦短期）

**理由**:
1. Terminal 版本已經正常工作
2. UI 層（Ratatui）功能完整
3. 不破壞現有功能

**現狀**:
```rust
// app.rs 直接使用
GameWorld + InputHandler + OutputManager + UI
```

這個方式在 Terminal 環境下完全正常，只是與 Crossterm 耦合。

### 選項 B: 創建雙模式（推薦中期）

**新增模式選擇**:
```rust
pub enum GameMode {
    Terminal,  // 使用 Crossterm + Ratatui（現有）
    Headless,  // 使用 GameEngine（新的）
}

pub fn run_terminal() {
    // 現有的 app::run() 邏輯
}

pub fn run_headless(engine: GameEngine) {
    // 新的無頭模式
}
```

**優點**:
- 不破壞現有功能
- 可選擇性使用新引擎
- 漸進式測試

### 選項 C: 完全重構（長期目標）

**統一架構**:
```rust
// 所有模式都使用 GameEngine
pub struct App {
    engine: GameEngine,
    ui: Box<dyn UIBackend>,  // 可替換的 UI
}

trait UIBackend {
    fn render(&mut self, output: &[String]);
    fn read_input(&mut self) -> Option<String>;
}

struct TerminalUI { /* Ratatui */ }
struct NoUI { /* 純文本 */ }
```

**優點**:
- 架構統一
- 易於擴展
- 完全解耦

**缺點**:
- 工作量大（估計 1-2 天）
- 需要大量測試
- 可能引入新 bug

## 當前建議

### 立即可行（已完成）:

✅ **example.c 作為參考實現**
- 展示如何使用 GameEngine
- 支援所有遊戲命令
- 可移植到其他平台

### 短期（可選）:

**保持 app.rs 現狀**，因為：
1. Terminal 版本已經工作良好
2. 不需要解耦（在終端環境下）
3. 重構風險 > 收益

### 中期（當需要時）:

**添加 run_headless()** 函數：
```rust
// main.rs
pub fn run_headless(player_name: &str) -> std::io::Result<()> {
    let mut engine = GameEngine::new(player_name, "冒險者");
    
    // 載入地圖
    engine.world.load_map("初始之地")?;
    
    // 簡單的文本循環
    loop {
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        
        print!("> ");
        io::Write::flush(&mut io::stdout())?;
        
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        
        let (should_continue, output) = engine.process_command(line.trim());
        println!("{}", output);
        
        if !should_continue {
            break;
        }
    }
    
    Ok(())
}

// 在 main() 中選擇模式
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--headless" {
        run_headless("玩家")
    } else {
        run()  // 現有的 Terminal UI
    }
}
```

## 測試建議

### 1. Terminal 版本（現有）
```bash
cargo run
```
繼續使用 Crossterm + Ratatui

### 2. C 客戶端（新增）
```bash
cd dist
./example
```
使用 GameEngine API

### 3. 無頭模式（可選）
```bash
cargo run -- --headless
```
純文本，使用 GameEngine

## 實際使用場景

### Terminal 用戶
- 運行 `cargo run`
- 使用完整的 TUI（圖形界面、小地圖、日誌等）
- 不需要改變

### 開發者（測試/調試）
- 運行 `cargo run -- --headless`
- 純文本，更簡單
- 或使用 `dist/example`（C 版本）

### iOS/Android 開發者
- 調用 C API（example.c 的模式）
- 使用 GameEngine
- 構建自己的 UI

## 結論

**當前最佳方案**:

1. ✅ **保持 app.rs 不變** - Terminal UI 繼續工作
2. ✅ **使用 example.c** - 作為跨平台參考
3. ⚠️ **可選添加 --headless 模式** - 如果需要純文本測試

**不建議立即重構 app.rs**，因為：
- Terminal 版本已經完美工作
- example.c 已提供跨平台方案
- 重構風險大於收益

## 下一步

根據實際需求選擇：

### A. 如果只需要跨平台開發
→ 使用 example.c 模式，**無需修改 app.rs**

### B. 如果需要純文本調試模式
→ 添加 `run_headless()` 函數（30 分鐘工作）

### C. 如果要完全統一架構
→ 重構 app.rs 使用 GameEngine（1-2 天工作）

## 當前狀態總結

✅ **核心目標已達成**:
- GameEngine 已解耦
- C API 完整可用
- example.c 展示所有功能
- 可以開發跨平台客戶端

✅ **Terminal 版本**:
- 保持現狀即可
- 功能完整
- 不需要修改

🎯 **建議**:
暫時不修改 app.rs，使用 example.c 作為跨平台開發的基礎。
