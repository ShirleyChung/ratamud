# RataMUD 文檔目錄

本目錄包含 RataMUD 專案的所有技術文檔。

## 📁 目錄結構

### Development/ - 開發規範與指南
- **CODE_RULES.md** - 程式碼規範與開發準則

### Features/ - 功能說明文檔
- **PERSON_DESCRIPTION.md** - 角色描述系統
- **PROXIMITY_WAIT_FEATURE.md** - 距離檢測與Wait指令功能

### Build/ - 建置與部署文檔
- **WINDOWS_BUILD.md** - Windows 平台建置指南
- **DIST_README.md** - 動態連結函式庫使用說明
- **CPP_EXAMPLES.md** - C/C++ 範例程式說明
- **BUILD_STATUS.md** - 建置狀態
- **SCONS_README.md** - SCons 建置系統說明

### Architecture/ - 架構設計文檔
- **REFACTOR_PLAN.md** - 架構重構計劃
- **REFACTOR_COMPLETE.md** - 重構完成報告
- **REFACTOR_VERIFY.md** - 重構驗證報告
- **OLD_CODE_REMOVAL.md** - 舊架構移除報告
- **CROSS_PLATFORM_ARCHITECTURE.md** - 跨平台架構設計
- **SHARED_ARCHITECTURE_PROGRESS.md** - 共享架構進度
- **ENGINE_DECOUPLING.md** - 引擎解耦設計
- **APP_RS_REFACTORING.md** - App.rs 重構文檔

### API/ - API 文檔
- **C_ABI_GUIDE.md** - C ABI 開發指南
- **C_ABI_README.md** - C API 快速入門
- **CALLBACK_USAGE.md** - Callback 使用說明
- **IOS_FRAMEWORK_README.md** - iOS Framework 文檔

### Bugfixes/ - 問題修復記錄
- **FIX_CHINESE_INPUT.md** - 中文輸入修復
- **FIX_CONTROL_ISSUE.md** - 控制問題修復
- **FIX_NPC_MAP_ISSUE.md** - NPC 地圖問題修復
- **ctrl_command_bugfix.md** - Ctrl 命令錯誤修復
- **ctrl_bugfix_summary.md** - Ctrl 錯誤修復總結
- **DIALOGUE_CONDITION_FIX.md** - 對話條件修復
- **NPC_LOADING_FIX.md** - NPC 載入修復
- **NPC_DUPLICATION_FIX.md** - NPC 重複問題修復
- **npc_interaction_freeze.md** - NPC 互動凍結修復
- **EVENT_FIX_REPORT.md** - 事件系統修復報告

### 其他重要文檔
- **INDEX.md** - 文檔索引
- **SUMMARY.md** - 專案總結
- **FINAL_SUMMARY.md** - 最終總結
- **UPDATE.md** - 更新日誌

## 📚 主題分類

### NPC 系統
- NPC_AI_IMPLEMENTATION.md
- NPC_AI_THREAD.md
- NPC_AI_THREAD_SUMMARY.md
- NPC_TALK_FEATURE.md
- NPC_TALK_IMPLEMENTATION.md
- NPC_TALK_QUICKSTART.md
- NPC_DIALOGUES_COMPLETE.md
- NPC_DIALOGUES_LIST.md
- README_DIALOGUE.md

### 對話系統
- RELATIONSHIP_SYSTEM.md
- interaction_menu_system.md
- SDL_SYNTAX.md

### 指令系統
- command_processing_analysis.md
- command_processing_summary.md
- repeat_command.md
- repeat_command_improvement.md
- repeat_command_error_handling.md
- repeat_command_summary.md
- give_command.md
- give_command_summary.md

### 事件系統
- EVENT_DRIVEN_RULE.md
- EVENT_SYSTEM_STATUS.md
- EVENT_THREAD_IMPLEMENTATION.md
- event_system_update_summary.md
- probability_event_system.md
- weather_event_example.md
- simple_weather_tutorial.md

### 代碼品質
- clippy_summary.md
- clippy_optimization.md
- RENAME_LOG.md
- STRATEGY_PATTERN_EXAMPLE.md

### 開發階段記錄
- STAGE1_COMPLETE.md
- STAGE2_COMPLETE.md
- IMPLEMENTATION_LOG.txt

## 🔍 快速查找

### 新手入門
1. 閱讀 [Development/CODE_RULES.md](Development/CODE_RULES.md) 了解開發規範
2. 閱讀 [Build/WINDOWS_BUILD.md](Build/WINDOWS_BUILD.md) 學習如何建置專案
3. 閱讀 [API/C_ABI_README.md](API/C_ABI_README.md) 了解 API 使用

### 功能開發
- 查看 Features/ 目錄下的功能說明文檔
- 參考 NPC 系統相關文檔進行 NPC 開發
- 參考對話系統文檔開發對話功能

### 架構理解
- 閱讀 Architecture/ 目錄下的架構設計文檔
- 查看重構相關文檔了解架構演進

### 問題排查
- 查看 Bugfixes/ 目錄下的修復記錄
- 參考類似問題的解決方案

## 📝 文檔維護

- 所有新增功能應在 Features/ 目錄添加說明文檔
- 架構變更應更新 Architecture/ 目錄的相關文檔
- 問題修復應記錄在 Bugfixes/ 目錄
- 保持 INDEX.md 和本 README.md 的更新
