# GameWorld 多執行緒架構重構計畫

## 目標
按照規格文件要求，移除 Arc<Mutex<GameWorld>> 模式，改用事件驅動架構

## 階段一：定義核心數據結構 ✨

### 1.1 創建 NpcView (src/npc_view.rs)
```rust
/// NPC 可見的世界快照 (不可變)
#[derive(Clone)]
pub struct NpcView {
    pub self_id: String,
    pub self_pos: Position,
    pub nearby_entities: Vec<EntityInfo>,
    pub time: GameTime,
    pub current_map: String,
    pub visible_items: Vec<ItemInfo>,
    pub terrain: TerrainInfo,
}

#[derive(Clone)]
pub struct EntityInfo {
    pub entity_type: EntityType,  // Player, Npc, Item
    pub id: String,
    pub pos: Position,
    pub name: String,
}

#[derive(Clone)]
pub enum EntityType {
    Player,
    Npc,
    Item,
}
```

### 1.2 創建 NpcAction (src/npc_action.rs)
```rust
/// NPC 意圖（不可變）
#[derive(Clone, Debug)]
pub enum NpcAction {
    Say(String),
    Move(Direction),
    PickupItem(String),
    UseItem(String),
    Trade { target: String },
    Idle,
}

#[derive(Clone, Debug)]
pub enum Direction {
    Up, Down, Left, Right
}
```

### 1.3 創建統一的 Event 系統 (src/game_event.rs)
```rust
/// 遊戲事件（跨執行緒通訊）
#[derive(Clone, Debug)]
pub enum GameEvent {
    NpcActions { npc_id: String, actions: Vec<NpcAction> },
    TimerTick { elapsed_secs: u64 },
    Input(InputEvent),
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Command(String),
    KeyPress(Key),
}
```

### 1.4 創建 Message 系統 (src/message.rs)
```rust
/// 輸出訊息（GameWorld → OutputManager）
#[derive(Clone, Debug)]
pub enum Message {
    NpcSay { npc_id: String, text: String },
    System(String),
    Combat { attacker: String, target: String, damage: i32 },
    ItemPickup { entity: String, item: String, count: u32 },
}
```

## 階段二：重構 NPC AI 系統 🤖

### 2.1 修改 NpcAiController (src/npc_ai.rs)
**目前**:
```rust
pub fn update_npc_with_components(
    npc_manager: &mut NpcManager,
    maps: &mut HashMap<String, Map>,
    npc_id: &str,
) -> Option<String>
```

**改為**:
```rust
pub fn decide_action(
    npc_view: &NpcView,
    npc: &Person,
) -> Option<NpcAction>
```

### 2.2 移除 Arc<Mutex> 從 app.rs
**刪除**:
```rust
// app.rs:97-106
let npc_manager = Arc::new(Mutex::new(...));
let maps = Arc::new(Mutex::new(...));
```

**改為使用 channel**:
```rust
let (npc_event_tx, npc_event_rx) = mpsc::channel::<GameEvent>();
```

## 階段三：重構 Game Loop 🔄

### 3.1 修改 NpcAiThread (src/npc_ai_thread.rs)
**目前**:
```rust
pub fn new<F>(mut update_fn: F, ...) where F: FnMut() -> Vec<String>
```

**改為**:
```rust
pub struct NpcAiThread {
    event_sender: mpsc::Sender<GameEvent>,
}

impl NpcAiThread {
    pub fn new(
        npc_views_rx: mpsc::Receiver<HashMap<String, NpcView>>,
        event_tx: mpsc::Sender<GameEvent>,
    ) -> Self {
        thread::spawn(move || {
            while let Ok(npc_views) = npc_views_rx.recv() {
                for (npc_id, view) in npc_views {
                    if let Some(action) = NpcAiController::decide_action(&view, ...) {
                        let _ = event_tx.send(GameEvent::NpcActions {
                            npc_id,
                            actions: vec![action],
                        });
                    }
                }
                thread::sleep(Duration::from_secs(5));
            }
        });
        Self { event_sender: event_tx }
    }
}
```

### 3.2 修改主迴圈 (app.rs::run_main_loop)
```rust
pub fn run_main_loop(...) -> Result<(), Box<dyn std::error::Error>> {
    // === Channel 設定 ===
    let (input_tx, input_rx) = mpsc::channel::<GameEvent>();
    let (npc_event_tx, npc_event_rx) = mpsc::channel::<GameEvent>();
    let (npc_view_tx, npc_view_rx) = mpsc::channel::<HashMap<String, NpcView>>();
    
    // === 啟動執行緒 ===
    let _npc_thread = NpcAiThread::new(npc_view_rx, npc_event_tx);
    let _input_thread = spawn_input_thread(input_tx);
    
    let mut message_queue: Vec<Message> = Vec::new();
    
    loop {
        // 1️⃣ 收集所有事件
        let mut events = Vec::new();
        while let Ok(event) = input_rx.try_recv() {
            events.push(event);
        }
        while let Ok(event) = npc_event_rx.try_recv() {
            events.push(event);
        }
        
        // 2️⃣ 處理事件（單一寫入者）
        for event in events {
            let messages = game_world.apply_event(event);
            message_queue.extend(messages);
        }
        
        // 3️⃣ 更新世界
        game_world.update();
        
        // 4️⃣ 建立 NPC Views (發送給 AI thread)
        let npc_views = game_world.build_npc_views();
        let _ = npc_view_tx.send(npc_views);
        
        // 5️⃣ 建立 RenderState
        let render_state = game_world.build_render_state();
        
        // 6️⃣ Render
        output_manager.render(&render_state, message_queue.drain(..).collect());
        
        if should_exit { break; }
        thread::sleep(Duration::from_millis(16));
    }
    Ok(())
}
```

