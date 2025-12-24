# NPC 載入問題修復

## 問題
舊的 NPC 文件缺少新增的欄位（max_hp, max_mp 等），導致載入失敗。

## 解決方案

### 1. 添加 Serde 預設值

在 `src/person.rs` 中為所有新欄位添加 `#[serde(default)]`：

```rust
#[serde(default = "default_hp")]
pub hp: i32,
#[serde(default = "default_mp")]
pub mp: i32,
#[serde(default = "default_max_hp")]
pub max_hp: i32,
#[serde(default = "default_max_mp")]
pub max_mp: i32,
#[serde(default = "default_stat")]
pub strength: i32,
// ... 等等

// 預設值函數
fn default_hp() -> i32 { 100000 }
fn default_mp() -> i32 { 100000 }
fn default_max_hp() -> i32 { 100000 }
fn default_max_mp() -> i32 { 100000 }
fn default_stat() -> i32 { 100 }
```

### 2. 添加 `npcs` 指令

新增 `npcs` 或 `listnpcs` 指令來列出所有載入的 NPC：

```bash
> npcs

═══ 所有 NPC ═══

  商人 - 精明的商人，販售各種物品 位於 (66, 63)
  醫生 - 經驗豐富的醫生 位於 (45, 52)
  工人 - 勤勞的工人 位於 (55, 48)

共 3 個 NPC
```

## 現在應該能看到 NPC 了！

### 測試步驟

1. **啟動遊戲**
```bash
cargo run --release
```

2. **列出所有 NPC**
```
> npcs
```

3. **傳送到 NPC 位置**
```
> flyto 66,63
> look
```

4. **召喚 NPC**
```
> summon 商人
> look
```

### 如果還是看不到 NPC

可能需要重新創建 NPC：

```bash
# 刪除舊的 NPC 文件
rm worlds/*/persons/*.json
# 保留 me.json
# 重新啟動遊戲

# 創建新 NPC
create npc 商人 merchant
set merchant hp 100
ctrl merchant
create item gold
get gold 1000
get apple 10
ctrl me
```

## 驗證

使用 `npcs` 指令應該能看到：
- 所有已載入的 NPC
- 它們的位置
- 它們的描述

如果列表為空，表示沒有 NPC 被載入，需要重新創建。
