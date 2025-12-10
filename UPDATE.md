# Ratamud 遊戲引擎更新日誌

## 📋 目錄
1. [核心功能實現](#核心功能實現)
2. [Item 初始化功能](#item-初始化功能)
3. [Look 命令增強](#look-命令增強)
4. [Get 命令](#get-命令)
5. [Drop 命令](#drop-命令)
6. [Status 命令增強](#status-命令增強)
7. [物品持久化](#物品持久化)

---

## 核心功能實現

### 1. 時間系統 ⏰
- **時間流逝**: 世界時間每秒自動更新（遊戲時間加速）
- **時間顯示**: 每60秒在輸出區顯示一次當前時間（格式：Day X HH:MM）
- **時間存檔**: 程式關閉時自動存檔時間，啟動時自動載回
- **時間文件位置**: `worlds/初始世界/time.json`
- **遊戲速度**: 預設為 60倍速（1秒現實時間 = 60秒遊戲時間）

### 2. 世界系統 🌍
- **多地圖支持**: 支援5個不同的地圖
  - 初始之地 (Normal)
  - 森林 (Forest)
  - 洞穴 (Cave)
  - 沙漠 (Desert)
  - 山脈 (Mountain)
- **地圖存檔**: 所有地圖存放在 `worlds/初始世界/maps/` 目錄
- **世界元數據**: 記錄世界名稱、描述、包含的地圖

### 3. 人物系統 👤
- **Me物件**: 玩家角色
- **NPC角色**: 5個NPC分散在地圖上
  - 商人 (merchant)
  - 路人 (traveler)
  - 醫生 (doctor)
  - 工人 (worker)
  - 農夫 (farmer)
- **人物屬性**:
  - 名字、描述
  - 能力列表
  - 持有物品
  - 當前狀態
  - 坐標位置
- **人物存檔**: 所有角色存放在 `worlds/初始世界/persons/` 目錄

### 4. 地圖系統 🗺️
- **地圖大小**: 100 × 100 格子
- **點的屬性**:
  - 可移動/不可移動
  - 詳細描述
- **地圖類型**: 每種類型有不同的地形生成邏輯

### 5. 輸入指令系統 ⌨️
所有輸入都視為指令，不需要 `/` 前綴：

#### 移動指令
- `u` / 上鍵: 往上移動
- `d` / 下鍵: 往下移動
- `l` / 左鍵: 往左移動
- `r` / 右鍵: 往右移動

#### 查看指令
- `look`: 查看當前位置及四周環境
  - 顯示當前位置坐標
  - 顯示當前點的描述
  - **顯示此處物品** ✨
  - 顯示上下左右四方向的描述
- `show minimap`: 開啟小地圖懸浮視窗
- `hide minimap`: 關閉小地圖視窗
- `show status`: 顯示Me的狀態面板（右側懸浮視窗）
  - 名字和當前位置 ✨
  - 描述
  - 能力列表
  - **持有物品清單** ✨
  - 當前狀態

#### 新增指令
- `get`: 撿起當前位置的所有物品
- `get <物品名>`: 撿起指定名稱的物品（模糊匹配）
- `drop <物品名>`: 放下身上持有的指定物品（模糊匹配）

#### 系統指令
- `hello [text]`: 在輸出區顯示指定的文字
- `quit` / `exit`: 退出遊戲

### 6. UI系統 🖥️
- **輸出區 (Output Area)**: 顯示遊戲訊息、look結果、系統訊息
- **輸入區 (Input Area)**: 顯示玩家輸入的指令
- **狀態列 (Status Bar)**: 單行顯示指令錯誤或移動訊息
  - 指令錯誤自動顯示
  - 錯誤訊息5秒後自動清除
- **懸浮視窗**:
  - 右上角顯示，寬度40%，高度60%
  - 可顯示狀態面板或小地圖
  - 支援多個Observable物件的顯示

### 7. 小地圖 (Minimap) 🗺️
- **位置**: 右上角懸浮視窗
- **內容**: 
  - 當前位置坐標
  - 四方向可移動/不可移動提示
  - 四方向環境描述
- **控制**: 
  - `show minimap`: 打開
  - `hide minimap`: 關閉
  - 移動時自動更新
- **持久化**: 開啟狀態在程式關閉時存檔，啟動時自動恢復

### 8. Observable特徵系統
定義了 `Observable` trait，用於在右側面板顯示不同的對象資訊：
- `show_title()`: 顯示標題
- `show_description()`: 顯示描述
- `show_list()`: 顯示列表項目

實現類：
- **Empty**: 顯示"無資料"
- **Person**: 顯示人物資訊
- **WorldInfo**: 顯示世界資訊

### 9. 設定系統 ⚙️
- **minimap狀態**: 記錄小地圖是否打開
- **文件位置**: `worlds/settings.json`
- **自動載入**: 啟動時自動恢復設定

---

## Item 初始化功能

### 功能說明

`Map::initialize_items()` 方法將隨機的 item 散落在地圖上，數量約為可移動地點的一半。

### 實現細節

#### 方法簽名
```rust
pub fn initialize_items(&mut self)
```

#### 工作流程
1. 收集地圖上所有可移動的點 (walkable points)
2. 計算要放置的 item 數量 = 可移動點數 / 2
3. 隨機選擇位置並在每個位置生成一個隨機 item
4. 將 item 放置在地圖的 Point 物件中

### 使用範例

#### 基本使用
```rust
// 創建地圖
let mut map = Map::new_with_type(
    "探險地圖".to_string(), 
    50, 
    50, 
    MapType::Forest
);

// 初始化 item
map.initialize_items();

// 現在地圖上已經散落了隨機的 item
```

#### 結合 GameWorld 使用
```rust
let mut world = GameWorld::new();

// 添加新地圖
let mut new_map = Map::new_with_type(
    "神秘洞穴".to_string(),
    30,
    30,
    MapType::Cave
);

// 初始化 item
new_map.initialize_items();

// 添加到世界
world.add_map(new_map);
```

### Item 類型

初始化會隨機生成以下類型的 item：

- **雜物 (Miscellaneous)**: 舊布料、石子、樹皮、羽毛
- **食物 (Food)**: 蘋果、麵包、乾肉、漿果
- **武器 (Weapon)**: 木劍、鐵劍、弓、匕首
- **裝備 (Armor)**: 皮衣、頭盔、盾牌
- **消耗品 (Consumable)**: 治療藥水、魔力藥水、毒藥
- **工具 (Tool)**: 火把、繩索、鎬、鑰匙

### 相關代碼位置

- **實現**: `src/map.rs` - `Map::initialize_items()` (第 308-330 行)
- **Item 定義**: `src/item.rs` - `Item` 結構體和 `Item::generate_random()`
- **Map 定義**: `src/map.rs` - `Map` 結構體和 `Point` 結構體

---

## Look 命令增強

### 功能說明

`look` 命令現在會顯示當前位置的物品列表。

### 顯示格式

```
【當前位置: (25, 30)】
【古老的廢墟遺跡】

🎁 此處物品:
  • [物品] 木劍 (武器)
  • [物品] 火把 (工具)
  • [物品] 蘋果 (食物)

↑ 北方: ...
↓ 南方: ...
← 西方: ...
→ 東方: ...
```

### 新增信息

- **物品列表**: 顯示當前位置的所有物品
- **物品格式**: `[物品] 名稱 (類型)`
- **視覺標記**: 🎁 符號標示物品區域
- **空位置**: 若沒有物品不顯示此區域

### 實現位置

檔案: `src/app.rs` - `display_look()` 函數 (第 301-310 行)

---

## Get 命令

### 功能說明

`get` 命令允許玩家撿起當前位置（Point）上的物品，並將其放入背包（Person.items）。

### 命令語法

#### 撿起所有物品
```
get
```
撿起當前位置的所有物品。

#### 撿起指定物品
```
get <物品名稱>
```
撿起指定名稱的物品（模糊匹配）。

### 使用範例

#### 例子 1：撿起所有物品
```
玩家位置: (15, 20)
此處物品: 蛋果、鐵劍、治療藥水

命令: get
結果:
  ✓ 撿起了: [物品] 蘋果 (食物)
  ✓ 撿起了: [物品] 鐵劍 (武器)
  ✓ 撿起了: [物品] 治療藥水 (消耗品)
  
狀態: 撿起了 3 個物品
```

#### 例子 2：撿起指定物品
```
命令: get 劍
結果: ✓ 撿起了: [物品] 鐵劍 (武器)

狀態: 撿起: 劍
```

#### 例子 3：物品不存在
```
命令: get 盾牌
結果: 找不到 "盾牌" 的物品。
```

### 完整遊戲流程

```
1. 初始化地圖並散落物品
   map.initialize_items();

2. 探索地圖
   命令: up / down / left / right (或 u/d/l/r)

3. 查看當前位置
   命令: look (或 l)
   
   輸出:
   【當前位置: (25, 30)】
   【古老的廢墟遺跡】
   
   🎁 此處物品:
     • [物品] 木劍 (武器)
     • [物品] 火把 (工具)

4. 撿起物品
   命令: get
   
   輸出:
   ✓ 撿起了: [物品] 木劍 (武器)
   ✓ 撿起了: [物品] 火把 (工具)

5. 查看背包
   命令: show status
   
   輸出顯示已撿起的物品
```

### 實現細節

#### CommandResult 新增變體
```rust
Get(Option<String>),  // 撿起物品 (可選：物品名稱)
```

#### 輸入解析 (input.rs)
```rust
"get" => {
    if parts.len() < 2 {
        CommandResult::Get(None)        // 撿起全部
    } else {
        let item_name = parts[1..].join(" ");
        CommandResult::Get(Some(item_name))  // 撿起指定物品
    }
}
```

### 代碼修改位置

| 檔案 | 位置 | 修改內容 |
|------|------|--------|
| src/input.rs | 第 142-153 行 | 添加 get 命令解析 |
| src/input.rs | 第 196 行 | 添加 Get(Option<String>) 變體 |
| src/app.rs | 第 187 行 | 添加 Get 命令處理 |
| src/app.rs | 第 451-491 行 | 添加 handle_get 函數 |

---

## Drop 命令

### 功能說明

`drop` 命令允許玩家放下身上持有的物品，將其放置在當前位置，供他人撿起。

### 命令語法

```
drop <物品名稱>
```

放下指定名稱的物品（模糊匹配）。

### 使用範例

#### 例子 1：放下指定物品
```
玩家位置: (15, 20)
背包物品: 蘋果、鐵劍、治療藥水

命令: drop 劍
結果:
  ✓ 放下了: [物品] 鐵劍 (武器)
  
狀態: 放下: 劍

此時位置物品: 蘋果、鐵劍、治療藥水
玩家背包: 蘋果、治療藥水
```

#### 例子 2：物品不存在
```
命令: drop 盾牌
結果: 身上沒有 "盾牌" 的物品。
```

#### 例子 3：物品名稱需要指定
```
命令: drop
結果: Usage: drop <item name>
```

### 完整遊戲流程 (Get + Drop)

```
1. 撿起物品
   命令: get
   背包: 蘋果、鐵劍、治療藥水

2. 查看背包
   命令: show status
   顯示: 【持有物品】(3 個)

3. 移動到新位置
   命令: right

4. 放下物品
   命令: drop 劍
   背包: 蘋果、治療藥水

5. 查看當前位置
   命令: look
   
   輸出顯示此處物品:
   🎁 此處物品:
     • [物品] 鐵劍 (武器)
```

### 實現細節

#### CommandResult 新增變體
```rust
Drop(String),  // 放下物品 (物品名稱)
```

#### 輸入解析 (input.rs)
```rust
"drop" => {
    if parts.len() < 2 {
        CommandResult::Error("Usage: drop <item name>".to_string())
    } else {
        let item_name = parts[1..].join(" ");
        CommandResult::Drop(item_name)
    }
}
```

#### Person 新增方法 (person.rs)
```rust
pub fn drop_item(&mut self, item_name: &str) -> Option<String> {
    if let Some(pos) = self.items.iter().position(|item| item.contains(item_name)) {
        Some(self.items.remove(pos))
    } else {
        None
    }
}
```

#### 命令處理 (app.rs)
```rust
fn handle_drop(
    item_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    if let Some(item) = me.drop_item(&item_name) {
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(me.x, me.y) {
                point.objects.push(item.clone());
                output_manager.print(format!("✓ 放下了: {}", item));
                output_manager.set_status(format!("放下: {}", item_name));
                
                // 保存角色物品
                let person_dir = format!("{}/persons", game_world.world_dir);
                let _ = me.save(&person_dir, "me");
            }
        }
    } else {
        output_manager.print(format!("身上沒有 \"{}\" 的物品。", item_name));
    }
}
```

### 代碼修改位置

| 檔案 | 位置 | 修改內容 |
|------|------|--------|
| src/input.rs | 第 153-161 行 | 添加 drop 命令解析 |
| src/input.rs | 第 216 行 | 添加 Drop(String) 變體 |
| src/person.rs | 第 42-49 行 | 添加 drop_item 方法 |
| src/app.rs | 第 188 行 | 添加 Drop 命令處理 |
| src/app.rs | 第 500-521 行 | 添加 handle_drop 函數 |

### 特性

- **與 Get 命令對稱**: Drop 反向操作 Get，形成完整的物品交互系統
- **模糊匹配**: 支援物品名稱的模糊匹配（與 get 命令相同）
- **多字詞支援**: 支援含有空格的物品名稱（如 "治療藥水"）
- **即時保存**: 放下物品後自動保存角色數據到檔案
- **完整反饋**: 提供操作成功或失敗的清晰提示

---

## Status 命令增強

### 功能改進

`show status` 命令已增強，現在會詳細列出玩家持有的所有物品。

### 顯示格式

#### 改進前
```
【角色名稱】

描述信息
狀態: 正常

• 無特殊能力
• 未持有物品
```

#### 改進後
```
【角色名稱】【位置: (25, 30)】

描述信息
狀態: 正常

• 【能力】
• 無特殊能力
• 【持有物品】(3 個)
• [物品] 蘋果 (食物)
• [物品] 鐵劍 (武器)
• [物品] 治療藥水 (消耗品)
```

### 新增信息

#### 1. 實時位置顯示
- **標題中添加位置**: 「角色名稱【位置: (x, y)】」
- 實時更新玩家當前座標

#### 2. 物品計數
- **物品數量統計**: 「【持有物品】(n 個)」
- 一眼看出背包中有多少物品

#### 3. 更清晰的物品列表
- 每個物品獨佔一行
- 保持 `[物品] 名稱 (類型)` 的格式
- 便於快速掃描物品清單

#### 4. 能力與物品分區
- 能力和物品各自有獨立的標題區
- 即使沒有能力也會顯示「【能力】」標題
- 即使沒有物品也會顯示「【持有物品】(0 個)」

### 代碼修改

#### 文件: src/person.rs

#### 修改 1: 更新 show_title()
**位置**: 第 87-88 行

```rust
// 改前
fn show_title(&self) -> String {
    self.name.clone()
}

// 改後
fn show_title(&self) -> String {
    format!("{}【位置: ({}, {})】", self.name, self.x, self.y)
}
```

#### 修改 2: 優化 show_list()
**位置**: 第 109-121 行

```rust
// 改進物品計數和顯示
if !self.items.is_empty() {
    list.push(format!("【持有物品】({} 個)", self.items.len()));
    for item in &self.items {
        list.push(item.clone());
    }
} else {
    list.push("【持有物品】(0 個)".to_string());
    list.push("未持有物品".to_string());
}
```

---

## 物品持久化

### 概述

玩家在遊戲中撿起的物品現在會被自動保存到檔案系統，遊戲啟動時會自動載入之前保存的物品。

### 存檔機制

#### 存檔位置
```
worlds/初始世界/persons/me.json
```

#### 存檔格式
Person 物件以 JSON 格式保存，包含以下信息：
```json
{
  "name": "冒險者",
  "description": "年輕的冒險者，渴望探索",
  "abilities": ["快速奔跑", "敏銳視覺"],
  "items": [
    "[物品] 蘋果 (食物)",
    "[物品] 鐵劍 (武器)",
    "[物品] 治療藥水 (消耗品)"
  ],
  "status": "正常",
  "x": 25,
  "y": 30
}
```

### 存檔觸發點

#### 1. 撿起物品時自動保存
**函數**: `handle_get()` (src/app.rs)

撿起物品時會立即保存角色信息：
```rust
// 撿起所有物品後
let _ = me.save(&person_dir, "me");

// 撿起指定物品後
let _ = me.save(&person_dir, "me");
```

**流程**:
```
用戶輸入: get
  ↓
handle_get() 執行
  ↓
me.add_item() 添加物品到背包
  ↓
me.save() 保存到 me.json
  ↓
遊戲繼續
```

#### 2. 移動時自動保存
**函數**: `handle_movement()` (src/app.rs)

移動到新位置時會保存位置信息：
```rust
me.move_to(new_x, new_y);
let _ = me.save(&person_dir, "me");
```

**保存內容**:
- 新的位置 (x, y)
- 當前持有的物品
- 其他角色信息

#### 3. 啟動時自動載入
**函數**: `main()` (src/main.rs)

遊戲啟動時自動載入之前的存檔：
```rust
if let Ok(loaded_me) = Person::load(&person_dir, "me") {
    me = loaded_me;
    output_manager.print("已載入角色: Me".to_string());
} else {
    let _ = me.save(&person_dir, "me");
    output_manager.print("已保存新角色: Me".to_string());
}
```

### 完整遊戲循環

```
1️⃣ 遊戲啟動
   └─ main() 載入 persons/me.json
   └─ 恢復上次的物品、位置、狀態

2️⃣ 探索地圖
   移動命令 (up/down/left/right)
   └─ handle_movement() 執行
   └─ me.move_to() 更新位置
   └─ me.save() 保存位置和物品

3️⃣ 查看物品
   命令: look
   └─ display_look() 顯示此處物品

4️⃣ 撿起物品
   命令: get [物品名]
   └─ handle_get() 執行
   └─ me.add_item() 添加到背包
   └─ me.save() 保存背包

5️⃣ 查看背包
   命令: show status
   └─ 顯示已撿起的物品列表

6️⃣ 退出遊戲
   命令: exit
   └─ Person 已保存（無需額外操作）
   └─ 下次啟動時自動載入

7️⃣ 下次啟動
   └─ 恢復所有之前撿起的物品
   └─ 恢復上次的位置
```

### 數據保存時機

| 動作 | 保存內容 | 觸發方式 | 檔案位置 |
|------|--------|--------|--------|
| 撿起物品 | items 列表 | `handle_get()` | persons/me.json |
| 移動 | x, y 座標 | `handle_movement()` | persons/me.json |
| 啟動 | 全部 | `main()` 載入 | persons/me.json |
| 退出 | 元數據和時間 | `handle_exit()` | world.json, time.json |

### 相關命令

| 命令 | 功能 | 是否觸發保存 |
|------|------|-----------|
| `get` | 撿起物品 | ✅ 立即保存 |
| `move` / 方向鍵 | 移動 | ✅ 立即保存 |
| `look` | 查看 | ❌ |
| `show status` | 查看背包 | ❌ |
| `exit` | 退出 | ✅ (其他數據) |

### 調試信息

#### 檢查物品是否保存
```bash
cat worlds/初始世界/persons/me.json | grep items
```

#### 查看完整的存檔內容
```bash
cat worlds/初始世界/persons/me.json
```

#### 清除存檔（新遊戲）
```bash
rm worlds/初始世界/persons/me.json
```

---

## 檔案結構

```
rataui_demo/
├── src/
│   ├── main.rs              # 程式入口、初始化
│   ├── app.rs               # 主迴圈、事件處理
│   ├── input.rs             # 輸入處理、命令解析
│   ├── output.rs            # 輸出管理器
│   ├── ui.rs                # UI元件渲染
│   ├── world.rs             # 遊戲世界、時間系統
│   ├── map.rs               # 地圖系統、物品初始化
│   ├── person.rs            # 人物系統、狀態顯示
│   ├── observable.rs        # Observable trait
│   ├── time_updatable.rs    # 時間事件系統
│   ├── item.rs              # 物品系統
│   ├── settings.rs          # 設定管理
│   └── bin/
│       └── genmap.rs        # 地圖生成工具
├── worlds/
│   └── 初始世界/
│       ├── world.json       # 世界元數據
│       ├── time.json        # 世界時間
│       ├── maps/            # 地圖存檔
│       └── persons/         # 人物存檔（含物品）
├── Cargo.toml               # 專案配置
└── UPDATE.md                # 更新日誌（本文件）
```

## 技術細節

### 時間管理
- 使用 `std::time::Instant` 追蹤時間間隔
- 每1秒檢查一次需要更新時間
- 每60秒顯示一次時間訊息
- 時間資訊可序列化為JSON，支援存檔

### 持久化設計
- 所有資料使用 serde 進行 JSON 序列化
- 各模組獨立的 save/load 方法
- 程式退出時自動保存
- Person 物品列表自動序列化

### 模組化設計
- 命令處理邏輯集中在 `input.rs`
- UI渲染邏輯集中在 `ui.rs` 和 `output.rs`
- 主迴圈事件處理在 `app.rs`
- 物品管理在 `map.rs` 和 `person.rs`

## 注意事項

- 遊戲時間會持續累積，即使遊戲關閉
- 小地圖大小會根據終端寬度自動調整
- 狀態面板顯示時，其他指令會關閉它（除了移動）
- 方向鍵對應的移動指令與minimap方向一致
- 物品會自動保存，無需手動保存
- 撿起物品時立即保存到檔案

