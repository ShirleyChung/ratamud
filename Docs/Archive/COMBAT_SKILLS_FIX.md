# 戰鬥技能初始化修復

## 問題
執行 `punch` 或 `kick` 指令時顯示「未知的指令」。

## 原因
舊存檔檔案中沒有 `combat_skills` 欄位。由於使用了 `#[serde(default)]`，載入時會初始化為空的 HashMap，但不會調用 `init_combat_skills()` 初始化戰鬥技能。

## 解決方案

### 1. 新增 `ensure_combat_skills()` 函數
**檔案**: `src/person.rs`

```rust
/// 確保戰鬥技能已初始化（用於舊存檔載入後的修復）
pub fn ensure_combat_skills(&mut self) {
    if self.combat_skills.is_empty() {
        self.init_combat_skills();
    }
}
```

### 2. 在載入時自動修復
**檔案**: `src/npc_manager.rs`

#### a. 載入所有NPC時
```rust
// 在 load_all_from_directory() 中
if let Ok(mut npc) = Person::load(person_dir, file_stem) {
    // 確保戰鬥技能已初始化（舊存檔修復）
    npc.ensure_combat_skills();
    // ...
}
```

#### b. 確保玩家 "me" 存在時
```rust
// 在 ensure_me() 中
if let Some(me) = self.get_me() {
    let mut me_clone = me.clone();
    me_clone.ensure_combat_skills();
    
    // 更新 npc_manager 中的 me
    if let Some(stored_me) = self.npcs.get_mut("me") {
        stored_me.ensure_combat_skills();
    }
    
    Ok(me_clone)
}
```

## 修復效果

### 修復前
```
> punch
未知的指令
```

### 修復後
```
> punch
你練習了 punch (熟練度 +1)

> punch 商人
⚔️  戰鬥開始！你 vs 商人
💥 me 說：「吃我一拳」造成 2 點傷害！商人 剩餘 HP: 98/100
```

## 自動修復流程

1. **載入遊戲時**
   - 所有從檔案載入的角色（包括玩家和NPC）
   - 自動檢查 `combat_skills` 是否為空
   - 如果為空，自動初始化戰鬥技能

2. **初始化內容**
   - `punch` 技能（傷害2，冷卻2）
   - `kick` 技能（傷害3，冷卻3）
   - 預設技能台詞

## 向後相容

- ✅ 舊存檔可正常載入
- ✅ 自動補充缺失的戰鬥技能
- ✅ 不影響已有的戰鬥技能資料
- ✅ 新角色正常初始化

## 測試方式

### 1. 測試舊存檔
```bash
# 如果有舊存檔（沒有 combat_skills）
cargo run
> punch
# 應該可以正常執行
```

### 2. 測試新角色
```bash
cargo run
> create npc person 新NPC
> punch 新NPC
# 應該可以正常戰鬥
```

### 3. 測試戰鬥
```bash
cargo run
> flyto 3,3
> punch 商人
> kick 商人
# 應該可以看到戰鬥效果
```

## 相關檔案

- `src/person.rs` - `ensure_combat_skills()` 函數
- `src/npc_manager.rs` - 載入時自動修復邏輯
- `src/app.rs` - 戰鬥指令處理
- `src/input.rs` - punch/kick 指令定義

## 技術細節

### 檢查邏輯
```rust
if self.combat_skills.is_empty() {
    self.init_combat_skills();
}
```

只有當 `combat_skills` 完全為空時才初始化，避免覆蓋已有的技能資料。

### 修復時機
- ✅ 載入所有NPC時（`load_all_from_directory`）
- ✅ 確保玩家存在時（`ensure_me`）
- ✅ 每個角色只修復一次

### 初始化內容
所有角色都會獲得：
- 基礎戰鬥技能（punch, kick）
- 預設技能台詞
- 正確的技能參數（傷害、冷卻）

## 練習模式改進

### 問題
練習戰鬥技能時（不在戰鬥中）也有冷卻時間限制，不合理。

### 解決方案
修改 `practice_skill()` 函數，只在戰鬥中才檢查和設置冷卻。

**修改前**：
```rust
pub fn practice_skill(&mut self, skill_name: &str, in_combat: bool) -> Option<String> {
    // 總是檢查冷卻
    if skill.current_cooldown > 0 {
        return Some(format!("你還沒準備好{skill_name}"));
    }
    // 總是設置冷卻
    skill.current_cooldown = skill.cooldown;
}
```

**修改後**：
```rust
pub fn practice_skill(&mut self, skill_name: &str, in_combat: bool) -> Option<String> {
    // 只在戰鬥中檢查冷卻
    if in_combat && skill.current_cooldown > 0 {
        return Some(format!("你還沒準備好{skill_name}"));
    }
    // 只在戰鬥中設置冷卻
    if in_combat {
        skill.current_cooldown = skill.cooldown;
    }
}
```

### 效果

**練習模式（不在戰鬥中）**：
```
> punch
你練習了 punch (熟練度 +1)
> punch
你練習了 punch (熟練度 +1)
> punch
你練習了 punch (熟練度 +1)
# 可以連續練習，無冷卻限制
```

**戰鬥模式**：
```
> punch 商人
💥 me 說：「吃我一拳」造成 2 點傷害！
> punch
你還沒準備好punch
# 戰鬥中有冷卻限制
```

### 熟練度系統
- 練習模式：每次 +1 熟練度
- 戰鬥模式：每次 +2 熟練度
- 練習無冷卻，戰鬥有冷卻
