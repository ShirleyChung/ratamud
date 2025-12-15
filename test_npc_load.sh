#!/bin/bash

# 測試 NPC 載入
cat << 'TESTCMD' | timeout 5 cargo run --release 2>/dev/null | grep -E "已載入 NPC|NPC"
look
exit
TESTCMD
