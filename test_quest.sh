#!/bin/bash
# 測試任務系統

echo "=== 測試任務系統 ==="
echo ""

# 編譯項目
echo "1. 編譯項目..."
cargo build --quiet
echo "   ✓ 編譯完成"
echo ""

# 創建測試命令
cat > test_quest_commands.txt << 'EOF'
# 列出所有任務
echo "=== 查看所有任務 ==="
quest list

# 查看任務詳情
echo ""
echo "=== 查看任務詳情 ==="
quest info tutorial_talk_to_merchant

# 開始任務
echo ""
echo "=== 開始任務 ==="
quest start tutorial_talk_to_merchant

# 查看進行中的任務
echo ""
echo "=== 查看進行中的任務 ==="
quest active

# 與商人對話
echo ""
echo "=== 與商人對話 ==="
talk 商人

# 嘗試完成任務（這會失敗，因為條件還沒完成）
echo ""
echo "=== 嘗試完成任務（應該失敗） ==="
quest complete tutorial_talk_to_merchant

exit
EOF

echo "2. 運行測試..."
echo ""
timeout 10 cargo run --quiet < test_quest_commands.txt 2>/dev/null | grep -E "===|任務|quest|商人|Completed" | head -40

# 清理
rm -f test_quest_commands.txt

echo ""
echo "✓ 測試完成！"
