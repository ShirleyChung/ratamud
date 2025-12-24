#!/bin/bash

echo "🔨 开始编译..."
cd /Users/shireychung/ratamud

cargo build 2>&1

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 编译成功！"
    echo ""
    echo "运行游戏："
    echo "  cargo run"
    echo ""
    echo "游戏内输入 'minimap' 查看 15x10 彩色网格"
else
    echo ""
    echo "❌ 编译失败，请查看上面的错误信息"
    exit 1
fi
