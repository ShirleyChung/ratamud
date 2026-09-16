# iOS Host 接入指南

iOS 版跟終端版共用同一份 Rust core（`src/` 底下的 `world` / `command_executor` /
`command_interact` / `combat` / `panel_render`）。host 只負責輸入、顯示與生命週期，
**不要**在 Swift 這邊重做遊戲規則。

## 一、啟動流程（順序很重要）

```swift
// 1) 註冊 callback（要在 init 之前，否則會漏掉初始化期間的輸出）
ratamud_register_output_callback(onOutput)   // MAIN / LOG / STATUS / SIDE
ratamud_register_state_callback(onState)     // 狀態列用的 JSON
ratamud_register_panel_callback(onPanel)     // MAP / MAP_JSON / MINIMAP / INVENTORY / STATUS / TRADE
ratamud_register_event_callback(onEvent)     // 世界事件觸發通知

// 2) 指定可寫資料目錄 + 從 app bundle 安裝世界資料 + 初始化
let docs = FileManager.default
    .urls(for: .documentDirectory, in: .userDomainMask)[0].path
guard ratamud_init_game_with_dir(docs, Bundle.main.resourcePath) == 0 else { /* 失敗 */ }

// 3) 告訴引擎畫面放得下多大的地圖
ratamud_set_map_view_size(41, 21)
```

### 為什麼一定要設資料目錄

引擎預設用相對路徑 `worlds/beginWorld`。iOS 的行程工作目錄是唯讀的，也不是
app bundle 的位置，所以 `create_dir_all` / `fs::write` 一定失敗——地圖載不進來、
狀態也存不起來。`ratamud_set_data_dir()` 把所有 `worlds/...` 重新指到可寫目錄。

`ratamud_seed_data_dir()`（`init_game_with_dir` 內部會呼叫）把 bundle 裡的
`worlds/` 複製到可寫目錄，**不覆蓋**已存在的檔案，所以每次啟動都呼叫也不會蓋掉存檔。

> Xcode 設定：把專案根目錄的 `worlds` 資料夾以 **folder reference**（藍色資料夾）
> 加入 target 的 Copy Bundle Resources，`Bundle.main.resourcePath/worlds` 才會存在。

## 二、主迴圈

```swift
// 每秒一次就夠了
Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
    ratamud_tick()
}
```

`ratamud_tick()` 一次做完終端版主迴圈每幀做的事：

1. 推進世界時間
2. 更新玩家自己的時間狀態（飢餓扣 HP、睡眠回 MP…）
3. 檢查並執行世界事件
4. 驅動 NPC AI（每 2 tick 一次：移動、撿物、對話、靠近/離開通知）
5. 推進戰鬥回合（戰鬥中每 3 秒一回合，跟終端版一致）
6. 刷新 host 目前開啟的面板

## 三、存檔

* 一般指令只寫小檔案（角色、時間、世界中繼資料），約幾十 KB。
* 地圖檔一張約 2.4MB、五張接近 10MB，所以改成**只寫有變動的地圖**，而且最多
  每 30 秒寫一次。
* app 要進背景時務必呼叫一次完整存檔：

```swift
.onChange(of: scenePhase) { phase in
    if phase != .active { ratamud_save() }
}
```

## 四、面板

| panel_type   | 內容                                             |
|--------------|--------------------------------------------------|
| `MAP`        | 以玩家為中心的 ASCII 視窗（預設 41x21）          |
| `MAP_JSON`   | 同一個視窗的結構化 JSON，適合自己畫格子          |
| `MINIMAP`    | 21x11 的小地圖                                   |
| `INVENTORY`  | 背包 + 金幣 + HP/MP                              |
| `STATUS`     | 角色詳細狀態                                     |
| `TRADE`      | 交易清單（買價 / 賣價）                          |

`ratamud_set_active_panel("MAP")` 之後，世界一有變動（NPC 移動、事件、戰鬥）
引擎就會自動重新渲染並透過 panel callback 推回，host 不必輪詢。
關閉面板時傳空字串。

地圖本身是 100x100；**不要**一次全部取回，那是一萬個字元。用
`ratamud_set_map_view_size()` 指定畫面放得下的大小即可。

## 五、指令

所有遊戲行為都走 `ratamud_input_command()`，字串格式與終端版完全相同：

```
look / l [npc]      up / down / left / right
get / drop / use    use <item> on <npc>
talk <npc> <話題>   wait [npc]      party <npc>     disband
trade <npc>         buy <npc> <item> [n]            sell <npc> <item> [n]
quest list / active / available / completed / info|start|complete|abandon <id>
punch [目標]        kick [目標]     escape
npcs                show map        show minimap
```

交易另外有直接的 API（會自動刷新 TRADE/INVENTORY 面板）：
`ratamud_trade_buy()` / `ratamud_trade_sell()`，`npc` 傳空字串表示自動偵測當前格的商人。

## 六、state callback JSON

```json
{
  "world": "beginWorld", "current_map": "beginMap",
  "time": "Day 1 09:31:07", "day": 1, "hour": 9, "minute": 31,
  "player": { "id": "me", "name": "創造者", "x": 50, "y": 50, "map": "beginMap",
              "hp": 100, "max_hp": 100, "mp": 80, "max_mp": 100,
              "gold": 1200, "status": "精力充沛",
              "is_sleeping": false, "combat_exp": 12 },
  "combat": { "in_combat": true, "round": 3, "participants": ["me", "merchant"] },
  "interaction": { "kind": "trading", "npc": "merchant" }
}
```

## 七、注意事項

* **NPC 用 ID，不要用顯示名稱當 key。** 世界裡有同名 NPC（`merchant` 和 `商人`
  的 `name` 都是「商人」）。`MAP_JSON` 的 `npcs[].id` 就是可以安全傳給指令與
  交易 API 的鍵值。
* 所有 FFI 函式共用一把世界鎖，可以從任何執行緒呼叫，但同一時間只有一個會跑；
  `ratamud_tick()` 建議固定在同一條（例如主）執行緒上跑。
* **callback 裡不要再呼叫 `ratamud_*`**（例如在 output callback 裡呼叫
  `ratamud_request_map()`）。callback 是在引擎持有世界鎖的狀態下被叫的，
  再進來一次會死鎖。正確做法是把內容丟進 Swift 的 state，由 SwiftUI 自己更新。
* `ratamud_get_data_dir()` 回傳的字串要用 `ratamud_free_string()` 釋放。
