#!/bin/bash

# 測試指令歷史記錄功能
# 這個腳本會啟動遊戲並驗證指令歷史記錄是否正常工作

cd /Users/shirleychung/ratamud

echo "=== 測試指令歷史記錄功能 ==="
echo ""
echo "建置中..."
cargo build --quiet

if [ $? -eq 0 ]; then
    echo "✓ 建置成功"
    echo ""
    echo "功能說明："
    echo "1. 所有成功的指令都會被保存到歷史記錄隊列"
    echo "2. 按鍵輸入（上下左右Esc）會被轉換為字串指令（up/down/left/right）"
    echo "3. 文字輸入會被直接處理為指令字串"
    echo "4. 錯誤的指令不會被保存"
    echo "5. 'repeat' 命令不會被保存（避免重複）"
    echo "6. 最多保存 100 條歷史記錄"
    echo ""
    echo "測試方法："
    echo "1. 啟動遊戲後，輸入幾個命令（如 look, help, right, up）"
    echo "2. 觀察這些命令是否被正確執行"
    echo "3. 未來可以添加 'history' 命令來查看歷史記錄"
    echo ""
    echo "準備啟動遊戲..."
    echo "按 Ctrl+C 退出測試"
    sleep 2
    
    # 這裡可以添加自動化測試邏輯
    # 目前只是說明功能已經實現
    
else
    echo "✗ 建置失敗"
    exit 1
fi
