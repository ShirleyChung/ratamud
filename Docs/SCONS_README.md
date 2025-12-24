# RataMUD SCons 構建系統使用說明

本項目使用 SCons 作為構建系統，支持構建 Rust 動態函式庫和 C/C++ 範例程式。

## 📦 安裝 SCons

### macOS
```bash
brew install scons
```

### Linux
```bash
# Ubuntu/Debian
sudo apt-get install scons

# Fedora
sudo dnf install scons
```

### Windows
```bash
pip install scons
```

## 🚀 快速開始

### 查看幫助
```bash
scons -h
```

### 構建所有目標（預設）
```bash
scons
```

### 僅構建 Rust 函式庫
```bash
scons lib
```

### 僅構建 C/C++ 範例
```bash
scons examples
```

### 清理構建產物
```bash
scons -c
```

## 🎯 可用目標

| 目標 | 說明 |
|------|------|
| `all` | 構建所有（預設） |
| `lib` | 僅構建 Rust 函式庫 |
| `examples` | 僅構建 C/C++ 範例 |
| `c-example` | 僅構建 C 範例 |
| `cpp-test` | 僅構建 C++ 測試 |
| `run-c` | 運行 C 範例 |
| `run-cpp` | 運行 C++ 測試 |

## ⚙️ 構建選項

### 構建模式

預設是 `release` 模式，可以切換到 `debug` 模式：

```bash
# Release 模式（預設）
scons

# Debug 模式
scons mode=debug
```

### 並行構建

使用 `-j` 選項指定並行任務數：

```bash
# 使用 4 個並行任務
scons -j 4

# 使用所有可用 CPU 核心
scons -j $(nproc)  # Linux
scons -j $(sysctl -n hw.ncpu)  # macOS
```

### 詳細輸出

```bash
# 顯示完整命令
scons --debug=explain
```

## 📝 使用範例

### 完整構建流程
```bash
# 清理舊的構建
scons -c

# 構建所有（release 模式）
scons

# 運行 C++ 測試
scons run-cpp

# 運行 C 範例
scons run-c
```

### 快速測試
```bash
# 構建並運行 C++ 測試
scons cpp-test run-cpp
```

### Debug 構建
```bash
# 構建 debug 版本
scons mode=debug

# 清理後重新構建 debug 版本
scons -c && scons mode=debug
```

## 🔧 構建系統結構

```
ratamud/
├── SConstruct          # 主構建文件
├── dist/
│   ├── SConscript      # 範例程序構建文件
│   ├── example.c       # C 範例
│   ├── test.cpp        # C++ 測試
│   ├── ratamud.h       # C API 標頭檔
│   └── libratamud.*    # 動態函式庫（構建後生成）
├── src/                # Rust 源代碼
└── Cargo.toml          # Rust 配置
```

## 🎨 輸出說明

SCons 使用彩色輸出來區分不同的構建步驟：

- **青色 [CARGO]** - Rust 編譯
- **綠色 [CC]** - C/C++ 編譯
- **藍色 [LINK]** - C 連結
- **紫色 [LINK]** - C++ 連結
- **黃色 [BUILD]** - 其他構建步驟

範例輸出：
```
[CARGO] Building Rust library (release)...
✓ Rust library: dist/libratamud.dylib

[CC] dist/example.c
[LINK] dist/example
[CC] dist/test.cpp
[LINK] dist/test
```

## 🆚 與 Makefile 比較

### 優勢

1. **自動依賴檢測** - SCons 自動追蹤文件依賴
2. **跨平台** - 無需為不同平台維護不同的構建文件
3. **Python 腳本** - 使用 Python 編寫，更靈活
4. **增量構建** - 更智能的增量構建
5. **並行構建** - 原生支持並行構建

### 使用建議

- 如果習慣 Make，可以繼續使用 `dist/Makefile`
- 如果需要跨平台或複雜構建邏輯，推薦使用 SCons
- 兩者可以並存，選擇您喜歡的即可

## 🐛 疑難排解

### SCons 找不到
```bash
# 確認 SCons 已安裝
which scons
scons --version
```

### Cargo 找不到
```bash
# 確認 Rust 工具鏈已安裝
which cargo
cargo --version
```

### 構建失敗

1. **清理後重新構建**
   ```bash
   scons -c
   scons
   ```

2. **檢查 Rust 構建**
   ```bash
   cargo build --lib --release
   ```

3. **檢查依賴**
   ```bash
   # 確認函式庫已生成
   ls -l dist/libratamud.*
   
   # 確認符號已導出
   nm -g dist/libratamud.dylib | grep ratamud
   ```

### 運行測試失敗

確保動態函式庫路徑正確：

macOS:
```bash
export DYLD_LIBRARY_PATH=./dist:$DYLD_LIBRARY_PATH
./dist/test
```

Linux:
```bash
export LD_LIBRARY_PATH=./dist:$LD_LIBRARY_PATH
./dist/test
```

## 📚 進階使用

### 自定義構建選項

在 `SConstruct` 中修改：

```python
# 修改編譯器
env['CC'] = 'clang'
env['CXX'] = 'clang++'

# 添加編譯選項
env.Append(CXXFLAGS=['-std=c++20'])

# 修改優化級別
env.Append(CXXFLAGS=['-O3'])
```

### 添加新的構建目標

在 `dist/SConscript` 中添加：

```python
# 添加新的程序
new_program = local_env.Program(
    target='#/dist/new_program',
    source='#/dist/new_program.cpp'
)
Depends(new_program, rust_lib)
local_env.Alias('new', new_program)
```

然後構建：
```bash
scons new
```

## 📖 更多資訊

- SCons 官方文檔: https://scons.org/documentation.html
- SCons 用戶指南: https://scons.org/doc/production/HTML/scons-user.html
- Rust 構建說明: 查看 `C_ABI_README.md`

## 🤝 貢獻

歡迎提交改進構建系統的 PR！

## 📄 授權

與 RataMUD 主項目相同
