# 互動式選單交易系統

## 完成日期
2025-12-25

## 功能概述

實作基於選單的 NPC 互動系統，特別是交易功能。當玩家與 NPC 交易時，使用專用的互動選單，在完成決定之前，NPC 和玩家都無法執行其他動作（如移動）。

## 主要特色

### 1. 獨立的互動選單系統

創建了專門的 `interaction_menu`，與 ESC 選單 (`menu`) 分離：

```rust
pub struct AppContext<'a> {
    pub menu: &'a mut Option<Menu>,               // ESC 選單（遊戲主選單）
    pub interaction_menu: &'a mut Option<Menu>,  // 互動選單（交易、對話等）
    // ...
}
```

**優勢**:
- 不會與 ESC 選單衝突
- 專門用於 NPC 互動
- 可獨立控制行為

### 2. 互動狀態追蹤

新增 `InteractionState` 枚舉追蹤玩家當前的互動狀態：

```rust
pub enum InteractionState {
    None,                              // 無互動
    Trading { npc_name: String },     // 交易主選單（買/賣選擇）
    Buying { npc_name: String },      // 購買物品選單
    Selling { npc_name: String },     // 出售物品選單
}
```

加入到 `GameWorld`:
```rust
pub struct GameWorld {
    // ...
    pub interaction_state: InteractionState,
}
```

### 3. 交易流程

#### 流程 1: 開始交易
```
> trade 商人
  ↓
打開交易主選單：
┌─────────────────┐
│ 與 商人 交易     │
├─────────────────┤
│ ▸ 購買物品       │
│   出售物品       │
│   離開           │
└─────────────────┘
```

#### 流程 2: 選擇購買
```
選擇「購買物品」
  ↓
打開購買物品選單：
┌────────────────────────────┐
│ 購買物品 - 商人             │
├────────────────────────────┤
│ ▸ 蘋果 x10 - 5 金幣         │
│   麵包 x20 - 10 金幣        │
│   治療藥水 x5 - 50 金幣     │
│   返回                      │
└────────────────────────────┘
```

#### 流程 3: 確認購買
```
選擇「蘋果 x10 - 5 金幣」
  ↓
執行 buy 命令
  ↓
關閉選單，完成交易
```

#### 流程 4: 取消交易
```
按 ESC
  ↓
關閉選單
  ↓
InteractionState = None
```

## 技術實作

### 1. 輸入處理優先級

```rust
pub fn handle_input_events(...) {
    // 優先處理互動選單
    if let Some(interaction_menu) = context.interaction_menu {
        // 處理互動選單輸入
        // ↑↓ 移動選項
        // Enter 確認
        // ESC 取消
        return None; // 消耗輸入，不往下傳
    }
    
    // 然後處理 ESC 選單
    if let Some(menu) = context.menu {
        // 處理 ESC 選單輸入
        return None;
    }
    
    // 最後處理一般輸入
    // ...
}
```

### 2. 選單渲染

在 `draw_ui()` 中分別渲染兩種選單：

```rust
// 先渲染 ESC 選單
if let Some(active_menu) = menu {
    if active_menu.active {
        // 渲染...
    }
}

// 再渲染互動選單（覆蓋在上面）
if let Some(active_interaction_menu) = interaction_menu {
    if active_interaction_menu.active {
        // 渲染（稍大一點）...
    }
}
```

### 3. trade 命令重構

**修改前**:
```rust
fn handle_trade(...) {
    // 印出商品列表
    for (item, quantity, price) in goods {
        println!("{item} x{quantity} - {price}");
    }
    println!("使用 buy <npc> <item> 購買");
}
```

**修改後**:
```rust
fn handle_trade(..., interaction_menu: &mut Option<Menu>) {
    match &game_world.interaction_state {
        InteractionState::Buying { .. } => {
            // 打開購買選單
            let mut items = vec![];
            for (item, quantity, price) in goods {
                items.push(format!("{item} x{quantity} - {price} 金幣"));
            }
            let menu = Menu::new("購買物品", items);
            *interaction_menu = Some(menu);
        },
        _ => {
            // 打開交易主選單
            let menu = Menu::new("交易", vec![
                "購買物品", "出售物品", "離開"
            ]);
            *interaction_menu = Some(menu);
        }
    }
}
```

## 使用方式

### 開始交易

```bash
> trade 商人
# 或
> trade merchant
```

**效果**:
- 打開交易主選單
- 設定 `InteractionState::Trading`
- 顯示「購買物品」、「出售物品」、「離開」選項

### 選擇購買

**操作**:
- ↑↓ 鍵選擇「購買物品」
- Enter 確認

**效果**:
- 設定 `InteractionState::Buying`
- 打開購買物品選單
- 列出所有可購買的物品

### 選擇物品

**操作**:
- ↑↓ 鍵選擇物品
- Enter 確認

**效果**:
- 執行 `buy` 命令
- 扣除金幣
- 獲得物品
- 關閉選單

### 取消交易

**操作**:
- 按 ESC

**效果**:
- 立即關閉選單
- 設定 `InteractionState::None`
- 顯示「取消交易」訊息

## 阻止動作機制

### 當前實作

當 `interaction_menu` 開啟時，所有輸入都被攔截：

