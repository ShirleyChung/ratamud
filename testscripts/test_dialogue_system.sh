#!/bin/bash
# 測試新的對話系統功能

echo "=== 測試 NPC 對話系統 ==="
echo ""

# 測試指令列表
commands=(
    # 1. 創建測試 NPC
    "create person 櫻花 可愛的女孩"
    "summon 櫻花"
    "set 櫻花 性別 女"
    "set 櫻花 顏值 85"
    "set 櫻花 mp 600"
    ""
    # 2. 設置簡單對話（無條件）
    "sdl 櫻花 見面 你好啊！很高興見到你"
    "sdl 櫻花 閒聊 今天天氣真好呢"
    "sdl 櫻花 閒聊 add 我最喜歡春天了"
    ""
    # 3. 設置帶條件的對話
    "sdl 櫻花 閒聊 add 你長得好漂亮啊 when 顏值>80 and 性別=女"
    "sdl 櫻花 閒聊 add 你看起來精神很好 when mp>500"
    "sdl 櫻花 閒聊 add 你有很多東西呢 when 物品數量>5"
    ""
    # 4. 測試對話
    "check 櫻花"
    "talk 櫻花 見面"
    "talk 櫻花 閒聊"
    "talk 櫻花 閒聊"
    "talk 櫻花 閒聊"
    ""
    # 5. 修改屬性後再測試
    "set 櫻花 mp 300"
    "talk 櫻花 閒聊"
    "talk 櫻花 閒聊"
    ""
    # 6. 測試工人 NPC
    "create person 工人 辛勤工作的人"
    "summon 工人"
    "set 工人 性別 男"
    "set 工人 mp 95000"
    "sdl 工人 見面 嘿！工作辛苦了！"
    "sdl 工人 閒聊 明天就可以放假了"
    "sdl 工人 閒聊 add 這個工程真不好做 when mp<98000"
    "sdl 工人 閒聊 add 今天狀態很好！ when mp>98000"
    ""
    "talk 工人 見面"
    "talk 工人 閒聊"
    "talk 工人 閒聊"
    "talk 工人 閒聊"
    ""
    # 7. 測試商人（依持有物品數量）
    "create person 商人 精明的商人"
    "summon 商人"
    "sdl 商人 閒聊 歡迎光臨！"
    "sdl 商人 閒聊 add 要不要賣些東西給我 when 物品數量>5"
    "sdl 商人 閒聊 add 看來你身無長物啊 when 物品數量<3"
    ""
    "get 麵包"
    "get 麵包"
    "get 火把"
    "talk 商人 閒聊"
    "talk 商人 閒聊"
    ""
    "get 麵包"
    "get 繩索"
    "get 匕首"
    "get 弓"
    "talk 商人 閒聊"
    "talk 商人 閒聊"
)

echo "開始執行測試命令..."
echo ""

for cmd in "${commands[@]}"; do
    if [ -z "$cmd" ]; then
        echo ""
        sleep 1
    else
        echo "> $cmd"
        sleep 0.5
    fi
done

echo ""
echo "=== 測試完成 ==="
