# 地圖和世界名稱更新記錄

## 更新時間
2025-12-24

## 更改摘要

將中文的世界和地圖名稱改為英文，以便更好的代碼可讀性和國際化支援。

## 具體更改

### 目錄和檔案重命名

1. **世界目錄**
   - `worlds/初始世界` → `worlds/beginWorld`

2. **地圖檔案** (在 `worlds/beginWorld/maps/` 下)
   - `初始之地.json` → `beginMap.json`
   - `森林.json` → `forest.json`
   - `山脈.json` → `mountain.json`
   - 沙漠.json - 保持不變
   - 洞穴.json - 保持不變

### 程式碼更新

#### src/person.rs
- `default_map()` 預設地圖：`初始之地` → `beginMap`
- `Person::new()` 預設地圖：`初始之地` → `beginMap`

#### src/world.rs
- 世界目錄：`worlds/初始世界` → `worlds/beginWorld`
- 世界名稱：`初始世界` → `beginWorld`
- 描述中的地圖名稱更新
- 預設地圖：`初始之地` → `beginMap`
- `initialize_maps()` 中的地圖配置更新

#### src/app.rs
- Header 顯示：`初始世界` → `beginWorld`

#### src/ffi.rs
- 預設地圖：`初始之地` → `beginMap`

#### src/event_loader.rs
- 事件範例中的地圖引用：`初始之地` → `beginMap`

### 資料檔案更新

#### worlds/beginWorld/world.json
```json
{
  "name": "beginWorld",
  "maps": [
    "beginMap",
    "forest",
    "洞穴",
    "沙漠",
    "mountain"
  ],
  "current_map": "beginMap"
}
```

#### worlds/beginWorld/persons/*.json
- 所有 NPC 和玩家檔案中的 `"map": "初始之地"` 更新為 `"map": "beginMap"`

## 名稱對照表

| 舊名稱 | 新名稱 | 類型 |
|--------|--------|------|
| 初始世界 | beginWorld | 世界 |
| 初始之地 | beginMap | 地圖 |
| 森林 | forest | 地圖 |
| 山脈 | mountain | 地圖 |
| 沙漠 | (保持不變) | 地圖 |
| 洞穴 | (保持不變) | 地圖 |

## 驗證結果

- ✅ 所有檔案和目錄已重命名
- ✅ 程式碼中的引用已更新
- ✅ 資料檔案已更新
- ✅ 編譯通過 (cargo build)
- ✅ Clippy 檢查通過

## 注意事項

1. **向後兼容性**：此更改會破壞舊存檔的兼容性，舊的 `worlds/初始世界` 需要手動遷移
2. **未來擴展**：建議後續新增的地圖也使用英文命名
3. **混合命名**：目前沙漠和洞穴保持中文，可視需要進一步統一

## 建議後續工作

- [ ] 考慮將 "沙漠" 改為 "desert"
- [ ] 考慮將 "洞穴" 改為 "cave"
- [ ] 更新相關文檔和教程
- [ ] 提供舊存檔遷移工具（如需要）
