# NPC AI 與交易系統實現

## 已完成的功能

### 1. 地形系統 (map.rs)

新增 `TerrainType` 枚舉：
- `Normal` - 普通地形
- `Farmland` - 農地
- `Road` - 道路
- `Shop` - 商店
- `House` - 房屋
- `Water` - 水域

每個 `Point` 現在都有 `terrain_type` 欄位。

### 2. NPC AI 系統 (npc_ai.rs)

**NPC 行為類型**：
- `Idle` - 閒置
- `Wander` - 漫遊
- `PickupItems` - 撿拾物品
- `UseFood` - 使用食物（當 HP < 50）
- `Farm` - 耕作（農夫專屬）
- `Trade` - 交易（商人專屬，被動）

**NpcAiController**：
- `update_all_npcs()` - 更新所有 NPC 的 AI
- 根據 NPC 描述自動判斷職業
- 自動執行相應行為

**農夫 AI**：
- 自動將周圍 3x3 範圍的普通地形轉換為農地
- 修改地形描述為"肥沃的農地"
- 自動保存地圖變更

**基礎 AI**：
- 自動行走（隨機方向）
- 自動撿拾地面物品
- HP 低時自動使用食物

### 3. 交易系統 (trade.rs)

**TradeSystem**：
- `buy_from_npc()` - 玩家從 NPC 購買物品
- `sell_to_npc()` - 玩家向 NPC 出售物品
- `get_item_base_price()` - 獲取物品基礎價格
- `calculate_buy_price()` - 計算購買價（基礎價 × 1.5）
- `calculate_sell_price()` - 計算出售價（基礎價 × 0.7）
- `get_npc_goods()` - 獲取 NPC 的商品列表

**價格表**：
- 蘋果: 5 金幣
- 乾肉: 10 金幣
- 麵包: 8 金幣
- 治療藥水: 50 金幣
- 木劍: 30 金幣
- 石子: 1 金幣
- 魔法書: 100 金幣

## 下一步整合

### 需要在 app.rs 添加：

1. **在事件檢查循環中調用 NPC AI**：
```rust
// 在 check_and_execute_events 或主循環中
let ai_logs = npc_ai::NpcAiController::update_all_npcs(game_world);
for log in ai_logs {
    output_manager.log(log);
}
```

2. **添加交易指令**：
- `trade <npc>` - 顯示 NPC 商品列表
- `buy <npc> <item> [quantity]` - 購買物品
- `sell <npc> <item> [quantity]` - 出售物品

3. **在 input.rs 添加命令**：
```rust
CommandResult::Trade(String),                // 查看商品
CommandResult::Buy(String, String, u32),     // 購買
CommandResult::Sell(String, String, u32),    // 出售
```

## 測試場景

### 農夫測試
```bash
create npc 農夫 farmer1
ctrl farmer1
# 等待 AI 自動開墾農地
look
# 應該看到周圍變成農地
```

### 交易測試
```bash
# 給商人一些商品和金幣
create npc 商人 merchant
# 給玩家金幣
# 使用 trade 指令查看和交易
```

### NPC 智能測試
```bash
# 創建 NPC 並放置食物
create npc 工人 worker1
drop apple 5
# 觀察 NPC 是否會撿起物品
# 降低 NPC HP，觀察是否會使用食物
```

## 文件清單

- ✅ src/map.rs - 添加 TerrainType
- ✅ src/npc_ai.rs - NPC AI 系統
- ✅ src/trade.rs - 交易系統
- ✅ src/npc_manager.rs - 添加 get_all_npc_ids()
- ✅ src/lib.rs - 添加模組聲明
- ✅ src/main.rs - 添加模組聲明
- ⏳ src/app.rs - 需要整合 AI 更新和交易指令
- ⏳ src/input.rs - 需要添加交易指令解析

## 編譯狀態

✅ 編譯成功，有一些未使用函數的警告（正常）

## ✅ 整合完成！

### 已添加功能

1. **Person 添加 max_hp**
   - 新增 `max_hp` 欄位
   - NPC AI 使用 `max_hp / 2` 判斷是否需要使用食物
   - 恢復 HP 時限制在 max_hp 範圍內

2. **交易指令完整實現**
   - `trade <npc>` - 查看 NPC 商品列表和價格
   - `buy <npc> <item> [數量]` - 購買物品
   - `sell <npc> <item> [數量]` - 出售物品
   - 自動計算價格（買入1.5倍、賣出0.7倍）
   - 自動檢查金幣和物品數量
   - 交易成功後自動保存

3. **NPC AI 自動更新**
   - 每遊戲分鐘自動執行一次
   - 整合到事件檢查循環中
   - 所有 AI 行為日誌會顯示在系統日誌中

4. **金幣系統**
   - 添加金幣到物品註冊表
   - 支持 `gold` 和 `coin` 別名

### 使用範例

#### 測試交易系統

```bash
# 1. 給商人一些商品和金幣
create npc 商人 merchant
set merchant hp 100
ctrl merchant
# 給商人物品（切換到商人後操作）
get apple 10  # 假設地面有蘋果
# 給商人金幣
create item 金幣 gold
get gold 1000

# 2. 切換回玩家
ctrl me

# 3. 給玩家金幣
create item 金幣
drop gold 100
get gold 100

# 4. 召喚商人到身邊
summon merchant

# 5. 查看商品
trade merchant

# 6. 購買物品
buy merchant apple 5

# 7. 查看狀態確認
status

# 8. 出售物品
sell merchant apple 2
```

#### 測試農夫 AI

```bash
# 創建農夫
create npc 農夫 farmer1

# 等待幾分鐘，觀察系統日誌
# 應該會看到「農夫 正在開墾農地」

# 檢查周圍地形
flyto <farmer1的位置>
look
# 應該看到「肥沃的農地」
```

#### 測試 NPC 自動行為

```bash
# 創建 NPC
create npc 工人 worker1

# 降低 HP 測試自動使用食物
set worker1 hp 40

# 給 NPC 食物
ctrl worker1
drop apple 5
get apple 5

# 等待一分鐘
# 應該在系統日誌看到「工人 使用了 蘋果 恢復 HP」

# 測試自動撿拾
ctrl me
drop apple 10
# 等待 NPC 路過
# 可能會看到「工人 撿起了 蘋果」
```

### 系統日誌示例

```
[14:45:10] 商人 正在開墾農地
[14:45:20] 工人 撿起了 蘋果
[14:45:30] 農夫 使用了 乾肉 恢復 HP
```

### 指令幫助

新的交易分類：

```
💰 交易
  trade <npc>              - 查看NPC商品
  buy <npc> <item> [數量]  - 購買物品
  sell <npc> <item> [數量] - 出售物品
```

## 技術細節

### NPC AI 更新頻率

- 每遊戲分鐘執行一次（約每真實秒）
- 與事件系統整合在同一個檢查循環
- 不會影響遊戲性能

### 交易價格計算

- 基礎價格在 `trade.rs` 定義
- 購買價 = 基礎價 × 1.5
- 出售價 = 基礎價 × 0.7
- 確保商人有利潤空間

### AI 行為優先級

1. HP < max_hp/2 → 使用食物
2. 根據職業 → 農夫耕作、商人待命
3. 其他 NPC → 30%撿拾、30%漫遊、40%閒置

## 編譯狀態

✅ **編譯成功！**
✅ **所有功能已整合**
✅ **可以開始測試**
