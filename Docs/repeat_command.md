# 重複命令功能 (Repeat Command)

## 功能說明

新增 `re` 或 `repeat` 命令，可以快速重複執行上一次輸入的命令。

## 使用方式

### 基本用法

```
> look
查看當前位置...

> re
查看當前位置...（重複執行 look）
```

### 支援的別名

- `re` - 簡短版本
- `repeat` - 完整版本

## 使用範例

### 範例 1: 重複移動
```
> up
向北移動...

> re
向北移動...

> re
向北移動...
```

### 範例 2: 重複撿拾物品
```
> get 蘋果
撿起了 蘋果

> re
撿起了 蘋果

> re
撿起了 蘋果
```

### 範例 3: 重複查看狀態
```
> status
顯示玩家狀態...

> re
顯示玩家狀態...
```

### 範例 4: 重複與 NPC 對話
```
> talk 商人 閒聊
商人: "歡迎光臨！"

> re
商人: "歡迎光臨！"
```

## 特殊說明

### 只記錄成功的命令
錯誤的命令不會被記錄，避免重複執行錯誤：

```
> wrongcommand
錯誤: No command provided

> re
（執行上一個成功的命令，不是 wrongcommand）
```

### 不會重複自身
`re` 命令本身不會被記錄為可重複的命令，避免無限循環：

```
> look
查看當前位置...

> re
查看當前位置...（重複 look）

> re
查看當前位置...（仍然重複 look，不是重複 re）
```

### 無可重複命令時
如果還沒有執行過任何命令就使用 `re`：

```
> re
錯誤: 沒有可重複的命令
```

### 跨命令重複
即使中間執行了 `re`，仍然會記住最後一個非 `re` 的命令：

```
> get 蘋果
撿起了 蘋果

> status
顯示狀態...

> re
顯示狀態...（重複 status）

> re
顯示狀態...（仍然重複 status）
```

## 實作細節

### 程式碼變更

1. **InputHandler 結構體** (`src/input.rs`)
   ```rust
   pub struct InputHandler {
       pub input: String,
       pub buffer: Vec<String>,
       pub last_command: Option<String>,  // 新增：儲存上一次命令
   }
   ```

2. **命令記錄** (`src/input.rs`)
   ```rust
   KeyCode::Enter => {
       if !self.input.is_empty() {
           let result = self.parse_input(self.input.clone());
           // 只儲存成功的命令（非錯誤、非重複命令）
           if self.input != "re" && self.input != "repeat" {
               // 檢查結果是否為錯誤
               if !matches!(result, CommandResult::Error(_)) {
                   self.last_command = Some(self.input.clone());
               }
           }
           self.input.clear();
           return Some(result);
       }
   }
   ```

3. **命令處理** (`src/input.rs`)
   ```rust
   match parts[0] {
       "re" | "repeat" => {
           // 重複上一次的命令
           if let Some(ref last_cmd) = self.last_command {
               return self.handle_command(last_cmd.clone());
           } else {
               CommandResult::Error("沒有可重複的命令".to_string())
           }
       },
       // ... 其他命令
   }
   ```

## 使用技巧

### 快速採集資源
```
> get 木材
撿起了 木材 x1

> re
撿起了 木材 x1

> re
撿起了 木材 x1
```

### 快速探索
```
> look
查看周圍環境...

> up
向北移動...

> re
向北移動...

> re
向北移動...

> look
查看周圍環境...（到達新位置後查看）
```

### 重複交易
```
> buy 商人 治療藥水 5
購買了 5 個治療藥水

> re
購買了 5 個治療藥水

> re
購買了 5 個治療藥水
```

## 未來可能的擴展

1. **命令歷史記錄**
   - 支援多條命令歷史（類似 shell 的上下鍵）
   - `re 2` - 重複倒數第二個命令
   - `re 3` - 重複倒數第三個命令

2. **命令宏**
   - `macro add mining "get 礦石\nre\nre\nre"`
   - 將多個命令組合成一個宏

3. **條件重複**
   - `re while 背包未滿` - 持續重複直到背包滿
   - `re until hp > 80` - 持續重複直到 HP 大於 80

4. **顯示上次命令**
   - `lastcmd` - 顯示上一次執行的命令是什麼

## 總結

`re` 命令是一個簡單但實用的功能，可以減少重複輸入相同命令的麻煩，特別適合：
- 重複移動
- 連續採集物品
- 反覆查看狀態或環境
- 批量交易

透過這個功能，玩家可以更高效地進行遊戲操作。

---

**更新日期**: 2025-12-25
**版本**: 1.0
**狀態**: ✅ 已實作並可用
