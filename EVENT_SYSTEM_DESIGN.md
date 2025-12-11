# 事件系統設計文件

## 概念

基於「人事時地物」原則的腳本化事件系統，支持時間觸發和隨機執行。

## 事件腳本結構

```json
{
  "id": "event_001",
  "name": "商人到訪",
  "description": "旅行商人來到城鎮",
  
  // 時間觸發條件（類似 crontab）
  "trigger": {
    "type": "time",  // time | random | location | condition
    "schedule": "*/10 * * * *",  // 每10分鐘
    "random_chance": 0.3,  // 30% 機率觸發（可選）
    "day_range": [1, 7],  // 遊戲第1-7天
    "time_range": ["09:00:00", "18:00:00"]  // 白天時段
  },
  
  // 人（角色條件）
  "who": {
    "player_present": true,  // 玩家是否在場
    "npcs": ["merchant_01"],  // 相關 NPC
    "player_level": {"min": 1, "max": 10}  // 玩家等級範圍
  },
  
  // 地（地點條件）
  "where": {
    "map": "初始之地",  // 地圖名稱
    "positions": [[10, 10], [11, 10]],  // 特定座標
    "area": {"x": [5, 15], "y": [5, 15]}  // 區域範圍
  },
  
  // 物（物品條件）
  "what": {
    "required_items": ["古老的鑰匙"],  // 玩家需擁有的物品
    "map_objects": ["寶箱"]  // 地圖上需存在的物品
  },
  
  // 事（事件執行內容）
  "actions": [
    {
      "type": "spawn_npc",
      "npc_id": "merchant_01",
      "position": [10, 10],
      "dialogue": "你好，旅行者！"
    },
    {
      "type": "message",
      "text": "一位商人出現在廣場上"
    },
    {
      "type": "add_item",
      "item": "神秘藥水",
      "position": [12, 12]
    }
  ],
  
  // 事件狀態
  "state": {
    "repeatable": true,  // 是否可重複觸發
    "cooldown": 600,  // 冷卻時間（秒）
    "max_triggers": -1,  // 最大觸發次數（-1 = 無限）
    "prerequisites": ["event_000"]  // 前置事件
  }
}
```

## 觸發器類型

1. **time** - 時間觸發（crontab 格式）
2. **random** - 隨機觸發
3. **location** - 進入特定位置觸發
4. **condition** - 條件觸發（物品、狀態等）
5. **manual** - 手動觸發（腳本調用）

## 動作類型

1. **spawn_npc** - 生成 NPC
2. **remove_npc** - 移除 NPC
3. **message** - 顯示訊息
4. **dialogue** - 觸發對話
5. **add_item** - 添加物品
6. **remove_item** - 移除物品
7. **change_weather** - 改變天氣
8. **teleport** - 傳送玩家
9. **quest** - 觸發任務
10. **script** - 執行自定義腳本

## 檔案結構

```
worlds/初始世界/
├── events/
│   ├── time_based/
│   │   ├── daily_merchant.json
│   │   └── night_patrol.json
│   ├── random/
│   │   ├── treasure_spawn.json
│   │   └── monster_attack.json
│   ├── location/
│   │   ├── enter_forest.json
│   │   └── discover_cave.json
│   └── conditional/
│       ├── key_found.json
│       └── quest_complete.json
└── maps/
    └── 初始之地.json
```

## Crontab 格式說明

```
分 時 日 月 星期
* * * * *

範例：
"0 * * * *"      - 每小時整點
"*/10 * * * *"   - 每10分鐘
"0 9-17 * * *"   - 每天 9:00-17:00 的整點
"0 12 * * 1-5"   - 週一到週五的中午12點
```

## 實現步驟

1. 創建事件數據結構
2. 實現事件調度器（Event Scheduler）
3. 實現事件條件檢查器
4. 實現事件執行器
5. 創建事件腳本加載器
6. 整合到遊戲主循環
