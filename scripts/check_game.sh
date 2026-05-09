#!/bin/bash
# 測試遊戲啟動和基本功能

echo "開始測試遊戲..."

# 檢查地圖檔案
echo "=== 檢查地圖檔案 ==="
ls -lh worlds/beginWorld/maps/*.json | awk '{print $9, $5}'

# 檢查地圖內部名稱
echo ""
echo "=== 檢查地圖名稱 ==="
for map in worlds/beginWorld/maps/*.json; do
    name=$(jq -r '.name' "$map")
    echo "$(basename $map): name=$name"
done

# 檢查 world.json
echo ""
echo "=== 檢查 world.json ==="
cat worlds/beginWorld/world.json | jq '.name, .current_map, .maps'

# 檢查玩家位置
echo ""
echo "=== 檢查玩家資料 ==="
cat worlds/beginWorld/persons/me.json | jq '{name, map, x, y, hp, mp}'

echo ""
echo "=== 檢查完成 ==="
echo ""
echo "如果一切正常，嘗試啟動遊戲："
echo "  cd /Users/shirleychung/ratamud && cargo run --bin main"
