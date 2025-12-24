#!/bin/bash

# NPC 說話功能演示
echo "╔════════════════════════════════════════════╗"
echo "║     NPC 說話功能演示 - 互動式測試         ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "此腳本將示範如何設置 NPC 台詞和說話積極度"
echo ""

read -p "按 Enter 鍵開始演示..."

echo ""
echo "▶ 步驟 1: 創建一個商人 NPC"
echo "  命令: create npc m 商人甲"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "創建|商人|位置"
create npc m 商人甲
look
exit
TESTCMD

read -p "按 Enter 繼續..."

echo ""
echo "▶ 步驟 2: 設置商人的「見面」台詞"
echo "  命令: setdialogue 商人甲 見面 哈囉！你好，來看看我的商品吧！"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "設置|台詞"
setdialogue 商人甲 見面 哈囉！你好，來看看我的商品吧！
exit
TESTCMD

read -p "按 Enter 繼續..."

echo ""
echo "▶ 步驟 3: 設置說話積極度為 100% (一定會說話)"
echo "  命令: seteagerness 商人甲 100"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "設置|積極度"
seteagerness 商人甲 100
exit
TESTCMD

read -p "按 Enter 繼續..."

echo ""
echo "▶ 步驟 4: 測試 - 移動到商人位置，觀察商人說話"
echo "  商人每次都應該會打招呼（積極度 100%）"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "商人|說|移動|人物"
look
goto 52 52
goto 50 50
goto 52 52
exit
TESTCMD

read -p "按 Enter 繼續..."

echo ""
echo "▶ 步驟 5: 設置說話積極度為 30% (低機率說話)"
echo "  命令: seteagerness 商人甲 30"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "設置|積極度"
seteagerness 商人甲 30
exit
TESTCMD

read -p "按 Enter 繼續最後測試..."

echo ""
echo "▶ 步驟 6: 多次移動測試 30% 說話機率"
echo "  商人大約只有 30% 機率會說話"
echo ""

cat << 'TESTCMD' | cargo run --release 2>/dev/null | grep -E "商人|說|位置|人物"
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
echo "╔════════════════════════════════════════════╗"
echo "║            演示完成！                      ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "您可以自行測試："
echo "  1. 使用 'create npc m 商人名稱' 創建商人"
echo "  2. 使用 'setdialogue 商人名稱 見面 台詞內容' 設置台詞"
echo "  3. 使用 'seteagerness 商人名稱 數值' 設置積極度 (0-100)"
echo "  4. 移動到商人位置觀察效果"
echo ""
