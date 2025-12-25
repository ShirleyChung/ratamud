# 重複命令功能 - 錯誤處理範例

## 改進說明

`re` 命令現在會智慧地過濾錯誤命令，只記錄成功執行的命令。

## 範例對比

### ❌ 改進前（會記錄錯誤命令）

```
> look
查看當前位置...

> wrongcmd
錯誤: No command provided

> re
錯誤: No command provided（重複了錯誤命令 wrongcmd）
```

### ✅ 改進後（不記錄錯誤命令）

```
> look
查看當前位置...

> wrongcmd
錯誤: No command provided

> re
查看當前位置...（重複的是 look，跳過了錯誤的 wrongcmd）
```

## 實際使用場景

### 場景 1: 打字錯誤

```
> get 蘋果
撿起了 蘋果

> gett 蘋果
錯誤: No command provided

> re
撿起了 蘋果（重複正確的 get 命令，忽略錯誤的 gett）
```

### 場景 2: 參數錯誤

```
> buy 商人 藥水 5
購買了 5 個藥水

> buy
錯誤: Usage: buy <npc> <item> [quantity]

> re
購買了 5 個藥水（重複完整的 buy 命令）
```

### 場景 3: 目標不存在

```
> talk 村長 問候
村長: "你好！"

> talk 不存在的NPC 閒聊
錯誤: 找不到該 NPC

> re
村長: "你好！"（重複與村長的對話）
```

### 場景 4: 連續操作中的錯誤

```
> up
向北移動...

> up
向北移動...

> up
錯誤: 無法向該方向移動（遇到牆壁）

> left
向西移動...

> re
向西移動...（重複的是 left，跳過了失敗的 up）
```

## 技術實作

### 核心邏輯

```rust
KeyCode::Enter => {
    if !self.input.is_empty() {
        let result = self.parse_input(self.input.clone());
        
        // 只儲存成功的命令
        if self.input != "re" && self.input != "repeat" {
            if !matches!(result, CommandResult::Error(_)) {
                self.last_command = Some(self.input.clone());
            }
        }
        
        self.input.clear();
        return Some(result);
    }
}
```

### 判斷邏輯

1. **檢查命令類型**: 不是 `re` 或 `repeat`
2. **檢查執行結果**: 使用 `matches!` 判斷不是 `CommandResult::Error`
3. **儲存命令**: 只有通過以上兩個檢查才儲存

## 好處

### 1. 避免重複錯誤
用戶不會因為一次打字錯誤而持續重複錯誤命令。

### 2. 提升使用體驗
即使中間有錯誤輸入，`re` 仍然能執行最後一個成功的命令。

### 3. 符合直覺
用戶期望 `re` 重複的是「有效的操作」，而不是錯誤。

### 4. 減少挫折感
不會因為一次失誤而需要重新輸入整個命令。

## 邊界情況處理

### 情況 1: 從未成功執行過命令

```
> wrongcmd
錯誤: No command provided

> re
錯誤: 沒有可重複的命令
```

### 情況 2: 所有命令都失敗

```
> wrongcmd1
錯誤: No command provided

> wrongcmd2
錯誤: No command provided

> re
錯誤: 沒有可重複的命令
```

### 情況 3: 成功命令後接連失敗

```
> look
查看當前位置...

> error1
錯誤: No command provided

> error2
錯誤: No command provided

> re
查看當前位置...（仍然記得最後一個成功的命令）
```

## 測試建議

### 手動測試步驟

1. 執行一個正確的命令（如 `look`）
2. 執行一個錯誤的命令（如 `wrongcmd`）
3. 執行 `re`
4. 驗證重複的是第 1 步的命令，不是第 2 步

### 預期結果

- ✅ `re` 應該重複 `look`
- ✅ 不應該重複 `wrongcmd`
- ✅ 不應該顯示「沒有可重複的命令」

## 總結

這個改進讓 `re` 命令更加智慧和實用：

- **更安全**: 不會重複錯誤命令
- **更智慧**: 自動過濾失敗的操作
- **更友善**: 符合用戶的直覺預期

用戶可以放心使用 `re`，即使中間有打字錯誤或操作失敗，系統都會自動跳過這些錯誤，重複最後一個成功的命令。

---

**更新日期**: 2025-12-25  
**版本**: 1.1  
**改進**: 新增錯誤命令過濾
