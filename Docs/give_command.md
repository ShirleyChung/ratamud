# Give 命令 - 給予物品給 NPC

## 功能說明

新增 `give` 命令，允許玩家將持有的物品給予 NPC，增加互動性和好感度系統。

## 使用方式

### 基本語法

```
give <npc> <item> [quantity]
```

### 參數說明

- `<npc>` - NPC 名稱（必須）
- `<item>` - 物品名稱（必須）
- `[quantity]` - 數量（可選，預設為 1）

## 使用範例

### 範例 1: 給予單個物品

```
> give 商人 蘋果
🎁 你給了 商人 1 個 蘋果
💖 商人 對你的好感度增加了！(現在: 5)
```

### 範例 2: 給予多個物品

```
> give 村長 麵包 5
🎁 你給了 村長 5 個 麵包
💖 村長 對你的好感度增加了！(現在: 10)
```

### 範例 3: 給予稀有物品

```
> give sakura 鑽石戒指
🎁 你給了 sakura 1 個 鑽石戒指
💖 sakura 對你的好感度增加了！(現在: 15)
```

## 功能特色

### 1. 物品轉移
- 從玩家背包移除物品
- 物品加入 NPC 背包
- 自動驗證數量是否足夠

### 2. 好感度系統
- 每次給予物品增加 5 點好感度
- 好感度上限為 100
- 顯示當前好感度數值

### 3. 驗證機制
- 檢查 NPC 是否在同一位置
- 檢查玩家是否擁有該物品
- 檢查物品數量是否足夠

### 4. 自動保存
- 自動保存玩家狀態
- 自動保存所有 NPC 狀態

## 錯誤處理

### 錯誤 1: NPC 不在附近

```
> give 遠方的NPC 蘋果
錯誤: 此處找不到 遠方的NPC
```

### 錯誤 2: 沒有該物品

```
> give 商人 黃金
錯誤: 你沒有 黃金
```

### 錯誤 3: 數量不足

```
> give 村長 蘋果 10
錯誤: 你只有 3 個 蘋果，不足 10 個
```

## 使用場景

### 場景 1: 送禮增加好感度

```
> status
背包: 蘋果 x5

> give sakura 蘋果 3
🎁 你給了 sakura 3 個 蘋果
💖 sakura 對你的好感度增加了！(現在: 5)

> status
背包: 蘋果 x2
```

### 場景 2: 完成任務交付物品

```
> quest info 尋找草藥
任務目標: 收集 5 個草藥並交給村長

> give 村長 草藥 5
🎁 你給了 村長 5 個 草藥
💖 村長 對你的好感度增加了！(現在: 10)

（任務系統可以檢測到交付並完成任務）
```

### 場景 3: 與商人交易準備

```
> give 商人 珍珠 2
🎁 你給了 商人 2 個 珍珠
💖 商人 對你的好感度增加了！(現在: 15)

（商人好感度提升後可能降低價格或提供特殊商品）
```

## 技術實作

### 程式碼結構

```rust
fn handle_give(
    npc_name: String,
    item_name: String,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>>
```

### 執行流程

1. **驗證 NPC 位置**
   ```rust
   let npcs_here = game_world.npc_manager.get_npcs_at_in_map(...);
   if !npc_found {
       return Error("此處找不到 NPC");
   }
   ```

2. **驗證物品擁有**
   ```rust
   if !me.items.contains_key(&resolved_item) {
       return Error("你沒有該物品");
   }
   ```

3. **驗證數量**
   ```rust
   let player_quantity = me.get_item_count(&resolved_item);
   if player_quantity < quantity {
       return Error("數量不足");
   }
   ```

4. **轉移物品**
   ```rust
   me.drop_items(&resolved_item, quantity);
   npc.add_items(resolved_item, quantity);
   ```

5. **增加好感度**
   ```rust
   npc.relationship = (npc.relationship + 5).min(100);
   ```

6. **保存狀態**
   ```rust
   me.save(...);
   game_world.npc_manager.save_all(...);
   ```

## 與其他系統的整合

### 1. 交易系統
- `buy` - 從 NPC 購買物品（需支付金錢）
- `sell` - 向 NPC 出售物品（獲得金錢）
- `give` - 給予 NPC 物品（免費，增加好感度）

### 2. 任務系統
- 任務可能要求給予 NPC 特定物品
- 完成給予後觸發任務進度
- 任務獎勵可能包含好感度提升

### 3. 對話系統
- 高好感度可能解鎖特殊對話
- NPC 可能根據好感度給予不同回應
- 某些對話選項需要特定好感度

### 4. NPC AI 系統
- 高好感度的 NPC 可能主動幫助玩家
- NPC 可能記住玩家給予的物品
- 影響 NPC 對玩家的行為模式

## 好感度系統

### 當前實作
- 每次給予物品：+5 好感度
- 好感度範圍：-100 到 100
- 上限：100（不會超過）

### 未來擴展可能性

1. **物品價值影響**
   ```rust
   let value = get_item_value(&item);
   let relationship_gain = (value / 10).max(1).min(20);
   ```

2. **好感度等級**
   - -100 ~ -50: 仇敵
   - -50 ~ 0: 不友善
   - 0 ~ 25: 陌生
   - 25 ~ 50: 友好
   - 50 ~ 75: 好友
   - 75 ~ 100: 摯友

3. **特殊物品加成**
   - 最喜愛的物品：+20 好感度
   - 討厭的物品：-10 好感度
   - 特殊紀念品：+50 好感度

4. **好感度衰減**
   - 長時間不互動可能降低好感度
   - 鼓勵玩家持續與 NPC 互動

## 相關命令

- `trade <npc>` - 查看 NPC 商品
- `buy <npc> <item> [quantity]` - 購買物品
- `sell <npc> <item> [quantity]` - 出售物品
- `talk <npc> [topic]` - 與 NPC 對話
- `check <npc>` - 查看 NPC 詳細資訊（包括好感度）

## 測試建議

### 測試案例 1: 基本功能
```
1. 撿起一些物品
2. 找到一個 NPC
3. 使用 give 命令給予物品
4. 驗證物品已轉移
5. 驗證好感度增加
```

### 測試案例 2: 錯誤處理
```
1. 嘗試給予不存在的物品
2. 嘗試給予超過擁有數量的物品
3. 嘗試給予不在附近的 NPC
4. 驗證錯誤訊息正確顯示
```

### 測試案例 3: 好感度上限
```
1. 連續給予物品 20 次以上
2. 驗證好感度不會超過 100
```

## 總結

`give` 命令為遊戲增加了：
- ✅ 玩家與 NPC 的互動深度
- ✅ 好感度系統的基礎
- ✅ 任務交付的可能性
- ✅ 社交玩法的擴展性

這個功能為未來的 NPC 關係系統、任務系統和社交玩法奠定了基礎。

---

**開發日期**: 2025-12-25  
**版本**: 1.0  
**狀態**: ✅ 已實作並可用
