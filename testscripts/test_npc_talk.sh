#!/bin/bash

# 測試 NPC 說話功能
echo "=== 测试 NPC 说话功能 ==="
echo ""
echo "步骤 1: 创建一个商人 NPC"
cat << 'TESTCMD' | timeout 10 cargo run --release 2>/dev/null
create npc m 商人甲 52 52
exit
TESTCMD

echo ""
echo "步骤 2: 设置商人台词和说话积极度为 100%"
cat << 'TESTCMD' | timeout 10 cargo run --release 2>/dev/null
setdialogue 商人甲 見面 哈囉！你好，來看看我的商品吧！
seteagerness 商人甲 100
exit
TESTCMD

echo ""
echo "步骤 3: 多次移动到商人位置，观察商人是否每次都说话（积极度100%）"
cat << 'TESTCMD' | timeout 15 cargo run --release 2>/dev/null
goto 52 52
goto 50 50
goto 52 52
goto 50 50
goto 52 52
exit
TESTCMD

echo ""
echo "步骤 4: 设置商人说话积极度为 50%"
cat << 'TESTCMD' | timeout 10 cargo run --release 2>/dev/null
seteagerness 商人甲 50
exit
TESTCMD

echo ""
echo "步骤 5: 多次移动到商人位置，观察 50% 说话概率"
cat << 'TESTCMD' | timeout 15 cargo run --release 2>/dev/null
goto 52 52
goto 50 50
goto 52 52
goto 50 50
goto 52 52
goto 50 50
goto 52 52
exit
TESTCMD

echo ""
echo "=== 测试完成 ==="

