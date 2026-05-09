#!/bin/bash
echo "=== 測試物品系統 ==="
cargo build --bin main --quiet 2>&1 | grep -v warning || true

# 測試使用物品
cat << 'COMMANDS' | ./target/debug/main
create item apple
get apple
status
use apple
status
exit
COMMANDS
