# Windows 構建指南

## 前置需求

### 1. 安裝 Python 3.x
- 從 https://www.python.org/downloads/ 下載並安裝
- 確保勾選 "Add Python to PATH"

### 2. 安裝 SCons
```powershell
pip install scons
```

### 3. 安裝 Rust
- 從 https://rustup.rs/ 下載並安裝 rustup
- 重啟終端確保 `cargo` 在 PATH 中

### 4. 安裝 C/C++ 編譯器

#### 選項 A: Visual Studio Build Tools (推薦)
1. 下載 Build Tools: https://visualstudio.microsoft.com/downloads/
2. 選擇 "Build Tools for Visual Studio"
3. 安裝時勾選 "Desktop development with C++"
4. 安裝後，在開始菜單找到 "Developer Command Prompt for VS"

#### 選項 B: MinGW-w64
1. 下載 MinGW-w64: https://www.mingw-w64.org/
2. 或使用 MSYS2: https://www.msys2.org/
3. 將 MinGW bin 目錄添加到 PATH

## 構建步驟

### 方法 1: 使用構建腳本（推薦）

```powershell
# 構建所有 (release 模式)
.\build_and_test_windows.ps1

# 構建 debug 模式
.\build_and_test_windows.ps1 debug
```

### 方法 2: 使用 SCons 命令

```powershell
# 構建所有
python -m SCons

# 僅構建 Rust 庫
python -m SCons lib

# 僅構建範例程式
python -m SCons examples

# Debug 模式
python -m SCons mode=debug

# 清理構建
python -m SCons -c
```

## 執行測試

### 執行範例程式

```powershell
# 方法 1: 通過 SCons
python -m SCons run-c      # C 範例
python -m SCons run-cpp    # C++ 測試

# 方法 2: 直接執行
.\dist\example.exe
.\dist\test.exe
```

### 執行 Rust 主程式

```powershell
cargo run
```

## 輸出文件

構建成功後會生成以下文件：

```
dist/
  ├── ratamud.dll        # Rust 動態函式庫
  ├── ratamud.dll.lib    # Windows 導入庫 (MSVC)
  ├── example.exe        # C 範例程式
  └── test.exe           # C++ 測試程式
```

## 常見問題

### Q: 找不到編譯器
**A:** 
- 如果使用 Visual Studio，請在 "Developer Command Prompt for VS" 中執行
- 如果使用 MinGW，確保 `gcc` 在 PATH 中: `gcc --version`

### Q: 鏈接錯誤 "cannot find -lratamud"
**A:** 
- 確保先構建了 Rust 庫: `python -m SCons lib`
- Windows 下需要 `.dll.lib` 文件，由 Cargo 自動生成

### Q: 執行時找不到 DLL
**A:** 
- 確保在專案根目錄執行，或將 `dist/` 添加到 PATH
- 或將 `ratamud.dll` 複製到執行檔同目錄

### Q: SCons 命令找不到
**A:** 
- 使用 `python -m SCons` 而非 `scons`
- 或將 Python Scripts 目錄添加到 PATH

## 與 macOS/Linux 的差異

| 項目 | macOS/Linux | Windows |
|-----|-------------|---------|
| 庫文件 | `.dylib` / `.so` | `.dll` |
| 執行檔 | 無副檔名 | `.exe` |
| 編譯器 | GCC/Clang | MSVC / MinGW |
| 路徑分隔符 | `/` | `\` |

## 測試腳本

Windows 版測試腳本 (PowerShell):
- `build_and_test_windows.ps1` - 構建腳本
- `test_combat_loop_windows.ps1` - 戰鬥系統測試

原 Bash 腳本 (需要 Git Bash 或 WSL):
- `build_and_test.sh`
- `test_combat_loop.sh`
- 其他 `test_*.sh`

## 開發環境建議

推薦使用以下任一環境:
1. **Visual Studio Code** + PowerShell
2. **Visual Studio** + Developer Command Prompt
3. **Windows Terminal** + PowerShell 7
