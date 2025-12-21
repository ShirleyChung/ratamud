# RataMUD 跨平台架構分析與建議

## 📋 當前狀態

### 1. Terminal UI（已實現）
- **框架**: Ratatui + Crossterm
- **支援平台**: macOS, Linux, Windows
- **遊戲循環**: 
  - 位置: `src/app.rs::run()`
  - 事件驅動: `crossterm::event::poll()`
  - 更新頻率: 60 FPS
  - **限制**: 只能在終端環境運行

### 2. C FFI 接口（部分實現）
- **已實現** (`src/ffi.rs`):
  ```c
  Person* ratamud_create_player(name, desc);
  GameWorld* ratamud_create_world(player);
  int ratamud_load_map(world, map_name);
  char* ratamud_get_player_info(player);
  int ratamud_get_player_position(player, x, y);
  // ... 等等
  ```

- **缺少的關鍵接口**:
  - ❌ 命令處理: `ratamud_process_command()`
  - ❌ 遊戲更新: `ratamud_update()` 
  - ❌ 輸出獲取: `ratamud_get_output()`
  - ❌ 事件輪詢: `ratamud_poll_events()`

## 🎯 問題分析

### 問題 1: 遊戲循環與 UI 耦合
```rust
// src/app.rs - 當前架構
pub fn run(terminal: &mut Terminal<B>, ...) {
    loop {
        // ❌ 緊密耦合到 Ratatui Terminal
        terminal.draw(|f| { ... })?;
        
        // ❌ 依賴 Crossterm 事件
        if event::poll(Duration::from_millis(16))? {
            let event = event::read()?;
            // 處理事件...
        }
    }
}
```

**影響**: 
- iOS/Android/Web 無法使用這個遊戲循環
- 必須重寫整個遊戲邏輯來支援其他平台

### 問題 2: 命令處理邏輯在 UI 層
```rust
// src/input.rs
impl InputHandler {
    pub fn handle_event(&mut self, event: Event) -> Option<CommandResult> {
        // ❌ 依賴 crossterm::event::Event
        match event {
            Event::Key(key) => { ... }
        }
    }
}
```

**影響**:
- 其他平台無法重用命令解析邏輯
- 需要在每個平台重新實現

## ✅ 解決方案

### 方案 A: 添加無頭模式 API（推薦）

保留現有 Terminal 版本，添加獨立的 C API 層：

```
┌─────────────────────────────────────────┐
│ Terminal UI (main.rs)                   │
│ ├─ Crossterm/Ratatui                    │
│ └─ 直接調用遊戲邏輯                      │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│ C FFI Layer (ffi.rs)                    │  ← 新增功能
│ ├─ ratamud_process_command()            │
│ ├─ ratamud_update()                     │
│ ├─ ratamud_get_output()                 │
│ └─ ratamud_poll_events()                │
└─────────────────────────────────────────┘
                ↓↑
┌─────────────────────────────────────────┐
│ 遊戲核心邏輯 (Rust)                      │
│ ├─ CommandProcessor                     │  ← 新增
│ ├─ GameWorld                            │
│ ├─ NpcManager                           │
│ └─ EventSystem                          │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│ iOS/Android/Web UI                      │  ← 未來
│ └─ 調用 C FFI                            │
└─────────────────────────────────────────┘
```

#### 需要新增的模組:

**1. CommandProcessor (新建 `src/command_processor.rs`)**
```rust
pub struct CommandProcessor {
    input_handler: InputHandler,
    output_buffer: Vec<String>,
}

impl CommandProcessor {
    pub fn process_command_str(&mut self, cmd: &str) -> CommandResult {
        // 將字串命令轉換為 CommandResult
        // 不依賴 Crossterm Event
    }
    
    pub fn get_output(&mut self) -> Vec<String> {
        // 獲取輸出訊息
        std::mem::take(&mut self.output_buffer)
    }
}
```

**2. GameEngine (新建 `src/game_engine.rs`)**
```rust
pub struct GameEngine {
    world: GameWorld,
    player: Person,
    processor: CommandProcessor,
}

impl GameEngine {
    pub fn new(player_name: &str) -> Self { ... }
    
    pub fn process_input(&mut self, cmd: &str) -> Result<String, String> {
        let result = self.processor.process_command_str(cmd);
        // 執行命令
        // 返回輸出
    }
    
    pub fn update(&mut self, delta_ms: u32) {
        // 更新遊戲狀態（NPC AI, 事件等）
    }
    
    pub fn get_state_json(&self) -> String {
        // 返回當前狀態的 JSON
    }
}
```

