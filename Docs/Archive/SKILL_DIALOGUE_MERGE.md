# 戰鬥技能台詞系統整合

## 修改內容

將戰鬥技能台詞系統合併到一般對話系統中，使 `sdl` 指令可以通用設定戰鬥技能的台詞。

## 變更說明

### 1. 移除 `skill_dialogues` 欄位
**檔案**: `src/person.rs`

- 移除 `Person` 結構中的 `pub skill_dialogues: HashMap<String, String>`
- 戰鬥技能台詞現在使用一般的 `dialogues` 系統儲存

### 2. 新增 `get_skill_dialogue()` 函數
**檔案**: `src/person.rs`

```rust
pub fn get_skill_dialogue(&self, skill_name: &str) -> String {
    // 從對話系統中獲取技能台詞
    if let Some(dialogue_options) = self.dialogues.get(skill_name) {
        if let Some(option) = dialogue_options.first() {
            return option.text.clone();
        }
    }
    // 預設台詞
    "攻擊！".to_string()
}
```

### 3. 更新初始化邏輯
**檔案**: `src/person.rs` - `init_combat_skills()`

```rust
// 設置預設技能台詞（使用一般對話系統）
if self.name == "me" {
    self.set_dialogue("punch".to_string(), "吃我一拳".to_string());
    self.set_dialogue("kick".to_string(), "看我的飛踢".to_string());
} else {
    self.set_dialogue("punch".to_string(), "你搞什麼".to_string());
    self.set_dialogue("kick".to_string(), "別惹我".to_string());
}
```

### 4. 更新所有使用技能台詞的地方

**修改位置**:
- `src/app.rs` - `execute_attack()` 函數（2處）
- `src/world.rs` - `apply_npc_combat_skill()` 函數（1處）

**修改前**:
```rust
let dialogue = npc.skill_dialogues.get(skill_name)
    .cloned()
    .unwrap_or_else(|| "攻擊！".to_string());
```

**修改後**:
```rust
let dialogue = npc.get_skill_dialogue(skill_name);
```

## 使用方式

### 設定戰鬥技能台詞

現在可以使用 `sdl` 指令設定戰鬥技能的台詞：

```
sdl npc名稱 技能名稱 台詞內容
```

**範例**:
```
sdl 張三 punch 揮拳攻擊
sdl 李四 kick 旋風腿
sdl 王五 punch 你找死！
```

### 支援條件式台詞

也可以設定帶條件的戰鬥技能台詞：

```
sdl 張三 punch add 你惹怒我了！ when 血量<50
sdl 李四 kick set 看我的絕招！ when 戰鬥經驗>100 say 超級旋風腿
```

### 玩家自己的技能台詞

```
sdl me punch 吃我一拳
sdl me kick 看我的飛踢
```

## 優勢

### 1. 統一管理
- 所有台詞（對話、戰鬥）都使用同一個系統
- 減少程式碼重複

### 2. 功能豐富
- 支援條件式台詞
- 支援多個台詞選項（隨機選擇）
- 可根據好感度、屬性等條件變化

### 3. 簡化指令
- 不需要單獨的戰鬥技能台詞設定指令
- `sdl` 指令通用於所有台詞設定

## 技術細節

### 台詞獲取順序
1. 從 `dialogues` HashMap 中查找技能名稱
2. 如果找到，返回第一個對話選項
3. 如果沒找到，返回預設 "攻擊！"

### 條件支援
由於使用一般對話系統，技能台詞也支援：
- 好感度條件
- 屬性條件（力量、顏值等）
- 時間條件
- 任務進度條件

### 儲存格式
技能台詞儲存在 `dialogues` 中：
```json
{
  "dialogues": {
    "punch": [
      {
        "text": "揮拳攻擊",
        "conditions": []
      }
    ],
    "kick": [
      {
        "text": "旋風腿",
        "conditions": []
      }
    ]
  }
}
```

## 測試方式

### 1. 基本測試
```
cargo run
sdl 商人 punch 你敢打我？
flyto 3,3
punch 商人
# 觀察是否顯示 "你敢打我？"
```

### 2. 多台詞測試
```
sdl 商人 punch 你敢打我？
sdl 商人 punch 住手！
punch 商人
# 應該隨機顯示其中一句
```

### 3. 條件台詞測試
```
sdl 商人 punch add 你欺負弱小！ when 血量<50
# 打到商人血量低於50%時，觀察台詞變化
```

## 向後相容性

### 舊存檔檔案
- 如果存檔檔案中有 `skill_dialogues` 欄位，會被忽略（因為有 `#[serde(default)]`）
- 不會造成載入錯誤
- 舊台詞會遺失，需要重新用 `sdl` 設定

### 建議
如果有重要的技能台詞，建議在更新後重新設定。

## 相關檔案

- `src/person.rs` - Person 結構定義和技能台詞函數
- `src/app.rs` - 戰鬥執行邏輯
- `src/world.rs` - NPC 戰鬥技能應用
- `src/input.rs` - sdl 指令解析

## 後續可能的增強

1. **技能特定條件**：例如 `when 技能熟練度>50`
2. **連擊台詞**：根據連續攻擊次數變化台詞
3. **對手感知**：根據對手屬性選擇台詞
4. **情緒系統**：戰鬥中的情緒影響台詞選擇
