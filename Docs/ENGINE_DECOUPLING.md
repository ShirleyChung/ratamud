# 遊戲引擎解耦合重構

## 完成狀態

### ✅ Phase 1: 核心模組創建（完成）

已創建兩個新的核心模組，完全獨立於 Crossterm/Ratatui：

#### 1. `src/command_processor.rs`
**純文本命令處理器**

- ✅ 解析文本命令字串
- ✅ 不依賴鍵盤事件（Crossterm Event）
- ✅ 返回 CommandResult 枚舉
- ✅ 支援所有遊戲命令：
  - 移動：up/down/left/right, move, goto
  - 查看：look, status, map
  - 物品：get, drop, eat
  - NPC：summon, ctrl, trade, buy, sell, npcs
  - 對話：setdialogue, seteagerness
  - 系統：help, clear, quit
  
**使用範例**:
```rust
let processor = CommandProcessor::new();
let result = processor.parse_command("move 10 20");
// 返回 CommandResult::Move(10, 20)
```

#### 2. `src/game_engine.rs`
**無頭遊戲引擎核心**

- ✅ 獨立的遊戲邏輯
- ✅ 不依賴終端 UI
- ✅ 輸出緩衝管理
- ✅ 回調系統整合
- ✅ JSON 狀態導出

**功能**:
```rust
pub struct GameEngine {
    pub world: GameWorld,
    pub player: Person,
    processor: CommandProcessor,
    output_buffer: VecDeque<String>,
}

impl GameEngine {
    // 處理命令
    pub fn process_command(&mut self, command: &str) -> (bool, String)
    
    // 獲取輸出
    pub fn get_output(&mut self) -> Vec<String>
    
    // 獲取狀態（JSON）
    pub fn get_state_json(&self) -> String
    
    // 更新遊戲邏輯
    pub fn update(&mut self, delta_ms: u32)
}
```

### ✅ Phase 2: FFI 擴展（完成）

#### 新增 API (`src/ffi.rs`):

**引擎管理**:
```c
// 創建無頭遊戲引擎
GameEngine* ratamud_create_engine(const char* player_name);

// 釋放引擎
void ratamud_free_engine(GameEngine* engine);
```

**命令處理**:
```c
// 處理命令（返回 1=繼續, 0=退出, -1=錯誤）
int ratamud_engine_process_command(GameEngine* engine, const char* command);
```

**輸出管理**:
```c
// 獲取輸出（清空緩衝區）
char* ratamud_engine_get_output(GameEngine* engine);
```

**狀態查詢**:
```c
// 獲取遊戲狀態（JSON）
char* ratamud_engine_get_state(GameEngine* engine);
```

**遊戲更新**:
```c
// 更新遊戲邏輯
void ratamud_engine_update(GameEngine* engine, uint32_t delta_ms);
```

### ✅ 編譯狀態

```
✓ 所有模組編譯成功
✓ 無錯誤
⚠️ 8 個警告（未使用的函數，正常）
```

## 架構改進

### Before（耦合）:
```
Terminal UI (Crossterm + Ratatui)
    ↓
  遊戲邏輯
```
- ❌ 無法在其他平台使用
- ❌ 必須在終端運行

### After（解耦）:
```
┌─────────────────────────┐
│ UI 層 (可選)             │
│ ├─ Terminal (Crossterm) │
│ ├─ iOS (SwiftUI)        │
│ ├─ Android (Compose)    │
│ └─ Web (Canvas)         │
└─────────────────────────┘
          ↓↑ FFI
┌─────────────────────────┐
│ 遊戲引擎核心             │
│ • CommandProcessor      │
│ • GameEngine            │
│ • 純邏輯，無 UI 依賴    │
└─────────────────────────┘
```
- ✅ 跨平台
- ✅ 可獨立測試
- ✅ UI 無關

## 使用範例

### C 客戶端:
```c
// 創建引擎
GameEngine* engine = ratamud_create_engine("勇者");

// 遊戲循環
while (running) {
    // 讀取用戶輸入
    char input[256];
    fgets(input, sizeof(input), stdin);
    
    // 處理命令
    int result = ratamud_engine_process_command(engine, input);
    if (result == 0) break;  // 退出
    
    // 獲取輸出
    char* output = ratamud_engine_get_output(engine);
    printf("%s\n", output);
    ratamud_free_string(output);
    
    // 更新遊戲
    ratamud_engine_update(engine, 16);
}

// 清理
ratamud_free_engine(engine);
```

