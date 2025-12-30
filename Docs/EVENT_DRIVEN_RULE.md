# GameWorld 多執行緒 NPC / Render 架構規格

> 目標：
>
> * NPC 可根據世界狀態自行判斷並產生行為（包含說話）
> * GameWorld 為唯一可變狀態擁有者
> * NPC / Timer / Input 可使用 thread
> * OutputManager 僅負責 render（crossterm）
> * 避免 Arc<Mutex<GameWorld>> 與共享可變狀態

---

## 一、核心設計原則（必讀）

1. **GameWorld 單一寫入者（Single Writer）**

   * 只有 main thread 可以修改 GameWorld
   * GameWorld 不使用 Mutex / Arc

2. **Thread 只產生事件（Event / Action）**

   * NPC / Timer / Input thread 不可直接存取 GameWorld
   * Thread 與 GameWorld 只透過 channel 溝通

3. **NPC 不直接輸出（No Print）**

   * NPC 只能「決定意圖」
   * 輸出一律由 OutputManager 處理

4. **Render 使用不可變快照（Snapshot Rendering）**

   * RenderState 為不可變資料
   * Render 不持有 GameWorld reference

---

## 二、整體架構總覽

```
NPC Thread ─┐
Timer Thread ─┼──► Event Channel ───► GameWorld (main loop)
Input Thread ─┘                         │
                                         ├─ 更新世界狀態
                                         ├─ 產生 Message
                                         └─ 建立 RenderState ─► OutputManager ─► Crossterm
```

---

## 三、GameWorld 設計

### 職責

* 擁有所有遊戲狀態
* 套用事件（Event / Action）
* 產生 RenderState 與 Message

### 結構範例

```rust
struct GameWorld {
    npc_manager: NpcManager,
    timer: TimerState,
    output_queue: Vec<Message>,
}
```

### 原則

* 不放 Mutex
* 不跨 thread 使用
* 所有修改透過 `&mut self`

---

## 四、NPC 設計

### NPC 的責任

* 根據「可見世界」判斷行為
* 回傳 Action（意圖）

### NPC 不可以做的事

* ❌ 不可存取 GameWorld
* ❌ 不可 print
* ❌ 不可持有 OutputManager

---

## 五、NpcView（NPC 世界快照）

### 目的

* 提供 NPC 判斷所需資訊
* 作為 GameWorld 與 NPC 的邊界

### 設計原則

* 不可變（immutable）
* Owned data（可跨 thread）
* 僅包含 NPC 被允許看到的資訊

### 範例

```rust
struct NpcView {
    self_pos: Position,
    nearby_entities: Vec<EntityInfo>,
    time: GameTime,
}
```

---

## 六、NpcAction（NPC 意圖）

### 定義

```rust
enum NpcAction {
    Say(String),
    Move(Direction),
    Attack(EntityId),
}
```

### 說明

* Action = NPC 想做什麼
* 是否成功由 GameWorld 判斷

---

## 七、Event 系統

### Event 定義

```rust
enum Event {
    NpcActions(NpcId, Vec<NpcAction>),
    TimerTick,
    Input(InputEvent),
}
```

### 原則

* 所有跨 thread 資訊都包成 Event
* Event 為 owned data

---

## 八、Game Loop（Main Thread）

### 基本流程

```rust
loop {
    // 1. 收事件
    while let Ok(event) = rx.try_recv() {
        world.apply_event(event);
    }

    // 2. 更新世界（tick）
    world.update();

    // 3. 建立 RenderState
    let render_state = world.build_render_state();

    // 4. Render
    output_manager.render(&render_state, world.drain_messages());
}
```

---

## 九、Message（輸出訊息）

### 定義

```rust
enum Message {
    NpcSay { npc_id: NpcId, text: String },
    System(String),
}
```

### 說明

* Message 為 GameWorld 的輸出
* OutputManager 只處理 Message + RenderState

---

## 十、OutputManager

### 職責

* 將 RenderState + Message render 到 terminal
* 不包含任何遊戲邏輯

### 範例

```rust
struct OutputManager;

impl OutputManager {
    fn render(&self, state: &RenderState, messages: Vec<Message>) {
        // crossterm render
    }
}
```

---

## 十一、RenderState

### 原則

* 不可變
* 可 clone
* 不持有 GameWorld reference

### 範例

```rust
#[derive(Clone)]
struct RenderState {
    entities: Vec<Drawable>,
    ui: UiState,
}
```

---

## 十二、多執行緒 NPC（可選）

### 流程

1. GameWorld 建立 NpcView
2. 傳送至 NPC thread
3. NPC thread 回傳 NpcAction
4. GameWorld 套用 Action

### Thread 僅持有

* channel Sender / Receiver
* NPC 本身（AI logic）

---

## 十三、為什麼不使用 Arc<Mutex<GameWorld>>

* 避免巨大鎖
* 避免 render / AI 互相阻塞
* 符合 Rust ownership flow
* 易於測試與重構

---

## 十四、設計哲學總結

> NPC 是「演員」
> GameWorld 是「導演」
> OutputManager 是「舞台與燈光」

* 演員不能改劇本
* 演員不能自己上燈
* 一切行為由導演裁決

---

## 十五、未來擴充方向（預留）

* LLM NPC（NpcView → prompt）
* Script / Behavior Tree
* Replay / Event Log
* ECS 架構

---

**本文件可作為重構與未來功能擴充的設計依據。**