```rust
if let Some(interaction_menu) = context.interaction_menu {
    // 處理選單輸入
    // ...
    return None; // ← 阻止輸入往下傳遞
}
```

**結果**:
- ✅ 無法移動（箭頭鍵被選單使用）
- ✅ 無法執行命令（輸入被攔截）
- ✅ NPC AI 仍在運行（可擴展為暫停）

### 未來擴展（待實作）

#### 選項 1: 暫停 NPC AI

```rust
if interaction_menu.is_some() {
    game_world.npc_ai_thread.pause();
}
```

#### 選項 2: 時間凍結

```rust
if interaction_menu.is_some() {
    game_world.time_thread.pause();
}
```

## 指令變更

### trade 命令

**原本**:
```bash
> trade 商人
═══ 商人 的商品 ═══
  蘋果 x10 - 5 金幣
  麵包 x20 - 10 金幣
使用 buy <npc> <item> 購買物品
```

**現在**:
```bash
> trade 商人
開始與 商人 交易

┌─────────────────┐
│ 與 商人 交易     │
├─────────────────┤
│ ▸ 購買物品       │
│   出售物品       │
│   離開           │
└─────────────────┘
```

### buy 和 sell 命令

**原本行為**:
- 直接輸入命令購買/出售
- 需要記住物品名稱

**現在建議使用方式**:
1. `trade 商人` 打開選單
2. 選擇「購買物品」
3. 從選單選擇物品

**直接命令仍可用**:
```bash
> buy 商人 蘋果 5
# 仍然有效，繞過選單直接購買
```

## 程式碼變更清單

### 新增檔案
- 無（使用現有結構）

### 修改檔案

#### src/world.rs
```rust
// 新增
pub enum InteractionState { ... }

pub struct GameWorld {
    // 新增
    pub interaction_state: InteractionState,
}
```

#### src/app.rs
```rust
pub struct AppContext<'a> {
    // 新增
    pub interaction_menu: &'a mut Option<Menu>,
}

fn run_main_loop(...) {
    // 新增
    let mut interaction_menu: Option<Menu> = None;
}

fn draw_ui(..., interaction_menu: &Option<Menu>) {
    // 新增渲染互動選單
}

fn handle_command_result(..., interaction_menu: &mut Option<Menu>) {
    // 新增參數
}

fn handle_trade(..., interaction_menu: &mut Option<Menu>) {
    // 重構為選單模式
}
```

#### src/input.rs
```rust
pub fn handle_input_events(...) {
    // 新增互動選單處理（優先級最高）
    if let Some(interaction_menu) = context.interaction_menu {
        // 處理互動選單輸入
    }
}
```

## 測試建議

### 測試案例 1: 基本交易流程

```bash
1. trade 商人
2. ↓ 選擇「購買物品」
3. Enter 確認
4. ↓ 選擇物品
5. Enter 購買
6. 驗證物品已加到背包
```

### 測試案例 2: 取消交易

```bash
1. trade 商人
2. ESC
3. 驗證選單已關閉
4. 驗證可以正常移動
```

### 測試案例 3: 巢狀選單

```bash
1. trade 商人（主選單）
2. Enter「購買物品」（子選單）
3. 選擇「返回」
4. 驗證回到主選單
```

### 測試案例 4: 無法移動

```bash
1. trade 商人
2. 嘗試按箭頭鍵移動
3. 驗證只是選單選項移動，角色不移動
```

## 已知限制

1. **NPC AI 未暫停**: 
   - NPC 仍會移動
   - 可能離開交易範圍
   - 待實作 AI 暫停

2. **出售功能未完成**:
   - 選單已顯示「出售物品」
   - 實際功能待實作

3. **時間未凍結**:
   - 遊戲時間仍在流動
   - 可擴展為暫停時間

## 未來擴展方向

### 1. 對話系統
```rust
InteractionState::Talking { npc_name, topic }
```

### 2. 任務接取選單
```rust
InteractionState::QuestMenu { npc_name }
```

### 3. 製作/合成選單
```rust
InteractionState::Crafting { station }
```

### 4. 數量選擇
```
選擇物品後，打開數量選單：
┌──────────────┐
│ 購買數量      │
├──────────────┤
│ ▸ 1          │
│   5          │
│   10         │
│   全部        │
└──────────────┘
```

### 5. 確認對話框
```
確認購買：
┌──────────────────────┐
│ 購買 5 個蘋果？       │
│ 花費：25 金幣         │
├──────────────────────┤
│ ▸ 確認               │
│   取消               │
└──────────────────────┘
```

## 總結

### 完成功能
- ✅ 獨立互動選單系統
- ✅ 交易主選單（買/賣選擇）
- ✅ 購買物品選單
- ✅ ESC 取消功能
- ✅ 阻止玩家移動（選單優先）
- ✅ 互動狀態追蹤

### 待完成功能
- ⏳ 出售物品選單
- ⏳ NPC AI 暫停
- ⏳ 時間凍結
- ⏳ 數量選擇
- ⏳ 確認對話框

### 架構優勢
- 清晰的狀態管理
- 獨立的選單系統
- 易於擴展
- 符合使用者預期

---

**開發日期**: 2025-12-25  
**版本**: 1.0  
**狀態**: ✅ 基本功能完成
