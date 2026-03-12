# Windows 版戰鬥迴圈測試腳本
#
# 測試戰鬥迴圈功能

Write-Host "=== 測試戰鬥迴圈系統 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "測試場景:" -ForegroundColor Yellow
Write-Host "1. 玩家攻擊NPC，開始戰鬥"
Write-Host "2. 每回合NPC有50%機率反擊"
Write-Host "3. 玩家繼續攻擊直到戰鬥結束或逃跑"
Write-Host ""
Write-Host "預期結果:" -ForegroundColor Yellow
Write-Host "- 戰鬥開始後，每次玩家攻擊，NPC都可能反擊"
Write-Host "- NPC不會完全沒有行動"
Write-Host "- esc指令能正確結算經驗並退出戰鬥"
Write-Host ""
Write-Host "請手動測試:" -ForegroundColor Green
Write-Host "1. 執行 cargo run"
Write-Host "2. 使用 flyto 3,3 靠近NPC"
Write-Host "3. 使用 punch 或 kick 攻擊NPC"
Write-Host "4. 觀察NPC是否會反擊（約50%機率）"
Write-Host "5. 繼續攻擊數回合"
Write-Host "6. 使用 esc 逃離戰鬥"
Write-Host "7. 檢查是否獲得經驗值"
Write-Host ""
Write-Host "修改說明:" -ForegroundColor Cyan
Write-Host "- 戰鬥系統現在是回合制"
Write-Host "- 玩家每次攻擊後會觸發一個戰鬥回合"
Write-Host "- 在戰鬥回合中，所有NPC都有機會行動"
Write-Host "- NPC有50%機率執行戰鬥技能（punch或kick）"
Write-Host "- 回合結束時會減少所有參與者的技能冷卻"
Write-Host "- esc指令會給予一半經驗值並結束戰鬥"
