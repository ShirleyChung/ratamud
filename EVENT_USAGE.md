# 事件系統使用指南

## 已實現的功能

### 1. 核心模組

- ✅ **event.rs** - 事件數據結構
- ✅ **event_scheduler.rs** - 事件調度器（時間觸發、Crontab 解析）
- ✅ **event_executor.rs** - 事件執行器
- ✅ **event_loader.rs** - 事件加載器

### 2. 支持的觸發器類型

| 類型 | 說明 | 範例 |
|------|------|------|
| `time` | 時間觸發（Crontab） | 每10分鐘、每小時 |
| `random` | 隨機觸發 | 30%機率 |
| `location` | 位置觸發 | 進入特定座標 |
| `condition` | 條件觸發 | 持有特定物品 |
| `manual` | 手動觸發 | 腳本調用 |

### 3. 支持的動作類型

| 動作 | 說明 |
|------|------|
| `spawn_npc` | 生成 NPC |
| `remove_npc` | 移除 NPC |
| `message` | 顯示訊息 |
| `dialogue` | NPC 對話 |
| `add_item` | 添加物品到地圖 |
| `remove_item` | 移除地圖物品 |
| `teleport` | 傳送玩家 |

## 事件腳本範例

### 範例 1: 定時事件（每5分鐘的商人到訪）

```json
{
  "id": "merchant_visit",
  "name": "商人到訪",
  "description": "旅行商人定期來到城鎮廣場",
  "trigger": {
    "type": "time",
    "schedule": "*/5 * * * *",
    "time_range": ["09:00:00", "18:00:00"]
  },
  "where": {
    "map": "初始之地"
  },
  "actions": [
    {
      "type": "message",
      "text": "一位商人來到了廣場"
    },
    {
      "type": "spawn_npc",
      "npc_id": "merchant_01",
      "position": [50, 50],
      "dialogue": "歡迎！看看我的商品吧！"
    }
  ],
  "state": {
    "repeatable": true,
    "cooldown": 0,
    "max_triggers": -1,
    "prerequisites": []
  }
}
```

### 範例 2: 隨機事件（寶藏出現）

```json
{
  "id": "random_treasure",
  "name": "神秘寶藏",
  "description": "隨機地點出現寶藏",
  "trigger": {
    "type": "time",
    "schedule": "*/5 * * * *",
    "random_chance": 0.3
  },
  "where": {
    "map": "初始之地"
  },
  "actions": [
    {
      "type": "add_item",
      "item": "神秘寶箱",
      "position": [30, 30]
    },
    {
      "type": "message",
      "text": "你感覺到附近有什麼特別的東西..."
    }
  ],
  "state": {
    "repeatable": true,
    "cooldown": 300,
    "max_triggers": -1,
    "prerequisites": []
  }
}
```

### 範例 3: 位置事件（發現神殿）

```json
{
  "id": "discover_shrine",
  "name": "發現神殿",
  "description": "玩家發現古老的神殿",
  "trigger": {
    "type": "location",
    "positions": [[25, 25]]
  },
  "actions": [
    {
      "type": "message",
      "text": "你發現了一座古老的神殿！"
    }
  ],
  "state": {
    "repeatable": false,
    "cooldown": 0,
    "max_triggers": 1,
    "prerequisites": []
  }
}
```

## Crontab 格式說明

```
分 時 日 月 星期
* * * * *
```

| 欄位 | 範圍 | 說明 |
|------|------|------|
| 分 | 0-59 | 分鐘 |
| 時 | 0-23 | 小時 |
| 日 | 1-31 | 日期（暫未實現） |
| 月 | 1-12 | 月份（暫未實現） |
| 星期 | 0-6 | 星期（暫未實現） |

### 特殊字符

- `*` - 任意值
- `*/N` - 每 N 單位
- `N-M` - 範圍
- `N` - 具體值

### 範例

| 表達式 | 說明 |
|--------|------|
| `*/10 * * * *` | 每 10 分鐘 |
| `0 * * * *` | 每小時整點 |
| `0 9-17 * * *` | 9:00-17:00 的整點 |
| `30 12 * * *` | 每天 12:30 |

## 檔案結構

```
worlds/初始世界/
├── events/
│   ├── time_based/          # 時間觸發事件
│   │   └── merchant_visit.json
│   ├── random/              # 隨機事件
│   │   └── treasure_spawn.json
│   ├── location/            # 位置觸發事件
│   │   └── discover_shrine.json
│   └── conditional/         # 條件事件
│       └── key_found.json
└── maps/
    └── 初始之地.json
```

## 下一步整合

要在遊戲中啟用事件系統，需要：

1. 在 `GameWorld` 中添加 `EventManager` 和 `EventScheduler`
2. 在遊戲主循環中調用 `event_scheduler.check_and_trigger()`
3. 在移動時檢查位置觸發事件
4. 將 `EventExecutor` 整合到事件觸發流程

## 未來擴展

- [ ] 支持更複雜的條件表達式
- [ ] NPC AI 整合
- [ ] 任務系統整合
- [ ] 對話系統
- [ ] 天氣系統
- [ ] 日月星期的完整支持
- [ ] 事件鏈和前置條件檢查
- [ ] 事件狀態持久化