## 階段四：GameWorld 新增方法 🌍

### 4.1 在 world.rs 新增
```rust
impl GameWorld {
    /// 套用事件（單一寫入者）
    pub fn apply_event(&mut self, event: GameEvent) -> Vec<Message> {
        match event {
            GameEvent::NpcActions { npc_id, actions } => {
                self.apply_npc_actions(npc_id, actions)
            },
            GameEvent::TimerTick { elapsed_secs } => {
                self.apply_timer_tick(elapsed_secs)
            },
            GameEvent::Input(input_event) => {
                self.apply_input(input_event)
            },
        }
    }
    
    /// 套用 NPC 行為
    fn apply_npc_actions(&mut self, npc_id: String, actions: Vec<NpcAction>) -> Vec<Message> {
        let mut messages = Vec::new();
        
        for action in actions {
            match action {
                NpcAction::Say(text) => {
                    messages.push(Message::NpcSay { npc_id: npc_id.clone(), text });
                },
                NpcAction::Move(direction) => {
                    // 執行移動邏輯
                    if let Some(npc) = self.npc_manager.get_npc_mut(&npc_id) {
                        let (dx, dy) = direction.to_delta();
                        let new_x = (npc.x as i32 + dx) as usize;
                        let new_y = (npc.y as i32 + dy) as usize;
                        
                        // 檢查是否可走
                        if let Some(map) = self.get_current_map() {
                            if let Some(point) = map.get_point(new_x, new_y) {
                                if point.walkable {
                                    npc.move_to(new_x, new_y);
                                }
                            }
                        }
                    }
                },
                // ... 其他行為
                _ => {}
            }
        }
        
        messages
    }
    
    /// 建立所有 NPC 的視圖
    pub fn build_npc_views(&self) -> HashMap<String, NpcView> {
        let mut views = HashMap::new();
        
        for (npc_id, npc) in self.npc_manager.npcs.iter() {
            let view = NpcView {
                self_id: npc_id.clone(),
                self_pos: Position { x: npc.x, y: npc.y },
                current_map: npc.map.clone(),
                time: self.get_time_info().into(),
                nearby_entities: self.get_nearby_entities(npc.x, npc.y, 5),
                visible_items: self.get_visible_items(npc.x, npc.y),
                terrain: self.get_terrain_info(npc.x, npc.y),
            };
            views.insert(npc_id.clone(), view);
        }
        
        views
    }
    
    /// 建立渲染狀態
    pub fn build_render_state(&self) -> RenderState {
        RenderState {
            player_pos: Position { x: self.player.x, y: self.player.y },
            current_map: self.current_map_name.clone(),
            time: self.format_time(),
            // ... 其他渲染資訊
        }
    }
}
```

## 階段五：測試與驗證 ✅

### 5.1 檢查清單
- [ ] GameWorld 無 Arc/Mutex
- [ ] NPC AI 只回傳 Action，不修改狀態
- [ ] 所有執行緒透過 channel 通訊
- [ ] OutputManager 只接收不可變資料
- [ ] 事件處理在主執行緒

### 5.2 效能測試
- [ ] 移除頻繁的 clone()
- [ ] 測量 channel 延遲
- [ ] 確認無死鎖

## 未來擴充 🚀

1. **ECS 架構準備**：將 Person 拆分為 Component
2. **LLM NPC**：NpcView → Prompt 生成
3. **Replay System**：記錄所有 GameEvent
4. **網路多人**：Event 可序列化

---

## 重構優先順序

1. ⭐⭐⭐ **先做階段一**：定義資料結構（不影響現有系統）
2. ⭐⭐ **再做階段二**：逐步替換 NPC AI（可並行測試）
3. ⭐ **最後做階段三**：整合主迴圈（一次性切換）

## 風險評估

- **高風險**：主迴圈改動（建議最後做）
- **中風險**：NPC AI 重構（可漸進式）
- **低風險**：新增資料結構（不影響現有系統）

## 相容性策略

在重構期間，可以**暫時保留舊的 Arc<Mutex> 路徑**，用 feature flag 切換：

```rust
#[cfg(feature = "new-architecture")]
let npc_thread = create_event_based_thread();

#[cfg(not(feature = "new-architecture"))]
let npc_thread = create_mutex_based_thread(); // 目前的實作
```

完成重構後移除舊代碼。
