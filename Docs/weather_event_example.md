# 天氣事件系統範例

## 概述
此範例展示如何使用機率權重系統來創建動態天氣事件。

## 新增功能

### 1. 地圖屬性系統
地圖現在支援自定義屬性（properties），可以儲存任意鍵值對：
```rust
// 設定屬性
map.set_property("天氣".to_string(), "晴天".to_string());

// 獲取屬性
if let Some(weather) = map.get_property("天氣") {
    println!("當前天氣: {}", weather);
}
```

### 2. 隨機動作（RandomAction）
新的 `random_action` 事件動作類型，可以根據權重隨機執行不同的動作：

```json
{
  "type": "random_action",
  "actions": [
    {
      "weight": 0.3,
      "action": { "type": "message", "text": "晴天" }
    },
    {
      "weight": 0.2,
      "action": { "type": "message", "text": "雨天" }
    }
  ]
}
```

權重說明：
- `weight` 可以是任意正數
- 系統會自動計算總權重並按比例分配機率
- 例如權重 [0.3, 0.2, 0.5] 會產生機率 [30%, 20%, 50%]

### 3. 設定地圖屬性動作（SetMapProperty）
新的 `set_map_property` 動作可以修改地圖的屬性：

```json
{
  "type": "set_map_property",
  "map": "beginMap",
  "property": "天氣",
  "value": "晴天"
}
```

## 天氣事件範例

檔案位置：`worlds/beginWorld/events/weather_event.json`

這個事件會：
1. 每5分鐘檢查一次（`*/5 * * * *`）
2. 每次觸發時，會隨機選擇一種天氣（晴天、陰天、雨天、多雲、霧天）
3. 更新 beginMap 的「天氣」屬性
4. 顯示對應的天氣描述訊息

### 天氣機率分布
- 晴天：30%
- 陰天：25%
- 雨天：20%
- 多雲：15%
- 霧天：10%

### 冷卻時間
設定了 300 秒（5分鐘）的冷卻時間，確保天氣變化不會太頻繁。

## 進階應用

### 多個機率動作組合
一個事件可以包含多個 `random_action`，分別控制不同的隨機結果：

```json
{
  "actions": [
    {
      "type": "random_action",
      "actions": [
        { "weight": 0.5, "action": { "type": "set_map_property", "map": "beginMap", "property": "天氣", "value": "晴天" } },
        { "weight": 0.5, "action": { "type": "set_map_property", "map": "beginMap", "property": "天氣", "value": "雨天" } }
      ]
    },
    {
      "type": "random_action",
      "actions": [
        { "weight": 0.7, "action": { "type": "message", "text": "風很小" } },
        { "weight": 0.3, "action": { "type": "message", "text": "風很大" } }
      ]
    }
  ]
}
```

### 結合地圖屬性與條件
未來可以擴展事件系統，根據地圖屬性觸發不同事件：
- 雨天時增加蘑菇生成機率
- 晴天時 NPC 更活躍
- 霧天時降低視野範圍

## 測試

啟動遊戲後：
1. 等待5分鐘，觀察天氣變化訊息
2. 可以在程式碼中查詢地圖的天氣屬性確認是否正確更新
3. 長時間運行可以觀察各種天氣出現的頻率是否符合設定的機率

## 擴展建議

1. **季節系統**：根據遊戲時間的月份調整天氣機率
2. **天氣效果**：不同天氣影響移動速度、戰鬥能力等
3. **區域天氣**：不同地圖有不同的天氣模式（沙漠很少下雨，森林常起霧）
4. **天氣預報 NPC**：某些 NPC 可以告訴玩家未來的天氣
5. **極端天氣事件**：暴風雨、暴雪等罕見但影響大的天氣
