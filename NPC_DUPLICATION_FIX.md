# NPC 重複問題修復說明

## 問題描述

當使用 `ctrl <npc>` 命令操控 NPC 時，會出現兩個相同的 NPC：
1. 你操控的 NPC（作為玩家角色 `me`）
2. 原始的 NPC（仍然在 `npc_manager` 中）

這導致在地圖上看到重複的 NPC。

## 問題根源

在 `handle_switch_control` 函數中（src/app.rs 第 1728-1732 行）：

```rust
// 舊代碼
if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
    let npc_clone = npc.clone();  // ❌ 克隆 NPC
    *me = npc_clone;              // ❌ 將克隆賦給 me
    // 但原始 NPC 仍在 npc_manager 中！
}
```

問題：
- NPC 被克隆到 `me`
- **原始 NPC 仍然留在 `npc_manager` 中**
- 結果：地圖上有兩個相同的 NPC

## 解決方案

修改邏輯，使用 **移除-操控-恢復** 模式：

### 1. 切換到 NPC 時：從列表中移除
```rust
// 新代碼
if let Some(npc) = game_world.npc_manager.remove_npc(&npc_name) {
    *me = npc;  // ✅ 直接使用移除的 NPC
    game_world.current_controlled_id = Some(npc_id);
}
```

### 2. 切換到另一個角色時：將前一個加回列表
```rust
if let Some(current_id) = &game_world.current_controlled_id {
    // 將當前操控的角色（me）加回 NPC 列表
    let npc_to_restore = me.clone();
    let id = current_id.clone();
    let aliases = vec![npc_to_restore.name.clone()];
    game_world.npc_manager.add_npc(id, npc_to_restore, aliases);
}
```

### 3. 切換回原始玩家時：NPC 重新出現
```rust
if npc_name == "me" || npc_name == "我" {
    if let Some(original) = &game_world.original_player {
        *me = original.clone();
        game_world.current_controlled_id = None;
        // 前一個被操控的 NPC 已在步驟2加回列表
    }
}
```

## 修復效果

### 修復前：
```
> summon worker1
> npcs
  - worker1 (在 50, 50)

> ctrl worker1
> npcs
  - worker1 (在 50, 50)  ← 這是你操控的
  - worker1 (在 50, 50)  ← ❌ 這是重複的！
```

### 修復後：
```
> summon worker1
> npcs
  - worker1 (在 50, 50)

> ctrl worker1
> npcs
  (空的)  ← ✅ NPC 被移除，你正在操控它

> ctrl me
> npcs
  - worker1 (在 50, 50)  ← ✅ NPC 重新出現
```

## 測試步驟

1. 啟動遊戲
2. 召喚一個 NPC：`summon worker1`
3. 查看 NPC 列表：`npcs` → 應該看到 1 個 worker1
4. 操控 NPC：`ctrl worker1`
5. 再次查看列表：`npcs` → 應該是空的（NPC 被你操控）
6. 切換回原始角色：`ctrl me`
7. 查看列表：`npcs` → worker1 重新出現

## 相關檔案

- `src/app.rs`: `handle_switch_control` 函數（第 1696-1746 行）
- `src/npc_manager.rs`: `add_npc`, `remove_npc` 方法

## 優點

1. **不再重複**: 操控的 NPC 從列表中移除，避免重複
2. **狀態保持**: NPC 的狀態（位置、HP 等）在切換時正確保存
3. **可恢復**: 切換回原始角色時，NPC 正確恢復到列表中
4. **更真實**: 模擬真實的"附身"行為 - 你操控 NPC 時，NPC 就是你
