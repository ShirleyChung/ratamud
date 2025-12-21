#!/bin/bash

# 測試所有 NPC 的台詞
echo "╔════════════════════════════════════════════╗"
echo "║     測試所有 NPC 的說話功能               ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# 列出所有已添加台詞的 NPC
echo "已為以下 NPC 添加台詞："
echo ""
echo "【商人類 - 說話積極度 100%】"
echo "  • 商人 - 歡迎光臨！我這裡有最好的商品，快來看看吧！"
echo "  • 李老闆 - 哎呀！有客人來了！今天有特價優惠喔！"
echo "  • merchant - Welcome! I have the finest goods in town!"
echo "  • Boss_Jenson - 啊，是你啊！最近生意還不錯，要不要看看我的新貨？(95%)"
echo "  • 小鹿 - 你好～我是小鹿，正在這裡擺攤呢！(90%)"
echo ""
echo "【醫生類 - 說話積極度 80-85%】"
echo "  • 醫生 - 你好，有什麼不舒服的嗎？我可以幫你檢查一下。(80%)"
echo "  • 櫻花 - 你好呀！需要醫療協助嗎？(85%)"
echo ""
echo "【農夫類 - 說話積極度 70%】"
echo "  • wang - 嗨，今年的收成不錯呢！你也是來看農田的嗎？"
echo ""
echo "【工人類 - 說話積極度 60-75%】"
echo "  • 工人 - 嘿！工作辛苦了！這附近有什麼需要修的嗎？(60%)"
echo "  • 阿猛 - 喲！有什麼活要幹嗎？我可是很能幹的！(75%)"
echo ""
echo "【工程師類 - 說話積極度 65%】"
echo "  • Steve - Hi there! I'm working on some interesting projects. (65%)"
echo ""

read -p "按 Enter 開始實際測試..."
echo ""

echo "▶ 測試 1: 訪問商人李老闆（位置: 51, 50）"
echo "  預期：100% 會打招呼"
cat << 'TESTCMD' | timeout 10 cargo run --release --bin main 2>/dev/null | grep -E "李老闆|說|打招呼|客人|優惠" | head -5
goto 51 50
exit
TESTCMD

echo ""
read -p "按 Enter 繼續下一個測試..."
echo ""

echo "▶ 測試 2: 訪問醫生櫻花（位置: 49, 52）"
echo "  預期：85% 會打招呼"
cat << 'TESTCMD' | timeout 10 cargo run --release --bin main 2>/dev/null | grep -E "櫻花|說|醫療|協助" | head -5
goto 49 52
exit
TESTCMD

echo ""
read -p "按 Enter 繼續下一個測試..."
echo ""

echo "▶ 測試 3: 訪問農夫 wang（位置: 48, 51）"
echo "  預期：70% 會打招呼"
cat << 'TESTCMD' | timeout 10 cargo run --release --bin main 2>/dev/null | grep -E "wang|說|收成|農田" | head -5
goto 48 51
exit
TESTCMD

echo ""
read -p "按 Enter 繼續下一個測試..."
echo ""

echo "▶ 測試 4: 訪問工程師 Steve（位置: 51, 49）"
echo "  預期：65% 會打招呼"
cat << 'TESTCMD' | timeout 10 cargo run --release --bin main 2>/dev/null | grep -E "Steve|說|projects|interesting" | head -5
goto 51 49
exit
TESTCMD

echo ""
read -p "按 Enter 繼續下一個測試..."
echo ""

echo "▶ 測試 5: 多次訪問同一個 NPC 測試機率"
echo "  訪問商人 Boss_Jenson 5次（位置: 53, 56）"
echo "  預期：約 4-5 次會說話（95% 機率）"
cat << 'TESTCMD' | timeout 15 cargo run --release --bin main 2>/dev/null | grep -E "Boss_Jenson|說|生意|新貨" | head -10
goto 53 56
goto 50 50
goto 53 56
goto 50 50
goto 53 56
goto 50 50
goto 53 56
goto 50 50
goto 53 56
exit
TESTCMD

echo ""
echo "╔════════════════════════════════════════════╗"
echo "║            測試完成！                      ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "💡 提示："
echo "  • 如果某些 NPC 沒說話，可能是因為積極度較低"
echo "  • 可以多訪問幾次來測試機率"
echo "  • 使用 'look <npc名稱>' 查看 NPC 詳細資訊"
echo ""
