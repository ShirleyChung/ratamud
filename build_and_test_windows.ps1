# Windows 版 RataMUD 構建和測試腳本
# 
# 使用 SCons 構建測試程式
#
# 前置需求:
# 1. Python 3.x 和 SCons (pip install scons)
# 2. Rust 和 Cargo
# 3. C/C++ 編譯器:
#    - Visual Studio Build Tools (推薦)
#    - 或 MinGW-w64
#
# 使用方法:
#   .\build_and_test_windows.ps1

Write-Host "=== RataMUD Windows 構建腳本 ===" -ForegroundColor Cyan
Write-Host ""

# 檢查必要工具
Write-Host "檢查必要工具..." -ForegroundColor Yellow

# 檢查 Python 和 SCons
try {
    $pythonVersion = python --version 2>&1
    Write-Host "✓ Python: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Python 未安裝或不在 PATH 中" -ForegroundColor Red
    exit 1
}

try {
    python -c "import SCons" 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ SCons 已安裝" -ForegroundColor Green
    } else {
        throw
    }
} catch {
    Write-Host "✗ SCons 未安裝" -ForegroundColor Red
    Write-Host "  請執行: pip install scons" -ForegroundColor Yellow
    exit 1
}

# 檢查 Cargo
try {
    $cargoVersion = cargo --version 2>&1
    Write-Host "✓ Cargo: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo 未安裝" -ForegroundColor Red
    exit 1
}

# 檢查 C 編譯器
$hasCompiler = $false
try {
    cl 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0 -or $LASTEXITCODE -eq null) {
        Write-Host "✓ MSVC (cl.exe) 可用" -ForegroundColor Green
        $hasCompiler = $true
    }
} catch {}

if (-not $hasCompiler) {
    try {
        gcc --version 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ GCC (MinGW) 可用" -ForegroundColor Green
            $hasCompiler = $true
        }
    } catch {}
}

if (-not $hasCompiler) {
    Write-Host "✗ 未找到 C 編譯器" -ForegroundColor Red
    Write-Host ""
    Write-Host "請安裝以下任一編譯器:" -ForegroundColor Yellow
    Write-Host "  1. Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/" -ForegroundColor Yellow
    Write-Host "  2. MinGW-w64: https://www.mingw-w64.org/" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "如果已安裝 Visual Studio, 請在 'Developer Command Prompt' 中執行此腳本" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "開始構建..." -ForegroundColor Cyan
Write-Host ""

# 構建
$buildMode = if ($args.Count -gt 0) { $args[0] } else { "release" }
Write-Host "構建模式: $buildMode" -ForegroundColor Yellow

python -m SCons mode=$buildMode

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ 構建失敗" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✓ 構建成功!" -ForegroundColor Green
Write-Host ""
Write-Host "輸出文件:" -ForegroundColor Cyan
Write-Host "  - dist\ratamud.dll     - Rust 動態函式庫" -ForegroundColor White
Write-Host "  - dist\example.exe     - C 範例程式" -ForegroundColor White
Write-Host "  - dist\test.exe        - C++ 測試程式" -ForegroundColor White
Write-Host ""
Write-Host "執行測試:" -ForegroundColor Cyan
Write-Host "  python -m SCons run-c       # 執行 C 範例" -ForegroundColor White
Write-Host "  python -m SCons run-cpp     # 執行 C++ 測試" -ForegroundColor White
Write-Host "  .\dist\example.exe          # 直接執行" -ForegroundColor White
