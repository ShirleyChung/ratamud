# RataMUD Framework 範例 - 快速開始

## 一鍵編譯運行

```bash
scons run-framework
```

就這麼簡單！這會自動：
1. 建立 macOS Framework
2. 編譯 C 範例
3. 運行程式

## 輸出範例

```
╔══════════════════════════════════════╗
║   RataMUD 遊戲核心範例 (無 UI 模式)  ║
╚══════════════════════════════════════╝

🔧 註冊輸出回調函數...
✅ 回調已註冊

🎮 啟動遊戲核心測試...
─────────────────────────────────────

💬 歡迎來到 RataMUD！
💬 你站在一個廣場中央。
📝 遊戲初始化完成
📝 載入地圖: town_square
⚡ 遊戲時間: Day 1 09:00
ℹ️  NPC: 商人
   等級: 10
   生命: 100/100
💬 一隻野豬向你衝來！

─────────────────────────────────────
📊 測試完成！共收到 7 條訊息
```

## 其他命令

```bash
# 只編譯 Framework
scons framework

# 只編譯範例
scons example-framework

# 手動運行
./dist/example-framework

# 清理
scons -c
```

## 詳細文檔

- **SCONS_BUILD.md** - 完整的 SCons 構建說明
- **FRAMEWORK_EXAMPLE.md** - Framework 範例詳解

## 特色

✅ 無需 ratatui 或 crossterm  
✅ 用 printf 直接輸出  
✅ 使用 macOS Framework  
✅ 彩色 emoji 輸出  
✅ SCons 自動化構建
