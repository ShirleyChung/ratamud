# NPC 說話功能 - 快速開始

## 🎯 功能簡介

此功能讓 NPC 可以在玩家接近時說話。你可以：
- 為 NPC 設置不同場景的台詞
- 調整 NPC 的說話積極度（0-100%）
- 商人預設積極度為 100%，一定會打招呼

## 🚀 快速開始

### 1. 啟動遊戲
```bash
cargo run --release --bin main
```

### 2. 創建一個商人 NPC
```
create npc m 張商人
```

### 3. 設置打招呼台詞
```
setdialogue 張商人 見面 哈囉！你好，來看看我的商品吧！
```

### 4. 設置說話積極度（可選，預設已經是 100）
```
seteagerness 張商人 100
```

### 5. 移動到商人位置並觀察
```
look
```
你會看到商人說：「哈囉！你好，來看看我的商品吧！」

## 📝 命令說明

### setdialogue（設置台詞）
**完整命令：** `setdialogue <NPC名稱> <場景> <台詞內容>`
**簡寫：** `setdia <NPC名稱> <場景> <台詞內容>`

範例：
```
setdialogue 商人 見面 歡迎！我有最好的商品
setdia 醫生 見面 你受傷了嗎？我可以幫你
```

### seteagerness（設置積極度）
**完整命令：** `seteagerness <NPC名稱> <積極度0-100>`
**簡寫：** `setea <NPC名稱> <積極度0-100>`

範例：
```
seteagerness 商人 100   # 100% 一定會說話
seteagerness 路人 50    # 50% 機率說話
seteagerness 害羞者 10  # 10% 機率說話
seteagerness 沉默者 0   # 完全不會說話
```

## 🎮 完整演示

```bash
# 啟動遊戲
cargo run --release --bin main

# 在遊戲中輸入以下命令：
create npc m 李老闆
setdialogue 李老闆 見面 哦！有客人來了！快來看看我的好貨！
seteagerness 李老闆 100

# 移動離開再回來
goto 52 52
goto 50 50
goto 52 52

# 每次回到李老闆身邊，他都會打招呼！
```

## 🎨 實際應用場景

### 熱情的商人（100% 積極度）
```
create npc m 熱情商人
setdialogue 熱情商人 見面 歡迎歡迎！今天有特價優惠喔！
seteagerness 熱情商人 100
```

### 普通路人（50% 積極度）
```
create npc m 路人甲
setdialogue 路人甲 見面 嗨...
seteagerness 路人甲 50
```

### 害羞的小孩（20% 積極度）
```
create npc m 小明
setdialogue 小明 見面 呃...你好...
seteagerness 小明 20
```

## 💡 提示

1. **積極度說明：**
   - 100 = 每次都會說話
   - 50 = 大約一半機會說話
   - 0 = 永遠不會說話

2. **台詞保存：** 設置的台詞和積極度會自動保存到 NPC 的 JSON 文件中

3. **場景擴展：** 目前支援「見面」場景，未來可以加入更多場景（告別、交易、戰鬥等）

## 🐛 故障排除

**Q: NPC 不說話？**
- 檢查是否設置了「見面」場景的台詞
- 檢查積極度是否大於 0
- 確認你真的移動到了 NPC 所在位置（使用 `look` 命令確認）

**Q: 想讓 NPC 完全沉默？**
```
seteagerness <NPC名稱> 0
```

**Q: 想查看 NPC 信息？**
```
look <NPC名稱>
```

## 📚 更多信息

詳細的技術文檔請參考：[NPC_TALK_FEATURE.md](./NPC_TALK_FEATURE.md)
