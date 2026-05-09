# 自動戰鬥回合系統

## 問題
戰鬥中NPC看起來沒有動作，只有在玩家輸入攻擊指令時才會反擊。

## 原因
戰鬥回合只在玩家執行攻擊技能時觸發，沒有自動回合系統。

## 解決方案

### 1. 添加自動戰鬥回合計時器
**檔案**: `src/app.rs` - `run_main_loop()`

```rust
let mut last_combat_round = Instant::now();
let combat_round_interval = Duration::from_secs(3);  // 每3秒執行一次戰鬥回合
```

### 2. 在主迴圈中自動執行戰鬥回合
```rust
// --- 3.5. 自動戰鬥回合 ---
// 如果在戰鬥中，每3秒自動執行一次戰鬥回合
use crate::world::CombatState;
if !matches!(game_world.combat_state, CombatState::None) {
    let now = Instant::now();
    if now.duration_since(last_combat_round) >= combat_round_interval {
        let _ = execute_combat_round(&mut output_manager, &mut game_world, &mut me);
        last_combat_round = now;
    }
}
```

### 3. 改進戰鬥回合顯示
```rust
// 顯示回合開始
if let CombatState::InCombat { round, .. } = &game_world.combat_state {
    output_manager.print(format!("--- 回合 {} ---", round));
}
```

## 戰鬥流程

### 開始戰鬥
```
> punch 商人
⚔️  戰鬥開始！你 vs 商人
💥 me 說：「吃我一拳」造成 2 點傷害！商人 剩餘 HP: 98/100
--- 回合 2 ---
💥 商人 說：「你搞什麼」造成 2 點傷害！你剩餘 HP: 98/100
```

### 自動回合（每3秒）
```
--- 回合 3 ---
💥 商人 說：「別惹我」造成 3 點傷害！你剩餘 HP: 95/100

--- 回合 4 ---
（商人沒有行動，50%機率）

--- 回合 5 ---
💥 商人 說：「你搞什麼」造成 2 點傷害！你剩餘 HP: 93/100
```

### 玩家可以在回合間行動
```
--- 回合 6 ---
> kick
💥 me 說：「看我的飛踢」造成 3 點傷害！商人 剩餘 HP: 95/100
💥 商人 說：「你搞什麼」造成 2 點傷害！你剩餘 HP: 91/100
```

### 逃離戰鬥
```
> esc
逃跑獲得 3 點戰鬥經驗
你逃離了戰鬥！
```

## 特性

### 自動進行
- ✅ 戰鬥中每3秒自動執行一次回合
- ✅ 不需要玩家輸入指令
- ✅ NPC會主動攻擊

### 回合機制
- ✅ 顯示回合數
- ✅ NPC有50%機率行動
- ✅ 隨機選擇技能（punch或kick）
- ✅ 自動減少技能冷卻

### 玩家行動
- ✅ 可以在任何時候輸入攻擊指令
- ✅ 玩家行動後立即觸發一次回合
- ✅ 自動回合持續進行

### 戰鬥結束
- ✅ 任一方HP低於50%自動結束
- ✅ 玩家可以使用 `esc` 逃離
- ✅ 自動結算經驗值

## 練習模式 vs 戰鬥模式

### 練習模式（無目標）
```
> punch
你練習了 punch (熟練度 +1)
> kick
你練習了 kick (熟練度 +1)
```
- 無冷卻限制
- 熟練度 +1
- 可以連續練習

### 戰鬥模式（有目標或在戰鬥中）
```
> punch 商人
⚔️  戰鬥開始！你 vs 商人
💥 me 說：「吃我一拳」造成 2 點傷害！
--- 回合 2 ---
💥 商人 說：「你搞什麼」造成 2 點傷害！
```
- 有冷卻限制
- 熟練度 +2
- 每3秒自動回合
- NPC會反擊

## 時間設定

### 回合間隔
```rust
let combat_round_interval = Duration::from_secs(3);  // 3秒
```

可以調整為：
- `Duration::from_secs(1)` - 快速戰鬥（1秒）
- `Duration::from_secs(5)` - 慢速戰鬥（5秒）
- `Duration::from_millis(1500)` - 1.5秒

### NPC行動機率
```rust
if rng.gen_bool(0.5) {  // 50%機率
    // 執行攻擊
}
```

可以調整為：
- `0.3` - 30%機率（簡單）
- `0.7` - 70%機率（困難）
- `1.0` - 100%機率（每回合必定攻擊）

## 技術細節

### 計時器實作
- 使用 `Instant::now()` 記錄上次回合時間
- 每個主迴圈檢查是否達到間隔
- 達到間隔時執行 `execute_combat_round()`

### 回合執行
1. 檢查是否在戰鬥中
2. 顯示回合數
3. 遍歷所有NPC參與者
4. 隨機決定是否行動
5. 執行攻擊
6. 更新回合數
7. 減少技能冷卻
8. 檢查戰鬥結束

### 與玩家行動的協調
- 玩家輸入攻擊指令 → 立即執行並觸發回合
- 自動回合計時器 → 每3秒觸發一次
- 兩者獨立運作，不會衝突

## 測試方式

```bash
cargo run

# 開始戰鬥
> flyto 3,3
> punch 商人

# 觀察：
# 1. 每3秒自動顯示新回合
# 2. NPC約50%機率會攻擊
# 3. 可以隨時輸入攻擊指令
# 4. 使用 esc 可以逃離

# 等待約10秒，觀察自動回合
```

## 相關檔案

- `src/app.rs` - 主迴圈和自動回合邏輯
- `COMBAT_LOOP_REFACTOR.md` - 戰鬥迴圈系統
- `COMBAT_SKILLS_FIX.md` - 戰鬥技能修復
