# 輸出回調機制使用指南

## 概述

`OutputManager` 現在支持輸出回調機制，讓系統可以作為 library 被其他程式使用，並實時接收遊戲輸出。

## 功能說明

### 輸出類型標記

所有輸出都會帶有類型標記，用於區分不同來源的訊息：

| 標記 | 說明 | 觸發方法 |
|------|------|----------|
| `MAIN` | 主遊戲訊息（移動、戰鬥、對話等） | `print()` |
| `LOG` | 系統日誌（帶時間戳） | `log()` |
| `STATUS` | 狀態欄訊息（5秒自動清除） | `set_status()` |
| `SIDE` | 側邊面板內容（NPC信息等） | `set_side_content()` |

### 使用方式

```rust
use ratamud::output::OutputManager;

fn main() {
    let mut output_manager = OutputManager::new();
    
    // 設置回調函數
    output_manager.set_output_callback(|msg_type, content| {
        // 這裡可以做任何事：
        // - 寫入文件：log:xxxxxxxx
        // - 發送到網絡
        // - 更新 GUI
        // - 存入數據庫
        
        match msg_type {
            "MAIN" => println!("[遊戲] {}", content),
            "LOG" => println!("[日誌] {}", content),
            "STATUS" => println!("[狀態] {}", content),
            "SIDE" => println!("[側邊] {}", content),
            _ => println!("[未知] {}", content),
        }
    });
    
    // 正常使用 OutputManager
    output_manager.print("你向北移動".to_string());
    output_manager.log("遊戲初始化完成".to_string());
    output_manager.set_status("保存成功".to_string());
}
```

### 文件輸出範例

如果要輸出到文件，可以這樣實現：

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

fn setup_file_logger(output_manager: &mut OutputManager) {
    // 創建文件寫入器（使用 Arc<Mutex> 讓回調可以安全使用）
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("game_output.log")
            .expect("無法創建日誌文件")
    ));
    
    // 設置回調
    output_manager.set_output_callback(move |msg_type, content| {
        if let Ok(mut f) = file.lock() {
            // 寫入格式：類型:內容
            writeln!(f, "{}:{}", msg_type, content).ok();
        }
    });
}
```

輸出結果示例：
```
MAIN:你向北移動
LOG:遊戲初始化完成
MAIN:你攻擊了野豬，造成15點傷害
STATUS:保存成功
SIDE:NPC: 商人\n等級: 10\n生命: 100/100
```

## 注意事項

1. **回調函數目前不做任何事**：這是設計初衷，讓使用者自行決定如何處理輸出
2. **線程安全**：回調函數必須是 `Send` trait，可安全跨線程使用
3. **不影響現有功能**：即使不設置回調，遊戲仍正常顯示在終端
4. **性能考量**：回調在輸出方法內同步調用，避免在回調中執行耗時操作

## 未來擴展

將來可以考慮添加：
- 更多輸出類型（COMBAT, DIALOGUE, ERROR 等）
- 異步回調支持
- 回調過濾器（只接收特定類型的輸出）
- 多個回調支持（觀察者模式）