### Swift (iOS):
```swift
class GameController {
    private var engine: OpaquePointer?
    
    init() {
        engine = ratamud_create_engine("Hero")
    }
    
    func processCommand(_ cmd: String) {
        let result = ratamud_engine_process_command(engine, cmd)
        
        let outputPtr = ratamud_engine_get_output(engine)
        let output = String(cString: outputPtr!)
        ratamud_free_string(outputPtr)
        
        // 更新 UI
        updateUI(output)
    }
    
    func update() {
        ratamud_engine_update(engine, 16)
    }
}
```

## ⚠️ 待完成工作

### Phase 3: Terminal UI 適配（未完成）

**需要修改 `src/app.rs`**:
- 當前仍使用舊的耦合架構
- 需要重構為使用 GameEngine
- 估計工作量: 2-3 小時

**步驟**:
1. 修改 `app::run()` 創建 GameEngine 實例
2. 鍵盤輸入轉為文本命令
3. 調用 `engine.process_command()`
4. 從 `engine.get_output()` 獲取輸出
5. 渲染到 Ratatui UI

### Phase 4: example.c 更新（未完成）

**更新 example.c 使用新 API**:
- 使用 `ratamud_create_engine()`
- 使用 `ratamud_engine_process_command()`
- 所有遊戲命令都能正常工作
- 估計工作量: 1 小時

## 測試建議

### 1. 測試無頭引擎（C）:
```bash
# 編譯測試程式
./build_dylib.sh
cd dist
gcc -o test_engine test_engine.c -L. -lratamud -Wl,-rpath,.
./test_engine
```

### 2. 測試命令處理:
```rust
#[test]
fn test_command_processor() {
    let proc = CommandProcessor::new();
    
    // 測試移動
    assert!(matches!(
        proc.parse_command("move 10 20"),
        CommandResult::Move(10, 20)
    ));
    
    // 測試方向
    assert!(matches!(
        proc.parse_command("up"),
        CommandResult::Move(0, -1)
    ));
    
    // 測試退出
    assert!(matches!(
        proc.parse_command("quit"),
        CommandResult::Exit
    ));
}
```

### 3. 測試遊戲引擎:
```rust
#[test]
fn test_game_engine() {
    let mut engine = GameEngine::new("測試", "玩家");
    
    // 測試命令
    let (cont, msg) = engine.process_command("status");
    assert!(cont);
    assert!(msg.contains("測試"));
    
    // 測試輸出
    let output = engine.get_output();
    assert!(!output.is_empty());
}
```

## 優勢

### 1. 真正的跨平台
- Terminal、iOS、Android、Web 共用同一遊戲邏輯
- 只需為每個平台實現 UI 層

### 2. 可測試性
- 遊戲邏輯可獨立單元測試
- 不需要模擬終端環境

### 3. 靈活性
- 可以在任何環境運行遊戲邏輯
- 輸出可以是文本、JSON、或任何格式

### 4. 維護性
- UI 和邏輯分離
- 修改一處，所有平台受益

## 下一步

1. **立即**: 更新 example.c 使用新 API
2. **短期**: 重構 app.rs 使用 GameEngine
3. **中期**: 創建 iOS/Android 客戶端
4. **長期**: Web 版本（WASM）

## 文檔

相關文檔已更新：
- ✅ `CALLBACK_USAGE.md` - 回調系統
- ✅ `CROSS_PLATFORM_ARCHITECTURE.md` - 架構說明
- ⚠️  需要更新 `dist/README.md` - 新 API 說明
- ⚠️  需要更新 `src/ratamud.h` - 新函數聲明

## 總結

✅ **核心工作已完成**：遊戲引擎已與 UI 解耦
⚠️ **剩餘工作**：更新現有客戶端使用新引擎
🎯 **目標達成**：現在可以在任何平台開發遊戲客戶端！
