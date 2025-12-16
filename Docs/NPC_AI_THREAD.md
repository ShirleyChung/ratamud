# NPC AI 獨立執行緒實現

## 概述

NPC AI 系統已改用獨立執行緒運行，避免阻塞主遊戲迴圈，確保遊戲流暢運行。

## 架構設計

### 1. **NPC AI 執行緒** (`npc_ai_thread.rs`)
- 使用泛型設計，接受任何 `FnMut() -> Vec<String>` 閉包
- 執行緒週期性執行 AI 邏輯（預設每 5 秒）
- 收集 AI 日誌並存入共享佇列
- 自動在 Drop 時清理資源

```rust
pub struct NpcAiThread {
    handle: Option<thread::JoinHandle<()>>,
    should_stop: Arc<Mutex<bool>>,
    ai_logs: Arc<Mutex<Vec<String>>>,
}
```

### 2. **執行緒安全的資料共享**
使用 `Arc<Mutex<T>>` 在主執行緒與 AI 執行緒間共享資料：

- **NpcManager**: `Arc<Mutex<NpcManager>>` - NPC 狀態
- **Maps**: `Arc<Mutex<HashMap<String, Map>>>` - 所有地圖資料
- **CurrentMap**: `Arc<Mutex<String>>` - 當前地圖名稱

主迴圈定期**雙向同步**，使用 `try_lock()` 非阻塞檢查。

### 3. **AI 行為系統**
AI 執行緒中執行的行為包括：
- **UseFood**: 生命值低時尋找食物
- **Wander**: 四處遊蕩
- **Farm**: 農夫耕作
- **PickupItems**: 撿拾物品
- **Trade**: 商人交易（被動）
- **Idle**: 閒置

## 實現細節

### 啟動流程（在 `app.rs::run_main_loop()`）

```rust
// 1. 將 GameWorld 的資料移出並用 Arc<Mutex> 包裝
let npc_manager = Arc::new(Mutex::new(std::mem::take(&mut game_world.npc_manager)));
let npc_manager_clone = Arc::clone(&npc_manager);

let maps = Arc::new(Mutex::new(std::mem::take(&mut game_world.maps)));
let maps_clone = Arc::clone(&maps);

let current_map = Arc::new(Mutex::new(game_world.current_map.clone()));
let current_map_clone = Arc::clone(&current_map);

// 2. 創建 AI 執行緒
let npc_ai_thread = NpcAiThread::new(
    move || {
        // AI 邏輯在此閉包中執行
        if let (Ok(mut manager), Ok(maps), Ok(current_map_name)) = 
            (npc_manager_clone.lock(), maps_clone.lock(), current_map_clone.lock()) {
            // 遍歷所有 NPC，執行 AI 決策
            // AI 可以讀取和修改地圖資料
            // 返回日誌訊息
        }
    },
    5000  // 每 5 秒執行一次
);
game_world.npc_ai_thread = Some(npc_ai_thread);
```

### 主迴圈雙向同步

```rust
'main_loop: loop {
    // 雙向同步 NpcManager 和 Maps
    if let (Ok(manager), Ok(mut maps_lock), Ok(mut current_map_lock)) = 
        (npc_manager.try_lock(), maps.try_lock(), current_map.try_lock()) {
        
        // 1. NPC 狀態同步（AI -> 主執行緒）
        for npc_id in manager.get_all_npc_ids() {
            if let Some(npc) = manager.get_npc(&npc_id) {
                game_world.npc_manager.update_npc(&npc_id, npc.clone());
            }
        }
        
        // 2. 地圖同步（主執行緒 -> AI）
        *maps_lock = game_world.maps.clone();
        *current_map_lock = game_world.current_map.clone();
        
        // 3. 地圖變更同步（AI -> 主執行緒）
        for (map_name, ai_map) in maps_lock.iter() {
            if let Some(world_map) = game_world.maps.get_mut(map_name) {
                // 同步每個點的物品和狀態（如 NPC 撿物品的結果）
                for y in 0..world_map.height.min(ai_map.height) {
                    for x in 0..world_map.width.min(ai_map.width) {
                        if let Some(ai_point) = ai_map.points.get(y).and_then(|row| row.get(x)) {
                            if let Some(world_point) = world_map.points.get_mut(y).and_then(|row| row.get_mut(x)) {
                                world_point.objects = ai_point.objects.clone();
                                world_point.object_ages = ai_point.object_ages.clone();
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 獲取 AI 日誌
    let ai_logs = game_world.get_npc_ai_logs();
    for log in ai_logs {
        output_manager.log(log);
    }
    
    // ... 其他遊戲邏輯
}
```

### 清理流程

