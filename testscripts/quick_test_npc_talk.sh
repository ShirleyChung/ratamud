#!/bin/bash

# 簡單測試 - 驗證 NPC 台詞功能
echo "🎮 快速測試 NPC 台詞功能"
echo ""
echo "測試：訪問李老闆（積極度 100%，一定會說話）"
echo "位置：(51, 50)"
echo ""

cat << 'TESTCMD' | cargo run --release --bin main 2>/dev/null
goto 51 50
exit
TESTCMD

echo ""
echo "✅ 如果看到 '💬 李老闆 說：' 就表示功能正常！"
echo ""
echo "📋 所有 NPC 台詞列表："
echo "  • 商人 (100%) - 歡迎光臨！我這裡有最好的商品..."
echo "  • 李老闆 (100%) - 哎呀！有客人來了！今天有特價優惠..."
echo "  • 醫生 (80%) - 你好，有什麼不舒服的嗎？..."
echo "  • 櫻花 (85%) - 你好呀！需要醫療協助嗎？"
echo "  • wang (70%) - 嗨，今年的收成不錯呢！..."
echo "  • 工人 (60%) - 嘿！工作辛苦了！..."
echo "  • 阿猛 (75%) - 喲！有什麼活要幹嗎？..."
echo "  • Steve (65%) - Hi there! I'm working on..."
echo "  • Boss_Jenson (95%) - 啊，是你啊！最近生意還不錯..."
echo "  • 小鹿 (90%) - 你好～我是小鹿，正在這裡擺攤..."
echo "  • merchant (100%) - Welcome! I have the finest..."
echo ""
echo "詳細文檔：Docs/NPC_DIALOGUES_LIST.md"
