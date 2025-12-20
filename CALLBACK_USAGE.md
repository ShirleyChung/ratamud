# Callback 系統使用指南

## 概述

RataMUD 現在支援事件驅動的回調系統，允許任何平台（iOS、Android、Web 等）通過註冊回調函數來接收遊戲輸出和狀態變化。

## 回調類型

### 1. 輸出回調 (OutputCallback)
接收所有遊戲文本輸出（如命令結果、NPC 對話等）

```c
typedef void (*OutputCallback)(const char* message);
```

### 2. 狀態回調 (StateCallback)  
接收遊戲狀態變化（JSON 格式）

```c
typedef void (*StateCallback)(const char* state_json);
```

### 3. 事件回調 (EventCallback)
接收遊戲事件（如 NPC 移動、物品拾取等）

```c
typedef void (*EventCallback)(const char* event_type, const char* event_data);
```

## C 使用範例

```c
#include <stdio.h>
#include "ratamud.h"

// 定義輸出回調函數
void on_game_output(const char* message) {
    printf("[遊戲輸出] %s\n", message);
    // 可以將訊息發送到 UI 線程更新顯示
}

// 定義狀態回調函數
void on_state_change(const char* state_json) {
    printf("[狀態變化] %s\n", state_json);
    // 解析 JSON 並更新 UI
}

// 定義事件回調函數
void on_game_event(const char* event_type, const char* event_data) {
    printf("[遊戲事件] %s: %s\n", event_type, event_data);
    // 處理特定事件
}

int main() {
    // 註冊回調
    ratamud_register_output_callback(on_game_output);
    ratamud_register_state_callback(on_state_change);
    ratamud_register_event_callback(on_game_event);
    
    // 創建遊戲
    Person* player = ratamud_create_player("Hero", "Brave adventurer");
    GameWorld* world = ratamud_create_world(player);
    
    // 載入地圖 - 自動觸發輸出回調
    ratamud_load_map(world, "初始之地");
    
    // 遊戲運行...
    // 所有 print() 輸出都會通過 on_game_output 發送
    
    // 清理
    ratamud_unregister_output_callback();
    ratamud_unregister_state_callback();
    ratamud_unregister_event_callback();
    
    ratamud_free_world(world);
    ratamud_free_player(player);
    
    return 0;
}
```

## Swift (iOS) 使用範例

```swift
import Foundation

class GameController {
    private var player: OpaquePointer?
    private var world: OpaquePointer?
    
    // UI 更新閉包
    var onOutput: ((String) -> Void)?
    var onStateChange: ((String) -> Void)?
    
    init() {
        // 設置回調
        setupCallbacks()
        
        // 創建遊戲
        player = ratamud_create_player("Hero", "Brave adventurer")
        if let player = player {
            world = ratamud_create_world(player)
        }
    }
    
    deinit {
        ratamud_unregister_output_callback()
        ratamud_unregister_state_callback()
        ratamud_free_world(world)
        ratamud_free_player(player)
    }
    
    private func setupCallbacks() {
        // 輸出回調
        let outputCallback: OutputCallback = { messagePtr in
            guard let ptr = messagePtr else { return }
            let message = String(cString: ptr)
            
            // 在主線程更新 UI
            DispatchQueue.main.async {
                // 通知 SwiftUI 更新
                NotificationCenter.default.post(
                    name: .gameOutputReceived,
                    object: message
                )
            }
        }
        
        // 狀態回調
        let stateCallback: StateCallback = { statePtr in
            guard let ptr = statePtr else { return }
            let json = String(cString: ptr)
            
            DispatchQueue.main.async {
                // 解析 JSON 並更新狀態
                if let data = json.data(using: .utf8),
                   let state = try? JSONDecoder().decode(GameState.self, from: data) {
                    NotificationCenter.default.post(
                        name: .gameStateChanged,
                        object: state
                    )
                }
            }
        }
        
        // 註冊回調
        ratamud_register_output_callback(outputCallback)
        ratamud_register_state_callback(stateCallback)
    }
    
    func loadMap(_ name: String) {
        guard let world = world else { return }
        _ = ratamud_load_map(world, name)
        // 輸出會自動通過回調發送到 UI
    }
}

// SwiftUI View 範例
struct GameView: View {
    @State private var messages: [String] = []
    @StateObject private var gameController = GameController()
    
    var body: some View {
        VStack {
            // 顯示遊戲輸出
            ScrollView {
                ForEach(messages, id: \.self) { message in
                    Text(message)
                        .padding(4)
                }
            }
            
            // 命令輸入
            TextField("輸入命令", text: $commandText)
                .onSubmit {
                    // 處理命令...
                }
        }
        .onReceive(NotificationCenter.default.publisher(for: .gameOutputReceived)) { notification in
            if let message = notification.object as? String {
                messages.append(message)
            }
        }
    }
}

// 通知名稱擴展
extension Notification.Name {
    static let gameOutputReceived = Notification.Name("gameOutputReceived")
    static let gameStateChanged = Notification.Name("gameStateChanged")
}
```

