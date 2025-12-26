#!/bin/bash

# 測試 trade 指令

echo "=== 測試 trade 指令 ==="

# 編譯
echo "編譯..."
cargo build --quiet || exit 1

# 建立測試地圖
mkdir -p worlds/test_world/maps
cat > worlds/test_world/maps/test_map.json << 'EOF'
{
  "name": "測試地圖",
  "width": 10,
  "height": 10,
  "tiles": "##########\n#........#\n#........#\n#........#\n#........#\n#........#\n#........#\n#........#\n#........#\n##########",
  "npcs": [
    {
      "id": "merchant",
      "name": "商人",
      "aliases": ["店主", "seller"],
      "x": 5,
      "y": 5,
      "hp": 100,
      "max_hp": 100,
      "items": {
        "蘋果": 10,
        "金幣": 10000
      }
    }
  ]
}
EOF

# 執行測試
echo "執行測試..."
cat << 'EOF' | ./target/debug/ratamud
loadmap worlds/test_world/maps/test_map.json
go 5,5
look
trade 商人
exit
EOF

echo ""
echo "=== 測試完成 ==="