**3. 擴展 FFI (修改 `src/ffi.rs`)**
```rust
// 遊戲引擎實例管理
#[no_mangle]
pub extern "C" fn ratamud_engine_new(player_name: *const c_char) -> *mut GameEngine;

#[no_mangle]
pub extern "C" fn ratamud_engine_free(engine: *mut GameEngine);

// 命令處理
#[no_mangle]
pub extern "C" fn ratamud_engine_process_input(
    engine: *mut GameEngine,
    command: *const c_char
) -> *mut c_char;  // 返回輸出文本

// 遊戲更新
#[no_mangle]
pub extern "C" fn ratamud_engine_update(
    engine: *mut GameEngine,
    delta_ms: u32
);

// 獲取狀態
#[no_mangle]
pub extern "C" fn ratamud_engine_get_state(
    engine: *mut GameEngine
) -> *mut c_char;  // 返回 JSON
```

#### 使用範例 (C/Swift/Java):

**C**:
```c
// 初始化
GameEngine* engine = ratamud_engine_new("Hero");

// 遊戲循環
while (running) {
    // 處理輸入
    char* output = ratamud_engine_process_input(engine, "look");
    printf("%s\n", output);
    ratamud_free_string(output);
    
    // 更新
    ratamud_engine_update(engine, 16);  // 16ms
    
    // 獲取狀態
    char* state = ratamud_engine_get_state(engine);
    // 解析 JSON 更新 UI...
    ratamud_free_string(state);
}

// 清理
ratamud_engine_free(engine);
```

**Swift (iOS)**:
```swift
class GameController {
    private var engine: OpaquePointer?
    
    init() {
        engine = ratamud_engine_new("Hero")
    }
    
    deinit {
        ratamud_engine_free(engine)
    }
    
    func processCommand(_ cmd: String) -> String {
        guard let engine = engine else { return "" }
        
        let output = ratamud_engine_process_input(engine, cmd)
        let result = String(cString: output!)
        ratamud_free_string(output)
        return result
    }
    
    func update(deltaMs: UInt32) {
        ratamud_engine_update(engine, deltaMs)
    }
    
    func getState() -> GameState {
        let json = ratamud_engine_get_state(engine)
        let jsonStr = String(cString: json!)
        ratamud_free_string(json)
        return try! JSONDecoder().decode(GameState.self, from: jsonStr.data(using: .utf8)!)
    }
}
```

### 方案 B: 完全重構（不推薦）

優點: 架構更清晰
缺點: 工作量大，可能破壞現有功能

## 🛠️ 實施步驟（方案 A）

### Phase 1: 核心抽離（1-2 天）
1. ✅ 創建 `command_processor.rs`
   - 將命令解析邏輯從 `input.rs` 抽離
   - 不依賴 Crossterm

2. ✅ 創建 `game_engine.rs`
   - 封裝 GameWorld + CommandProcessor
   - 提供簡單的文本接口

3. ✅ 測試核心邏輯
   - 寫單元測試確保功能正常

### Phase 2: FFI 擴展（1 天）
4. ✅ 擴展 `ffi.rs`
   - 添加 engine_* 系列函數
   - 添加命令處理函數

5. ✅ 更新頭文件
   - 更新 `src/ratamud.h`
   - 添加新的 API 文檔

### Phase 3: 測試與文檔（1 天）
6. ✅ 創建 C 測試程式
   - 驗證 API 可用性

7. ✅ 創建使用範例
   - C 範例
   - Swift 範例（iOS）
   - Kotlin 範例（Android，可選）

8. ✅ 文檔編寫
   - API 參考
   - 集成指南

## 📊 預期效果

### Terminal 版本（不變）
- 保持原有功能
- 繼續使用 Crossterm/Ratatui

### iOS/Android 版本（新增）
- 通過 FFI 調用遊戲邏輯
- 使用原生 UI (SwiftUI / Jetpack Compose)
- 完整的遊戲功能

### Web 版本（未來）
- 編譯為 WASM
- JavaScript 調用
- Canvas/WebGL 渲染

## 💡 其他建議

1. **狀態同步**: 使用 JSON 格式統一狀態表示
2. **事件系統**: 考慮添加事件回調機制
3. **序列化**: 確保所有狀態可序列化/反序列化
4. **線程安全**: 如果需要多線程，添加互斥鎖

## 🎯 結論

**Terminal 版本**: ✅ 已經跨平台（macOS/Linux/Windows）
- 使用 Crossterm，無需修改

**其他平台**: ⚠️ 需要添加無頭模式 API
- 推薦方案 A: 最小改動
- 工作量: 3-4 天
- 效果: 支援所有主流平台
