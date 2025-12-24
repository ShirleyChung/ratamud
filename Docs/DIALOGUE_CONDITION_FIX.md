# 對話系統條件評估修復

## 修復日期
2025-12-24

## 問題描述
對話系統的條件評估有誤。原本設計應該是根據**玩家**的屬性來決定 NPC 說什麼話，但實作時錯誤地使用了 **NPC 自己**的屬性來評估條件。

### 錯誤範例
```bash
# 設定：當顏值>80且性別=女時說這句話
sdl 櫻花 set 閒聊 when 顏值>80 and 性別=女 say 你長得好漂亮啊

# 錯誤行為：檢查櫻花自己的顏值和性別（應該檢查玩家的）
# 正確行為：檢查玩家的顏值和性別
```

## 修復內容

### 修改的方法簽名

#### person.rs

1. **get_weighted_dialogue**
```rust
// 舊版
pub fn get_weighted_dialogue(&self, topic: &str) -> Option<String>

// 新版
pub fn get_weighted_dialogue(&self, topic: &str, target_person: &Person) -> Option<String>
```

2. **try_talk**
```rust
// 舊版
pub fn try_talk(&self, topic: &str) -> Option<String>

// 新版
pub fn try_talk(&self, topic: &str, target_person: &Person) -> Option<String>
```

### 修改的調用點

#### app.rs

1. **handle_talk** (處理 talk 命令)
```rust
// 舊版
if let Some(dialogue) = npc.try_talk(&topic) {

// 新版
if let Some(dialogue) = npc.try_talk(&topic, me) {
```

2. **handle_look** (look 命令時的見面對話)
```rust
// 舊版
if let Some(greeting) = npc.try_talk("見面") {

// 新版
if let Some(greeting) = npc.try_talk("見面", me) {
```

### 核心邏輯變更

在 `person.rs` 的 `get_weighted_dialogue` 方法中：

```rust
// 舊版：使用 NPC 自己的屬性評估條件
let weights: Vec<f32> = options.iter()
    .map(|opt| opt.get_effective_weight(self))  // self = NPC
    .collect();

// 新版：使用目標 Person（通常是玩家）的屬性評估條件
let weights: Vec<f32> = options.iter()
    .map(|opt| opt.get_effective_weight(target_person))  // target_person = 玩家
    .collect();
```

## 影響範圍

### 正確評估的屬性
現在對話條件會正確評估**玩家**的以下屬性：
- `hp`, `mp`, `max_hp`, `max_mp`
- `strength`/`力量`, `knowledge`/`知識`, `sociality`/`交誼`
- `gender`/`性別`, `appearance`/`顏值`
- `relationship`/`好感度`, `talk_eagerness`/`積極度`
- `age`/`年齡`
- `items_count`/`物品數量`
- `item:物品名` (特定物品數量)

### 使用範例

```bash
# 範例 1：根據玩家性別和顏值
sdl 櫻花 set 閒聊 when 顏值>80 and 性別=女 say 你長得好漂亮啊

# 範例 2：根據玩家物品數量
sdl 商人 set 閒聊 when 物品數量>5 say 要不要賣些東西給我

# 範例 3：根據玩家精神力
sdl 工人 set 閒聊 when mp<50000 say 你看起來很累，要休息一下嗎

# 範例 4：根據玩家力量
sdl 教練 set 閒聊 when 力量>100 say 你的力量進步很多！
```

## 測試驗證

### 測試步驟
1. 設置玩家屬性
```bash
set me 性別 女
set me 顏值 85
```

2. 設置 NPC 對話條件
```bash
sdl 櫻花 set 閒聊 when 顏值>80 and 性別=女 say 你長得好漂亮啊
sdl 櫻花 set 閒聊 when 顏值<80 say 你好呀
```

3. 測試對話
```bash
talk 櫻花 閒聊
# 預期：因為玩家顏值=85>80 且性別=女，應該說「你長得好漂亮啊」
```

4. 修改玩家屬性後再測試
```bash
set me 顏值 60
talk 櫻花 閒聊
# 預期：因為玩家顏值=60<80，應該說「你好呀」
```

## 向後兼容性

- ✅ 無條件的對話仍然正常工作
- ✅ 舊的 `get_dialogue` 方法保留（內部調用新方法）
- ✅ 所有現有功能不受影響

## 文檔更新

- ✅ 更新 `Docs/README_DIALOGUE.md` - 新增條件評估說明
- ✅ 註明條件是根據**玩家**屬性評估，非 NPC 屬性

## 編譯驗證

```bash
cargo build     # ✅ 成功
cargo clippy    # ✅ 無警告
```

## 總結

這次修復確保了對話系統按照原本的設計意圖運作：
- **NPC 根據玩家的狀態來決定說什麼話**
- 讓對話更加動態和個性化
- 玩家的屬性、物品、狀態會直接影響 NPC 的反應

這使得對話系統更具互動性和沉浸感。
