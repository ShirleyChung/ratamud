# 修復：NPC 跟隨玩家跨地圖問題

## 問題描述

當玩家使用 `flyto` 命令傳送到其他地圖（如森林），原本在初始之地的 NPC 也會出現在新地圖上。

## 問題原因

### 1. Person 結構缺少地圖資訊
`Person` 結構只有 `x`, `y` 座標，沒有記錄 NPC 所在的**地圖名稱**：

```rust
pub struct Person {
    pub x: usize,
    pub y: usize,
    // ❌ 缺少 map 欄位
}
```

### 2. NPC 查詢沒有過濾地圖
`get_npcs_at(x, y)` 會返回**所有地圖**上該座標的 NPC：

```rust
// ❌ 沒有地圖過濾
pub fn get_npcs_at(&self, x: usize, y: usize) -> Vec<&Person> {
    self.npcs.values()
        .filter(|npc| npc.x == x && npc.y == y)  // 只檢查座標
        .collect()
}
```

### 3. 玩家切換地圖時沒有更新地圖資訊
當玩家 `flyto 森林` 時：
```rust
game_world.current_map = "森林".to_string();  // 只改變當前地圖
// ❌ 沒有更新 me.map
```

## 解決方案

### 1. 添加 `map` 欄位到 Person

```rust
pub struct Person {
    pub x: usize,
    pub y: usize,
    #[serde(default = "default_map")]
    pub map: String,  // ✅ 記錄所在地圖
    // ...
}

fn default_map() -> String {
    "初始之地".to_string()
}
```

使用 `#[serde(default)]` 確保舊存檔相容。

### 2. 添加地圖過濾方法

```rust
/// 獲取指定地圖和位置的 NPC
pub fn get_npcs_at_in_map(&self, map_name: &str, x: usize, y: usize) -> Vec<&Person> {
    self.npcs.values()
        .filter(|npc| npc.map == map_name && npc.x == x && npc.y == y)
        .collect()
}
```

### 3. 更新所有 NPC 查詢調用

修改所有 `get_npcs_at()` 調用為 `get_npcs_at_in_map()`：

```rust
// ❌ 舊代碼
let npcs_here = game_world.npc_manager.get_npcs_at(me.x, me.y);

// ✅ 新代碼
let npcs_here = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map, me.x, me.y);
```

### 4. 切換地圖時更新玩家地圖

```rust
// flyto 命令處理
if game_world.maps.contains_key(&target) {
    game_world.current_map = target.clone();
    me.map = target.clone();  // ✅ 更新玩家所在地圖
    // ...
}
```

## 修改的檔案

1. **src/person.rs**
   - 添加 `map: String` 欄位
   - 添加 `default_map()` 函數
   - 在 `Person::new()` 初始化 `map` 欄位

2. **src/npc_manager.rs**
   - 添加 `get_npcs_at_in_map()` 方法

3. **src/app.rs**
   - 更新所有 `get_npcs_at()` 調用（5處）
   - 在 `handle_flyto()` 中更新 `me.map`

## 測試

```bash
cargo build
cargo run

# 測試步驟
> flyto 森林
> look              # 應該看不到初始之地的 NPC
> flyto 初始之地
> look              # 應該看到原本的 NPC
```

### 測試案例

| 操作 | 預期結果 |
|------|---------|
| 在初始之地 | 看到初始之地的 NPC |
| flyto 森林 | 只看到森林的 NPC（如果有） |
| flyto 初始之地 | 看到原本的 NPC |
| summon 在森林 | NPC 出現在森林 |

## 向後相容性

使用 `#[serde(default = "default_map")]` 確保舊存檔相容：

- **舊存檔（沒有 map 欄位）**：自動設為 "初始之地"
- **新存檔**：正確保存 NPC 所在地圖

```rust
// 載入舊存檔時
{
  "name": "商人",
  "x": 10,
  "y": 20
  // 缺少 map 欄位
}

// 自動補充為
person.map = "初始之地"
```

## 未來改進

### 1. NPC 地圖傳送
可以添加命令讓 NPC 在地圖間移動：

```rust
// 將 NPC 傳送到其他地圖
> move npc 商人 to 森林 50 50
```

### 2. NPC AI 跨地圖行為
AI 執行緒也需要考慮地圖過濾：

```rust
for npc_id in npc_ids {
    if let Some(npc) = manager.get_npc(&npc_id) {
        // 只處理當前活躍地圖的 NPC
        if npc.map != active_map {
            continue;
        }
        // AI 邏輯...
    }
}
```

### 3. 多地圖 NPC 顯示
可以顯示各地圖的 NPC 數量：

```rust
> list npcs
初始之地: 商人(10,20), 農夫(30,40)
森林: 獵人(50,60)
沙漠: (無 NPC)
```

## 相關問題

### Q: 如果 NPC 在不存在的地圖怎麼辦？
A: 使用預設地圖或顯示警告，可以添加驗證：

```rust
if !game_world.maps.contains_key(&npc.map) {
    npc.map = "初始之地".to_string();
}
```

### Q: summon 命令會在哪個地圖？
A: summon 創建的 NPC 會在玩家當前地圖：

```rust
npc.map = game_world.current_map.clone();
```

**已修復的命令**：
- ✅ `create npc` - 在玩家當前地圖創建
- ✅ `summon` - 將 NPC 傳送到玩家當前地圖

## 總結

✅ **問題解決**
- NPC 現在綁定到特定地圖
- 跨地圖傳送不會帶走 NPC
- 完全向後相容
- 為未來的跨地圖功能打下基礎

🎉 **NPC 現在會正確留在各自的地圖上！**

## 補充修正（2024-12-16）

### 問題：create npc 和 summon 沒有設置地圖

#### create npc 命令
創建 NPC 時沒有設置 `map` 欄位，導致 NPC 使用預設值 "初始之地"：

```rust
// ❌ 舊代碼
let mut npc = Person::new(npc_name.clone(), description);
npc.x = me.x;
npc.y = me.y;
// 缺少 npc.map 設置
```

**修正**：
```rust
// ✅ 新代碼
let mut npc = Person::new(npc_name.clone(), description);
npc.x = me.x;
npc.y = me.y;
npc.map = game_world.current_map.clone();  // 設置在當前地圖
```

#### summon 命令
召喚 NPC 時只更新座標，沒有更新地圖：

```rust
// ❌ 舊代碼
if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
    npc.move_to(me.x, me.y);
    // 缺少 map 更新
}
```

**修正**：
```rust
// ✅ 新代碼
if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
    npc.move_to(me.x, me.y);
    npc.map = game_world.current_map.clone();  // 更新到玩家當前地圖
}
```

### 測試

```bash
# 測試 create npc
> flyto 森林
> create npc 商人 小李
> look              # 應該看到小李在森林
> flyto 初始之地
> look              # 不應該看到小李

# 測試 summon
> summon 小李       # 應該將小李從森林召喚到初始之地
> look              # 應該看到小李
> flyto 森林
> look              # 不應該看到小李（已經離開森林）
```

### 修改檔案
- `src/app.rs` (第 1555 行、第 1127 行)

✅ **所有 NPC 相關命令現在都正確設置地圖了！**
