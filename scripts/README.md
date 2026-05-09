# Scripts

這裡放建置、測試、Codex resume 等輔助腳本。

這些腳本不是 RataMUD 的核心程式；主要程式是 `../src/` 裡的 Rust code。

常用腳本：

- `resume_codex.sh`：在本 repo resume 最近一次 Codex session
- `build_and_test.sh`：建置 Rust library 並跑 C++ callback 範例
- `build_frameworks.sh`：建置 iOS/macOS framework 產物
- `check_game.sh`：檢查遊戲資料狀態

其他 `test_*.sh` 多半是 Copilot 生成的測試輔助或手動測試流程。