```rust
// 在迴圈結束前恢復資源回 game_world
if let Ok(manager) = Arc::try_unwrap(npc_manager) {
    game_world.npc_manager = manager.into_inner().unwrap();
}
if let Ok(maps_data) = Arc::try_unwrap(maps) {
    game_world.maps = maps_data.into_inner().unwrap();
}
```

## 地圖同步機制

### 為什麼需要地圖同步？

AI 執行緒需要讀取地圖資料來：
- 判斷 NPC 能否移動到某個位置
- 檢查地面上有什麼物品可撿
- 找尋農地進行耕作
- 尋找商店進行交易

### 雙向同步流程

1. **主執行緒 → AI 執行緒**（每幀）
   - 同步最新的地圖狀態
   - 確保 AI 看到玩家的操作結果
   - 例如：玩家切換地圖、拾取物品

2. **AI 執行緒 → 主執行緒**（每幀）
   - 同步 AI 修改的地圖資料
   - 只同步可變動的部分（物品、物件年齡）
   - 例如：NPC 撿起物品、種植作物

### 同步的資料

| 資料類型 | 方向 | 說明 |
|---------|------|------|
| 地圖結構 | 主 → AI | 完整地圖克隆 |
| 當前地圖 | 主 → AI | 地圖切換通知 |
| Point.objects | AI → 主 | 地面物品變更 |
| Point.object_ages | AI → 主 | 物品年齡追蹤 |

## 優勢

1. **非阻塞**：NPC AI 在獨立執行緒運行，不影響主遊戲迴圈
2. **即時同步**：主執行緒與 AI 執行緒的地圖資料保持一致
3. **可擴展**：未來可以輕鬆添加其他執行緒任務
4. **安全**：使用 Rust 的所有權系統和 Arc<Mutex> 確保執行緒安全
5. **效能**：AI 計算與遊戲渲染並行執行
6. **彈性調整**：可以輕鬆調整 AI 更新頻率

## 時間系統對比

| 系統 | 執行緒 | 更新頻率 | 用途 |
|------|--------|----------|------|
| TimeThread | 獨立 | 每秒 | 遊戲時間流動 |
| NpcAiThread | 獨立 | 每 5 秒 | NPC 行為決策 |
| 資料同步 | 主執行緒 | 每幀 | NPC/地圖狀態同步 |
| 事件檢查 | 主執行緒 | 每 0.1 秒 | 遊戲事件觸發 |

## 未來擴展

可以參考此架構實現更多獨立執行緒任務：

```rust
// 敵人 AI 執行緒
pub struct EnemyAiThread { ... }

// 環境系統執行緒（天氣、季節等）
pub struct EnvironmentThread { ... }

// 經濟系統執行緒（市場價格波動）
pub struct EconomyThread { ... }
```

## 注意事項

### 1. 避免死鎖
- 始終使用 `try_lock()` 或確保鎖的持有時間極短
- 不要在持有鎖時調用其他可能加鎖的函數
- 按固定順序獲取多個鎖

### 2. 資料同步
- 主迴圈需定期同步執行緒的變更
- 使用 `try_lock()` 避免阻塞主迴圈
- 只同步必要的資料，減少克隆開銷

### 3. 資源清理
- 確保在程式結束前正確恢復共享資源
- `Arc::try_unwrap()` 需要所有引用都釋放

### 4. 效能監控
- 監控執行緒的執行時間，避免過度計算
- 地圖克隆有開銷，考慮使用更細粒度的鎖
- 可以使用 `Arc<RwLock>` 允許多讀單寫

## 測試

```bash
# 編譯
cargo build

# 運行
cargo run --bin main

# NPC AI 日誌會每 5 秒在遊戲中顯示，例如：
# 🚶 旅行者 正在四處遊蕩
# 🌾 農夫 老王 正在耕作
# 🍎 商人 正在尋找食物恢復生命 (HP: 30/100)
```

## 效能優化建議

### 當前實現
- 每幀完整克隆地圖資料（約 100x100 點）
- 適合小型地圖和少量 NPC

### 未來優化
1. **分塊同步**：只同步 NPC 附近的地圖區域
2. **變更追蹤**：只傳遞變更的點，不是整個地圖
3. **讀寫鎖**：使用 `RwLock` 允許多個 AI 同時讀取
4. **事件驅動**：地圖變更時發送事件通知 AI

```rust
// 優化範例：只同步 NPC 視野範圍
let view_distance = 10;
for npc in npcs {
    let nearby_points = get_points_in_range(npc.x, npc.y, view_distance);
    // 只同步這些點
}
```

## 相關檔案

- `src/npc_ai_thread.rs` - AI 執行緒實現
- `src/npc_ai.rs` - AI 行為邏輯
- `src/npc_manager.rs` - NPC 管理器
- `src/app.rs` - 主迴圈整合與資料同步
- `src/world.rs` - 遊戲世界結構
- `src/map.rs` - 地圖和點資料結構
