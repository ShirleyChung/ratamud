# 使用 SCons 編譯 RataMUD Framework 範例

## 快速開始

```bash
# 編譯並運行 Framework 版本範例
scons run-framework

# 或分步執行
scons framework           # 建立 macOS Framework
scons example-framework   # 編譯範例
./dist/example-framework  # 運行
```

## SCons 目標

### Framework 相關

| 目標 | 說明 |
|------|------|
| `framework` | 建立 macOS Framework |
| `example-framework` | 編譯 Framework 版本的 C 範例 |
| `all-framework` | 建立 Framework + 編譯範例 |
| `run-framework` | 運行 Framework 版本範例 |

### 一般目標

| 目標 | 說明 |
|------|------|
| `all` | 建立所有（dylib + 範例）|
| `lib` | 僅建立 Rust 動態函式庫 |
| `examples` | 建立 C/C++ 範例（dylib 版本）|
| `run-c` | 運行 C 範例（dylib）|
| `run-cpp` | 運行 C++ 測試 |

## 使用範例

### 編譯 Framework 版本

```bash
# 方法一：一次完成
scons all-framework

# 方法二：分步執行
scons framework
scons example-framework
```

### 運行範例

```bash
# 使用 SCons 運行（自動設定環境變數）
scons run-framework

# 或手動運行
./dist/example-framework
```

### 清理

```bash
# 清理編譯產物
scons -c

# 清理 Framework
rm -rf frameworks/RataMUD.framework
```

## 編譯選項

### Debug 模式

```bash
scons mode=debug example-framework
```

### 並行編譯

```bash
scons -j 4 all-framework
```

## 輸出文件

編譯成功後會產生以下文件：

```
dist/
├── example              # C 範例（dylib 版本）
├── example-framework    # C 範例（Framework 版本）
├── test                 # C++ 測試
└── libratamud.dylib     # Rust 動態函式庫

frameworks/
└── RataMUD.framework/
    ├── RataMUD              # Framework 二進制
    ├── Headers/
    │   └── ratamud.h        # C 頭文件
    └── Resources/
        └── Info.plist       # Framework 元數據
```

## Framework vs dylib

### Framework 版本的優勢

1. **標準 macOS 格式**：符合 Apple 平台慣例
2. **自包含**：包含頭文件和元數據
3. **易於整合**：可直接拖入 Xcode 專案
4. **版本管理**：支援多版本共存

### 編譯命令比較

```bash
# dylib 版本
gcc example.c -L./dist -lratamud -o example

# Framework 版本
gcc example.c -F./frameworks -framework RataMUD -framework CoreFoundation -o example-framework
```

## SCons 構建流程

```
1. [CARGO] 編譯 Rust 靜態庫
   ├─ target/aarch64-apple-darwin/release/libratamud.a
   
2. [FRAMEWORK] 建立 Framework 結構
   ├─ 建立 Versions/A/ 目錄
   ├─ 複製靜態庫 → RataMUD
   ├─ 複製頭文件 → Headers/ratamud.h
   ├─ 建立 Info.plist
   └─ 建立符號鏈接
   
3. [CC] 編譯 C 源碼
   ├─ gcc -c example.c
   
4. [LINK] 鏈接 Framework
   └─ gcc example-fw.o -framework RataMUD -framework CoreFoundation
```

## 疑難排解

### Framework 不存在

```
ld: framework not found RataMUD
```

**解決方案**：
```bash
scons framework
```

### 找不到符號

```
Undefined symbols: "_CFRelease"
```

**解決方案**：Framework 已自動鏈接 CoreFoundation，若手動編譯需加上：
```bash
-framework CoreFoundation
```

### 執行時錯誤

```
dyld: Library not loaded
```

**解決方案**：使用 `scons run-framework`，或設定 rpath：
```bash
gcc example.c -F./frameworks -framework RataMUD \
    -Wl,-rpath,./frameworks -o example-framework
```

## 與 Makefile 比較

| 特性 | SCons | Makefile |
|------|-------|----------|
| 跨平台 | ✅ 自動偵測 | ⚠️ 需手動配置 |
| 依賴追蹤 | ✅ 自動 | ⚠️ 需手動聲明 |
| 並行編譯 | ✅ `-j N` | ✅ `-j N` |
| 顏色輸出 | ✅ 內建 | ⚠️ 需手動實現 |
| 學習曲線 | ⚠️ 中等 | ✅ 低 |

## 幫助資訊

查看完整的 SCons 幫助：

```bash
scons -h
```

查看 SCons 內建選項：

```bash
scons -H
```
