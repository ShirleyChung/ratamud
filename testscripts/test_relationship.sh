#!/bin/bash
# 測試關係系統

echo "=== 測試關係系統 ==="
echo ""

# 編譯項目
echo "1. 編譯項目..."
cargo build --quiet
echo "   ✓ 編譯完成"
echo ""

# 創建測試腳本
cat > test_rel_commands.txt << 'EOF'
# 創建一個測試 NPC
create npc m 測試商人

# 設置不同好感度等級的對話
setdialogue 測試商人 對話:敵對 哼！滾開！
setdialogue 測試商人 對話:冷淡 你要幹嘛...
setdialogue 測試商人 對話:普通 你好，需要什麼嗎？
setdialogue 測試商人 對話:好友 嘿！老朋友，今天過得怎麼樣？
setdialogue 測試商人 對話:摯友 我最好的朋友！有什麼需要儘管說！

# 測試不同好感度的對話
echo "=== 測試敵對狀態 (-50) ==="
setrel 測試商人 -50
check 測試商人
talk 測試商人

echo ""
echo "=== 測試冷淡狀態 (-20) ==="
setrel 測試商人 -20
check 測試商人
talk 測試商人

echo ""
echo "=== 測試普通狀態 (10) ==="
setrel 測試商人 10
check 測試商人
talk 測試商人

echo ""
echo "=== 測試好友狀態 (50) ==="
setrel 測試商人 50
check 測試商人
talk 測試商人

echo ""
echo "=== 測試摯友狀態 (80) ==="
setrel 測試商人 80
check 測試商人
talk 測試商人

echo ""
echo "=== 測試好感度變化 ==="
setrel 測試商人 0
changerel 測試商人 10
changerel 測試商人 20
changerel 測試商人 30
check 測試商人

echo ""
echo "=== 測試互動計數 ==="
talk 測試商人
talk 測試商人
talk 測試商人
check 測試商人

exit
EOF

echo "2. 運行測試..."
echo ""
cargo run < test_rel_commands.txt 2>/dev/null | grep -A 200 "測試"

# 清理
rm -f test_rel_commands.txt

echo ""
echo "✓ 測試完成！"
