#!/bin/bash
# RataMUD Library 和 C++ 测试编译脚本

set -e  # 遇到错误立即退出

echo "========================================"
echo "  RataMUD Library 编译脚本"
echo "========================================"
echo ""

# 1. 编译 Rust 库
echo "→ 编译 Rust 库 (release 模式)..."
cargo build --release --lib
echo "✓ Rust 库编译完成"
echo ""

# 2. 检查生成的库文件
echo "→ 检查生成的库文件..."
if [ -f "target/release/libratamud.dylib" ]; then
    echo "✓ 找到 libratamud.dylib (macOS 动态库)"
    ls -lh target/release/libratamud.dylib
elif [ -f "target/release/libratamud.so" ]; then
    echo "✓ 找到 libratamud.so (Linux 动态库)"
    ls -lh target/release/libratamud.so
fi

if [ -f "target/release/libratamud.a" ]; then
    echo "✓ 找到 libratamud.a (静态库)"
    ls -lh target/release/libratamud.a
fi
echo ""

# 3. 编译 C++ 测试程序
echo "→ 编译 C++ 测试程序..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -o test_callback
else
    # Linux
    g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -Wl,-rpath,./target/release -o test_callback
fi
echo "✓ C++ 测试程序编译完成"
echo ""

# 4. 运行测试
echo "→ 运行测试..."
echo "========================================"
echo ""
./test_callback
echo ""
echo "========================================"
echo "✅ 所有步骤完成！"
echo ""
echo "生成的文件:"
echo "  - target/release/libratamud.* (库文件)"
echo "  - test_callback (C++ 测试程序)"
echo "  - game_output.log (输出日志)"
