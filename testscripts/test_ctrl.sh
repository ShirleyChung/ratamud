#!/bin/bash

# 測試切換控制角色功能

# 先創建一個 NPC
echo "create npc 工人 worker1"
sleep 0.5

# 查看該 NPC
echo "look worker1"
sleep 0.5

# 設置 NPC 屬性
echo "set worker1 hp 100"
sleep 0.5
echo "set worker1 strength 50"
sleep 0.5

# 切換控制到 NPC
echo "ctrl worker1"
sleep 0.5

# 查看現在控制的角色狀態
echo "status"
sleep 0.5

# 用新控制的角色移動
echo "move right"
sleep 0.5

# 查看位置
echo "look"
sleep 0.5

# 退出
echo "exit"
