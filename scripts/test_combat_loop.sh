#!/bin/bash

# 測試戰鬥迴圈功能
# 此腳本測試：
# 1. NPC在戰鬥中會自動行動
# 2. 每回合NPC隨機執行戰鬥指令
# 3. esc指令會結算並離開戰鬥

echo "=== 測試戰鬥迴圈系統 ==="
echo ""
echo "測試場景："
echo "1. 玩家攻擊NPC，開始戰鬥"
echo "2. 每回合NPC有50%機率反擊"
echo "3. 玩家繼續攻擊直到戰鬥結束或逃跑"
echo ""
echo "預期結果："
echo "- 戰鬥開始後，每次玩家攻擊，NPC都可能反擊"
echo "- NPC不會完全沒有行動"
echo "- esc指令能正確結算經驗並退出戰鬥"
echo ""
echo "請手動測試："
echo "1. 執行 cargo run"
echo "2. 使用 flyto 3,3 靠近NPC"
echo "3. 使用 punch 或 kick 攻擊NPC"
echo "4. 觀察NPC是否會反擊（約50%機率）"
echo "5. 繼續攻擊數回合"
echo "6. 使用 esc 逃離戰鬥"
echo "7. 檢查是否獲得經驗值"
echo ""
echo "修改說明："
echo "- 戰鬥系統現在是回合制"
echo "- 玩家每次攻擊後會觸發一個戰鬥回合"
echo "- 在戰鬥回合中，所有NPC都有機會行動"
echo "- NPC有50%機率執行戰鬥技能（punch或kick）"
echo "- 回合結束時會減少所有參與者的技能冷卻"
echo "- esc指令會給予一半經驗值並結束戰鬥"
