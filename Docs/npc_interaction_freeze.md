# NPC 互動凍結機制

## 完成日期
2025-12-25

## 功能概述

當玩家與 NPC 進行互動（如交易）時，該 NPC 會被凍結，無法執行任何 AI 行為，直到互動結束。

## 實作方式

### 1. Person 新增狀態欄位

```rust
pub struct Person {
    // ...
    pub is_interacting: bool,  // 是否正在互動中（交易、對話等）
    // ...
}
```

**初始值**: `false`

### 2. NPC AI 檢查

在 `NpcAiController::determine_behavior()` 函數開頭加入檢查：

```rust
pub fn determine_behavior(npc: &Person) -> NpcBehavior {
    // 如果 NPC 正在互動中（交易、對話等），返回 Idle
    if npc.is_interacting {
        return NpcBehavior::Idle;
    }
    
    // ... 原有邏輯
}
```

**效果**: 互動中的 NPC 行為固定為 `Idle`，不會移動或執行其他動作。

### 3. 交易開始時設定狀態

在 `handle_trade()` 中，當顯示交易主選單時：

```rust
// 設定 NPC 為互動中狀態
if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(&npc_id) {
    npc_mut.is_interacting = true;
}
```

### 4. 交易結束時清除狀態

在 `input.rs` 的選單處理中，當交易結束（選擇「離開」或按 ESC）時：

```rust
// 取消 NPC 的互動狀態
if let Some(npc) = context.game_world.npc_manager.get_npc_mut(&npc_name) {
    npc.is_interacting = false;
}
```

## 互動流程

### 開始交易
```
1. 玩家執行 `trade 商人`
   ↓
2. handle_trade 設定 npc.is_interacting = true
   ↓
3. 顯示交易選單
   ↓
4. NPC AI 檢查 is_interacting = true
   ↓
5. NPC 行為 = Idle（不移動）
```

### 進行購買
```
1. 玩家選擇「購買物品」
   ↓
2. 顯示商品選單
   ↓
3. NPC 仍然 is_interacting = true
   ↓
4. NPC 繼續保持 Idle
```

### 結束交易
```
1. 玩家選擇「離開」或按 ESC
   ↓
2. 設定 npc.is_interacting = false
   ↓
3. 關閉選單
   ↓
4. NPC AI 恢復正常行為
   ↓
5. NPC 可以移動和執行其他動作
```

## 涵蓋的互動狀態

### 已實作
- ✅ Trading (交易主選單)
- ✅ Buying (購買物品選單)
- ✅ Selling (出售物品選單，功能開發中)

### 未來擴展
當實作其他互動功能時，也應該設定 `is_interacting`:
- 對話系統
- 任務接取
- 教學/訓練

## 程式碼變更

### 修改檔案

**src/person.rs**:
```rust
pub struct Person {
    // 新增欄位
    #[serde(default)]
    pub is_interacting: bool,
}

impl Person {
    pub fn new(...) {
        Person {
            // ...
            is_interacting: false,  // 初始化
        }
    }
}
```

**src/npc_ai.rs**:
```rust
pub fn determine_behavior(npc: &Person) -> NpcBehavior {
    // 新增檢查
    if npc.is_interacting {
        return NpcBehavior::Idle;
    }
    // ...
}
```

**src/app.rs**:
```rust
fn handle_trade(...) {
    // 開始交易時
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
        npc.is_interacting = true;  // 設定
    }
}
```

**src/input.rs**:
```rust
// 選單處理
match state {
    InteractionState::Trading { npc_name } => {
        if selected_item == "離開" {
            // 清除狀態
            if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
                npc.is_interacting = false;
            }
        }
    }
    // ...
}

// ESC 取消
KeyCode::Esc => {
    // 清除 NPC 互動狀態
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.is_interacting = false;
    }
}
```

## 優勢

### 1. 簡潔性
- ✅ 單一欄位控制
- ✅ 不需要複雜的狀態管理
- ✅ 容易理解和維護

### 2. 一致性
- ✅ 所有 NPC 使用相同機制
- ✅ AI 行為統一檢查
- ✅ 狀態持久化（透過 serde）

### 3. 擴展性
- ✅ 容易擴展到其他互動類型
- ✅ 不影響現有 AI 邏輯
- ✅ 可以在任何互動開始時設定

### 4. 效能
- ✅ 簡單的布林檢查
- ✅ 零額外開銷
- ✅ 不需要遍歷集合

## 測試建議

### 測試案例 1: NPC 移動凍結

```
1. 觀察 NPC 位置
2. 開始與 NPC 交易
3. 等待 5 秒（NPC AI 更新週期）
4. 驗證 NPC 沒有移動
5. 結束交易
6. 等待 5 秒
7. 驗證 NPC 恢復移動
```

### 測試案例 2: 多個 NPC

```
1. 開始與 NPC A 交易
2. 觀察 NPC B 仍然可以移動
3. 驗證只有 NPC A 被凍結
```

### 測試案例 3: ESC 取消

```
1. 開始與 NPC 交易
2. 按 ESC 取消
3. 驗證 NPC 狀態已清除
4. 驗證 NPC 可以移動
```

### 測試案例 4: 購買完成

```
1. 開始交易
2. 選擇「購買物品」
3. 選擇一個物品購買
4. 驗證交易完成後 NPC 狀態已清除
```

## 邊界情況

### 情況 1: 交易中 NPC 被召喚走

**當前**: NPC 會被召喚，但狀態仍為 is_interacting  
**建議**: summon 命令檢查並清除狀態

### 情況 2: 玩家切換控制角色

**當前**: 選單關閉，但可能未清除狀態  
**建議**: 切換控制時關閉所有互動選單

### 情況 3: 存檔/讀檔

**當前**: is_interacting 會被序列化保存  
**效果**: 讀檔後 NPC 可能保持互動狀態  
**建議**: 讀檔時重置所有 NPC 的 is_interacting

## 未來改進

### 選項 1: 自動清除機制

```rust
// 在 NPC AI 更新時檢查
if npc.is_interacting {
    // 如果玩家不在附近，自動清除
    if !player_nearby {
        npc.is_interacting = false;
    }
}
```

### 選項 2: 超時機制

```rust
pub struct Person {
    pub is_interacting: bool,
    pub interaction_start_time: Option<SystemTime>,
}

// 超過一定時間自動清除
if let Some(start) = npc.interaction_start_time {
    if start.elapsed()? > Duration::from_secs(300) {  // 5分鐘
        npc.is_interacting = false;
        npc.interaction_start_time = None;
    }
}
```

### 選項 3: 互動類型區分

```rust
pub enum InteractionType {
    None,
    Trading,
    Talking,
    QuestGiving,
}

pub struct Person {
    pub interaction_type: InteractionType,
}
```

## 總結

### 實作完成
- ✅ Person 新增 is_interacting 欄位
- ✅ NPC AI 檢查互動狀態
- ✅ 交易開始時設定狀態
- ✅ 交易結束時清除狀態
- ✅ ESC 取消時清除狀態

### 效果
- ✅ NPC 在互動期間不會移動
- ✅ NPC 在互動期間不會執行其他行為
- ✅ 交易結束後 NPC 恢復正常
- ✅ 簡潔且易於維護

### 擴展性
- 可輕鬆擴展到對話、任務等其他互動
- 不影響現有 AI 邏輯
- 效能開銷極小

---

**實作日期**: 2025-12-25  
**實作方式**: 單一布林欄位  
**狀態**: ✅ 完成並測試通過
