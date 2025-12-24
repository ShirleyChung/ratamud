# NPC 關係系統使用指南

## 📋 概述

關係系統允許 NPC 根據與玩家的好感度展示不同的對話和行為。這為遊戲增加了更豐富的互動體驗。

## 🎯 核心功能

### 1. 好感度系統
- **範圍**: -100 到 100
- **等級劃分**:
  - 摯友: >= 70
  - 好友: >= 30
  - 普通: >= 0
  - 冷淡: >= -30
  - 敵對: < -30

### 2. NPC 屬性

每個 NPC 現在擁有以下關係相關屬性：

```json
{
  "relationship": 0,           // 好感度 (-100 到 100)
  "dialogue_state": "初見",    // 當前對話狀態
  "met_player": false,         // 是否見過玩家
  "interaction_count": 0       // 互動次數
}
```

## 🎮 指令使用

### 基礎指令

#### 1. 設置好感度
```bash
setrelationship <NPC> <好感度>
setrel <NPC> <好感度>           # 簡寫

# 範例
setrel 商人 50                  # 設置商人好感度為 50
```

#### 2. 改變好感度
```bash
changerelationship <NPC> <變化量>
changerel <NPC> <變化量>        # 簡寫
addrel <NPC> <變化量>           # 別名

# 範例
changerel 商人 10               # 提升商人好感度 10 點
changerel 商人 -5               # 降低商人好感度 5 點
```

#### 3. 與 NPC 對話
```bash
talk <NPC>
speak <NPC>                     # 別名

# 範例
talk 商人                       # 與商人對話
```

**對話規則**:
- NPC 必須在同一地圖
- 距離不超過 3 格
- 首次對話會自動標記為已見面，並獲得 +5 好感度
- 每次對話會增加互動計數
- 成功對話後會獲得 +1 好感度

#### 4. 查看 NPC 信息
```bash
check <NPC>
inspect <NPC>                   # 別名
examine <NPC>                   # 別名

# 範例
check 商人                      # 查看商人詳細資訊，包含關係狀態
```

## 💬 動態對話系統

### 設置多層級對話

對話可以根據好感度等級自動選擇：

```bash
# 設置基礎對話
setdialogue 商人 對話 你好！

# 設置不同好感度等級的對話
setdialogue 商人 對話:敵對 哼！滾開！
setdialogue 商人 對話:冷淡 你要幹嘛...
setdialogue 商人 對話:普通 你好，需要什麼嗎？
setdialogue 商人 對話:好友 嘿！老朋友！
setdialogue 商人 對話:摯友 我最好的朋友！有什麼需要儘管說！
```

### 對話選擇邏輯

系統會按以下優先級選擇對話：
1. **帶狀態的對話**: `對話:初見`
2. **帶好感度等級的對話**: `對話:摯友`、`對話:好友` 等
3. **基礎對話**: `對話`

## 📝 完整示例

### 創建一個有深度的 NPC

```bash
# 1. 創建 NPC
create npc m 藥店老闆

# 2. 設置不同場景的對話
# 見面時的對話
setdialogue 藥店老闆 見面:初見 咦？新面孔？我是這裡的藥店老闆。
setdialogue 藥店老闆 見面:普通 又見面了。
setdialogue 藥店老闆 見面:好友 歡迎回來！今天身體如何？
setdialogue 藥店老闆 見面:摯友 老朋友！我一直在等你！

# 對話時的對話
setdialogue 藥店老闆 對話:冷淡 我很忙，有事就說。
setdialogue 藥店老闆 對話:普通 需要什麼藥嗎？
setdialogue 藥店老闆 對話:好友 看你氣色不錯！來聊聊天吧。
setdialogue 藥店老闆 對話:摯友 我這裡有個祕方，只給你看！

# 3. 設置初始好感度
setrel 藥店老闆 0

# 4. 測試互動
talk 藥店老闆                  # 首次對話，+5 好感度
changerel 藥店老闆 20           # 完成任務，提升好感度
talk 藥店老闆                  # 再次對話，對話內容會改變
check 藥店老闆                 # 查看當前關係狀態
```

### 建立劇情發展

```bash
# 劇情開始：初見
setrel 神秘旅者 -10
setdialogue 神秘旅者 對話:冷淡 ...（沉默不語）
talk 神秘旅者

# 劇情發展：幫助後
changerel 神秘旅者 30
setdialogue 神秘旅者 對話:普通 謝謝你的幫助...我叫艾倫。
talk 神秘旅者

# 劇情高潮：成為好友
changerel 神秘旅者 50
setdialogue 神秘旅者 對話:摯友 你是我唯一信任的人。我告訴你一個秘密...
talk 神秘旅者
```

## 🔧 技術細節

### Person 結構新增欄位

```rust
pub struct Person {
    // ... 原有欄位 ...
    pub relationship: i32,       // 好感度
    pub dialogue_state: String,  // 對話狀態
    pub met_player: bool,        // 是否見過玩家
    pub interaction_count: u32,  // 互動次數
}
```

### 主要方法

```rust
// 根據好感度選擇對話
pub fn get_context_dialogue(&self, scene: &str) -> Option<String>

// 改變好感度（會自動更新對話狀態）
pub fn change_relationship(&mut self, delta: i32)

// 標記為已見過玩家（首次自動 +5 好感度）
pub fn mark_met_player(&mut self)

// 增加互動次數
pub fn increment_interaction(&mut self)

// 獲取關係等級描述
pub fn get_relationship_description(&self) -> String
```

## 🎨 使用技巧

### 1. 漸進式關係發展
```bash
# 從冷淡開始
setrel NPC名 -20

# 每次完成小任務
changerel NPC名 5

# 完成主線任務
changerel NPC名 30
```

### 2. 關鍵對話點
在特定好感度設置特殊對話：
- 30 分：NPC 開始信任，透露基本信息
- 50 分：NPC 視為好友，願意幫助
- 70 分：NPC 完全信任，透露秘密

### 3. 負面關係
```bash
# 做錯事導致關係惡化
changerel 守衛 -30

# 不同的負面對話
setdialogue 守衛 對話:敵對 我記住你了！滾出我的視線！
```

## 📊 狀態顯示

使用 `check` 命令查看完整的關係信息：

```
┌─ 商人 ─────────────────
│ 精明的商人，販售各種物品
│ 位置: (66, 63) @ 初始之地
│ 狀態: 正常
├─────────────────────────
│ HP: 100/100  力量: 100
│ MP: 100/100  知識: 100
│ 年齡: 0秒   交誼: 100
├─────────────────────────
│ 關係: 好友 (50)
│ 互動次數: 5
└─────────────────────────
```

## 🚀 快速測試

使用測試腳本：
```bash
./test_relationship.sh
```

## 💡 下一步

關係系統已完成！接下來可以實現：
- **階段二**: 任務系統（與關係掛鉤）
- **階段三**: 基於關係的事件觸發
- **階段四**: 戰鬥系統（關係影響戰鬥意願）

---

**版本**: 1.0.0  
**最後更新**: 2025-12-20
