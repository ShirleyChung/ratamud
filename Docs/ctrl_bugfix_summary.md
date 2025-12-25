# Ctrl 命令 Bug 修復總結

## 問題 🐛

使用 `ctrl npc` 切換到 NPC 後，再用 `ctrl me` 切換回玩家時，**玩家的屬性會恢復到遊戲初始狀態**，導致之前的所有進度（位置、物品、屬性變化等）全部丟失。

## 影響範圍 ⚠️

- **嚴重度**: 高
- **影響**: 玩家進度丟失
- **觸發條件**: 使用 `ctrl` 命令切換角色

## 範例

### Bug 重現

```
> status
位置: (5, 5), 背包: 蘋果 x3, HP: 80/100

> ctrl 商人
已切換控制角色為: 商人

> ctrl me
已切換回原始角色

> status
位置: (0, 0), 背包: 空, HP: 100/100  ❌ 所有進度丟失！
```

## 根本原因 🔍

### 原始邏輯流程

```rust
if let Some(current_id) = &game_world.current_controlled_id {
    // 只有在控制 NPC 時才保存
    game_world.npc_manager.add_npc(id, me.clone(), aliases);
}
// ❌ 如果 current_controlled_id 是 None（控制玩家），不會保存狀態

// 直接切換
*me = npc;  // ❌ 玩家當前狀態被覆蓋！
```

### 問題分析

1. **首次切換** (玩家 → NPC):
   - `current_controlled_id = None`（控制原始玩家）
   - `if let Some(...)` 不執行
   - **玩家當前狀態沒有保存**
   - `me` 被 NPC 數據覆蓋
   - **玩家進度永久丟失**

2. **切換回玩家** (NPC → 玩家):
   - 從 `original_player` 恢復
   - `original_player` 是初始化時的備份
   - **得到初始狀態，不是最新狀態**

## 修復方案 ✅

### 核心思路

在首次切換到 NPC **之前**，將玩家的**當前狀態**保存到 `original_player`。

### 修復代碼

```rust
if let Some(current_id) = &game_world.current_controlled_id {
    // 如果控制的是 NPC，加回 NPC 列表
    let npc_to_restore = me.clone();
    let id = current_id.clone();
    let aliases = vec![npc_to_restore.name.clone()];
    game_world.npc_manager.add_npc(id, npc_to_restore, aliases);
} else {
    // ✅ 新增：如果控制的是原始玩家，更新 original_player 為當前狀態
    game_world.original_player = Some(me.clone());
}
```

### 修復效果

```
> status
位置: (5, 5), 背包: 蘋果 x3, HP: 80/100

> ctrl 商人
（保存玩家當前狀態到 original_player）
已切換控制角色為: 商人

> ctrl me
（從 original_player 恢復）
已切換回原始角色

> status
位置: (5, 5), 背包: 蘋果 x3, HP: 80/100  ✅ 狀態正確保存！
```

## 數據流對比

### 修復前 ❌

```
初始: original_player = 玩家初始狀態 (HP:100, 位置:0,0)

玩家操作 → 玩家當前狀態 (HP:80, 位置:5,5)
    ↓
ctrl npc → me = NPC (玩家狀態丟失！)
    ↓
ctrl me → me = original_player (HP:100, 位置:0,0)
```

### 修復後 ✅

```
初始: original_player = 玩家初始狀態 (HP:100, 位置:0,0)

玩家操作 → 玩家當前狀態 (HP:80, 位置:5,5)
    ↓
ctrl npc → original_player = 當前狀態 (HP:80, 位置:5,5) ✅
         → me = NPC
    ↓
ctrl me → me = original_player (HP:80, 位置:5,5) ✅
```

## 測試驗證 ✅

### 測試場景 1: 基本切換

```
玩家(移動+撿物品) → ctrl npc → ctrl me → 狀態恢復 ✅
```

### 測試場景 2: 多次切換

```
玩家 → NPC1 → 玩家 → NPC2 → 玩家 ✅
每次都保持正確狀態
```

### 測試場景 3: 連續操作

```
玩家(狀態A) → ctrl npc → ctrl me → 玩家(狀態A) → 操作 → 玩家(狀態B)
→ ctrl npc2 → ctrl me → 玩家(狀態B) ✅
```

## 程式碼變更

### 修改文件
- `src/app.rs` - `handle_switch_control()` 函數

### 變更統計
- **新增行數**: 3 行
- **修改邏輯**: 1 處
- **影響範圍**: 角色切換功能

## 性能影響

- **克隆操作**: 每次切換執行一次 `me.clone()`
- **頻率**: 低（只在用戶執行 ctrl 命令時）
- **開銷**: 小（Person 結構相對簡單）
- **結論**: 性能影響可忽略

## 相關功能

### 受影響的命令
- `ctrl <npc>` - 切換到 NPC
- `ctrl me` - 切換回玩家
- `control <npc>` - 別名

### 相關數據結構
```rust
pub struct GameWorld {
    pub original_player: Option<Person>,  // 玩家備份
    pub current_controlled_id: Option<String>,  // 當前控制的角色
}
```

## 總結

### 修復前
- ❌ 玩家進度會丟失
- ❌ 切換回玩家時恢復初始狀態
- ❌ 無法正常使用 ctrl 功能

### 修復後
- ✅ 玩家進度正確保存
- ✅ 切換回玩家時恢復最新狀態
- ✅ 支持多次切換
- ✅ 狀態持久化正確

### 關鍵改進
只新增了 3 行代碼，在 `else` 分支保存玩家當前狀態，徹底解決了屬性丟失的問題。

---

**修復日期**: 2025-12-25  
**Bug ID**: ctrl-attribute-loss  
**嚴重度**: 高  
**修復者**: AI Assistant  
**狀態**: ✅ 已修復並測試通過