## Kotlin (Android) 使用範例

```kotlin
class GameController {
    private var player: Long = 0
    private var world: Long = 0
    
    // LiveData 用於 UI 更新
    val gameOutput = MutableLiveData<String>()
    val gameState = MutableLiveData<GameState>()
    
    init {
        // 載入 native library
        System.loadLibrary("ratamud")
        
        // 設置回調
        setupCallbacks()
        
        // 創建遊戲
        player = createPlayer("Hero", "Brave adventurer")
        world = createWorld(player)
    }
    
    private fun setupCallbacks() {
        // 輸出回調
        registerOutputCallback { message ->
            // 在主線程更新 UI
            Handler(Looper.getMainLooper()).post {
                gameOutput.value = message
            }
        }
        
        // 狀態回調
        registerStateCallback { stateJson ->
            Handler(Looper.getMainLooper()).post {
                val state = Gson().fromJson(stateJson, GameState::class.java)
                gameState.value = state
            }
        }
    }
    
    fun loadMap(name: String) {
        nativeLoadMap(world, name)
        // 輸出會自動通過回調發送
    }
    
    fun cleanup() {
        unregisterOutputCallback()
        unregisterStateCallback()
        freeWorld(world)
        freePlayer(player)
    }
    
    // Native 方法聲明
    private external fun registerOutputCallback(callback: (String) -> Unit)
    private external fun registerStateCallback(callback: (String) -> Unit)
    private external fun unregisterOutputCallback()
    private external fun unregisterStateCallback()
    private external fun createPlayer(name: String, desc: String): Long
    private external fun createWorld(player: Long): Long
    private external fun nativeLoadMap(world: Long, name: String)
    private external fun freeWorld(world: Long)
    private external fun freePlayer(player: Long)
}

// Jetpack Compose UI
@Composable
fun GameScreen(viewModel: GameViewModel) {
    val messages by viewModel.gameOutput.observeAsState(initial = emptyList())
    
    Column {
        // 顯示遊戲輸出
        LazyColumn(modifier = Modifier.weight(1f)) {
            items(messages) { message ->
                Text(
                    text = message,
                    modifier = Modifier.padding(4.dp)
                )
            }
        }
        
        // 命令輸入
        TextField(
            value = commandText,
            onValueChange = { commandText = it },
            modifier = Modifier.fillMaxWidth()
        )
    }
}
```

## JavaScript (WASM) 使用範例

```javascript
// 載入 WASM 模組
const ratamud = await import('./ratamud_wasm.js');

// 設置回調
ratamud.register_output_callback((message) => {
    console.log('[遊戲輸出]', message);
    
    // 更新 DOM
    const outputDiv = document.getElementById('game-output');
    const p = document.createElement('p');
    p.textContent = message;
    outputDiv.appendChild(p);
    
    // 自動滾動到底部
    outputDiv.scrollTop = outputDiv.scrollHeight;
});

ratamud.register_state_callback((stateJson) => {
    const state = JSON.parse(stateJson);
    console.log('[狀態]', state);
    
    // 更新 UI
    updatePlayerInfo(state.player);
    updateMap(state.world);
});

// 創建遊戲
const player = ratamud.create_player("Hero", "Brave adventurer");
const world = ratamud.create_world(player);

// 載入地圖
ratamud.load_map(world, "初始之地");
// 輸出會自動通過回調顯示在頁面上
```

## 優點

### 1. 平台無關
- ✅ 不依賴特定 UI 框架
- ✅ 可用於任何支援 C FFI 的語言
- ✅ 事件驅動，解耦合

### 2. 即時更新
- ✅ 遊戲輸出立即通知 UI
- ✅ 不需要輪詢
- ✅ 減少延遲

### 3. 靈活性
- ✅ 可選擇性註冊需要的回調
- ✅ 可以隨時註冊/取消註冊
- ✅ 支援多種數據格式（文本、JSON）

### 4. 效能
- ✅ 零拷貝（直接傳遞指針）
- ✅ 異步處理
- ✅ 不阻塞遊戲邏輯

## 線程安全注意事項

回調函數可能在不同線程調用，建議：

1. **iOS/Swift**: 使用 `DispatchQueue.main.async` 更新 UI
2. **Android/Kotlin**: 使用 `Handler(Looper.getMainLooper()).post`
3. **Web/JS**: 瀏覽器自動處理（單線程）
4. **C/C++**: 使用互斥鎖保護共享數據

## 調試

啟用回調日誌：

```c
void debug_output_callback(const char* msg) {
    fprintf(stderr, "[DEBUG] Output: %s\n", msg);
}

ratamud_register_output_callback(debug_output_callback);
```

## 完整範例

查看:
- `dist/example_callback.c` - C 範例
- `IOS_FRAMEWORK_README.md` - iOS 集成
- `CROSS_PLATFORM_ARCHITECTURE.md` - 架構說明
