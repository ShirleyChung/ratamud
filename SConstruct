# RataMUD SCons 構建配置
# 主構建文件

import os
import platform

# 設定環境變數（移除 rust 工具）
env = Environment(
    ENV=os.environ,
)

# 檢測平台
system = platform.system()
print(f"Building on {system}")

# 平台特定設定
if system == 'Darwin':
    env['LIB_EXT'] = 'dylib'
    env['RPATH'] = '-Wl,-rpath,.'
elif system == 'Linux':
    env['LIB_EXT'] = 'so'
    env['RPATH'] = '-Wl,-rpath,.'
elif system == 'Windows':
    env['LIB_EXT'] = 'dll'
    env['RPATH'] = ''
else:
    print(f"Warning: Unknown platform {system}")
    env['LIB_EXT'] = 'so'
    env['RPATH'] = ''

# Rust 相關設定
env['CARGO'] = 'cargo'
env['RUST_BUILD_DIR'] = 'target/release'
env['RUST_DEBUG_DIR'] = 'target/debug'

# C/C++ 編譯設定
if system == 'Windows':
    # Windows 使用 MSVC 或 MinGW
    # SCons 會自動偵測可用的編譯器
    env['CFLAGS'] = []
    env['CXXFLAGS'] = ['/std:c++17'] if 'MSVC' in env['TOOLS'] else ['-std=c++17', '-Wall', '-Wextra']
else:
    env['CC'] = 'gcc'
    env['CXX'] = 'g++'
    env['CFLAGS'] = ['-Wall', '-Wextra']
    env['CXXFLAGS'] = ['-std=c++17', '-Wall', '-Wextra']

# 構建模式（release 或 debug）
build_mode = ARGUMENTS.get('mode', 'release')
env['BUILD_MODE'] = build_mode

if build_mode == 'debug':
    if system == 'Windows':
        env.Append(CFLAGS=['/Zi', '/Od'])
        env.Append(CXXFLAGS=['/Zi', '/Od'])
    else:
        env.Append(CFLAGS=['-g', '-O0'])
        env.Append(CXXFLAGS=['-g', '-O0'])
    env['RUST_TARGET_DIR'] = env['RUST_DEBUG_DIR']
else:
    if system == 'Windows':
        env.Append(CFLAGS=['/O2'])
        env.Append(CXXFLAGS=['/O2'])
    else:
        env.Append(CFLAGS=['-O2'])
        env.Append(CXXFLAGS=['-O2'])
    env['RUST_TARGET_DIR'] = env['RUST_BUILD_DIR']

# 輸出目錄
env['DIST_DIR'] = 'dist'
env['BUILD_DIR'] = 'build'

# 顏色輸出支持
colors = {}
colors['cyan']   = '\033[96m'
colors['purple'] = '\033[95m'
colors['blue']   = '\033[94m'
colors['green']  = '\033[92m'
colors['yellow'] = '\033[93m'
colors['red']    = '\033[91m'
colors['end']    = '\033[0m'

# 如果不支持顏色（Windows CMD），禁用顏色
if system == 'Windows' and not os.environ.get('ANSICON'):
    colors = {k: '' for k in colors}

env['COLORS'] = colors

# 自定義構建訊息
def print_cmd_line(s, target, source, env):
    if not source:
        print(f"{env['COLORS']['yellow']}[CMD]{env['COLORS']['end']} {s}")
    elif 'rust' in str(source[0]):
        print(f"{env['COLORS']['cyan']}[RUST]{env['COLORS']['end']} Building Rust library...")
    elif str(target[0]).endswith('.o'):
        print(f"{env['COLORS']['green']}[CC]{env['COLORS']['end']} {source[0]}")
    elif 'example' in str(target[0]):
        print(f"{env['COLORS']['blue']}[LINK]{env['COLORS']['end']} {target[0]}")
    elif 'test' in str(target[0]):
        print(f"{env['COLORS']['purple']}[LINK]{env['COLORS']['end']} {target[0]}")
    else:
        print(f"{env['COLORS']['yellow']}[BUILD]{env['COLORS']['end']} {target[0]}")

env['PRINT_CMD_LINE_FUNC'] = print_cmd_line

# 自定義 Rust 構建器
def rust_builder(target, source, env):
    """構建 Rust 動態函式庫"""
    import subprocess
    
    build_type = env['BUILD_MODE']
    cmd = [env['CARGO'], 'build', '--lib']
    
    if build_type == 'release':
        cmd.append('--release')
    
    print(f"{env['COLORS']['cyan']}Running: {' '.join(cmd)}{env['COLORS']['end']}")
    
    result = subprocess.run(cmd, cwd=env.Dir('.').abspath)
    
    if result.returncode != 0:
        print(f"{env['COLORS']['red']}Rust build failed!{env['COLORS']['end']}")
        return result.returncode
    
    # 複製生成的函式庫到 dist
    import shutil
    lib_name = f"libratamud.{env['LIB_EXT']}"
    src = os.path.join(env['RUST_TARGET_DIR'], lib_name)
    dst = os.path.join(env['DIST_DIR'], lib_name)
    
    os.makedirs(env['DIST_DIR'], exist_ok=True)
    shutil.copy2(src, dst)
    
    print(f"{env['COLORS']['green']}✓ Rust library built: {dst}{env['COLORS']['end']}")
    
    # 複製 worlds 資料夾到 dist
    worlds_src = 'worlds'
    worlds_dst = os.path.join(env['DIST_DIR'], 'worlds')
    
    if os.path.exists(worlds_src):
        if os.path.exists(worlds_dst):
            shutil.rmtree(worlds_dst)
        shutil.copytree(worlds_src, worlds_dst)
        print(f"{env['COLORS']['green']}✓ Worlds folder copied to: {worlds_dst}{env['COLORS']['end']}")
    
    return 0

# iOS Framework 構建器
def ios_framework_builder(target, source, env):
    """構建 iOS Frameworks (真機和模擬器)"""
    import subprocess
    import shutil
    
    project_name = "ratamud"
    framework_name = "RataMUD"
    
    print(f"{env['COLORS']['cyan']}Building iOS Frameworks...{env['COLORS']['end']}")
    
    # 確保 iOS targets 已安裝
    print(f"{env['COLORS']['yellow']}Checking iOS targets...{env['COLORS']['end']}")
    for target_triple in ['aarch64-apple-ios', 'aarch64-apple-ios-sim', 'x86_64-apple-ios']:
        subprocess.run(['rustup', 'target', 'add', target_triple], 
                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    # 建立輸出目錄
    os.makedirs('frameworks/ios', exist_ok=True)
    os.makedirs('frameworks/ios-simulator', exist_ok=True)
    
    # iOS 真機 (ARM64) - 無 UI 模式
    print(f"{env['COLORS']['green']}Building for iOS Device (ARM64)...{env['COLORS']['end']}")
    result = subprocess.run([
        env['CARGO'], 'build', '--release', 
        '--target', 'aarch64-apple-ios', 
        '--lib', '--no-default-features'
    ])
    if result.returncode != 0:
        print(f"{env['COLORS']['red']}iOS device build failed!{env['COLORS']['end']}")
        return result.returncode
    
    # iOS 模擬器 (Apple Silicon)
    print(f"{env['COLORS']['green']}Building for iOS Simulator (ARM64)...{env['COLORS']['end']}")
    result = subprocess.run([
        env['CARGO'], 'build', '--release',
        '--target', 'aarch64-apple-ios-sim',
        '--lib', '--no-default-features'
    ])
    if result.returncode != 0:
        print(f"{env['COLORS']['red']}iOS Simulator ARM64 build failed!{env['COLORS']['end']}")
        return result.returncode
    
    # iOS 模擬器 (Intel)
    print(f"{env['COLORS']['green']}Building for iOS Simulator (x86_64)...{env['COLORS']['end']}")
    result = subprocess.run([
        env['CARGO'], 'build', '--release',
        '--target', 'x86_64-apple-ios',
        '--lib', '--no-default-features'
    ])
    if result.returncode != 0:
        print(f"{env['COLORS']['red']}iOS Simulator x86_64 build failed!{env['COLORS']['end']}")
        return result.returncode
    
    # 建立模擬器 Universal Binary
    print(f"{env['COLORS']['cyan']}Creating Simulator Universal Binary...{env['COLORS']['end']}")
    result = subprocess.run([
        'lipo', '-create',
        f'target/aarch64-apple-ios-sim/release/lib{project_name}.a',
        f'target/x86_64-apple-ios/release/lib{project_name}.a',
        '-output', f'frameworks/ios-simulator/lib{project_name}.a'
    ])
    if result.returncode != 0:
        print(f"{env['COLORS']['red']}lipo failed!{env['COLORS']['end']}")
        return result.returncode
    
    # 建立 iOS Framework 結構 (真機)
    ios_framework = f"frameworks/{framework_name}-iOS.framework"
    if os.path.exists(ios_framework):
        shutil.rmtree(ios_framework)
    os.makedirs(f"{ios_framework}/Headers")
    
    shutil.copy2(f'target/aarch64-apple-ios/release/lib{project_name}.a', 
                 f'{ios_framework}/{framework_name}')
    shutil.copy2('src/ratamud.h', f'{ios_framework}/Headers/')
    
    # 建立 Info.plist
    with open(f'{ios_framework}/Info.plist', 'w') as f:
        f.write('''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>''' + framework_name + '''</string>
    <key>CFBundleIdentifier</key>
    <string>com.ratamud.framework.ios</string>
    <key>CFBundleName</key>
    <string>''' + framework_name + '''</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>MinimumOSVersion</key>
    <string>13.0</string>
</dict>
</plist>
''')
    
    print(f"{env['COLORS']['green']}✓ iOS Framework: {ios_framework}{env['COLORS']['end']}")
    
    # 建立 iOS 模擬器 Framework
    ios_sim_framework = f"frameworks/{framework_name}-iOS-Simulator.framework"
    if os.path.exists(ios_sim_framework):
        shutil.rmtree(ios_sim_framework)
    os.makedirs(f"{ios_sim_framework}/Headers")
    
    shutil.copy2(f'frameworks/ios-simulator/lib{project_name}.a',
                 f'{ios_sim_framework}/{framework_name}')
    shutil.copy2('src/ratamud.h', f'{ios_sim_framework}/Headers/')
    
    with open(f'{ios_sim_framework}/Info.plist', 'w') as f:
        f.write('''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>''' + framework_name + '''</string>
    <key>CFBundleIdentifier</key>
    <string>com.ratamud.framework.ios-simulator</string>
    <key>CFBundleName</key>
    <string>''' + framework_name + '''</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>MinimumOSVersion</key>
    <string>13.0</string>
</dict>
</plist>
''')
    
    print(f"{env['COLORS']['green']}✓ iOS Simulator Framework: {ios_sim_framework}{env['COLORS']['end']}")
    
    # 建立 XCFramework
    print(f"{env['COLORS']['cyan']}Creating XCFramework...{env['COLORS']['end']}")
    xcframework = f"frameworks/{framework_name}.xcframework"
    if os.path.exists(xcframework):
        shutil.rmtree(xcframework)
    
    result = subprocess.run([
        'xcodebuild', '-create-xcframework',
        '-framework', ios_framework,
        '-framework', ios_sim_framework,
        '-output', xcframework
    ])
    
    if result.returncode == 0:
        print(f"{env['COLORS']['green']}✓ XCFramework: {xcframework}{env['COLORS']['end']}")
    
    print(f"{env['COLORS']['cyan']}🎉 All iOS frameworks built successfully!{env['COLORS']['end']}")
    
    return 0

# 註冊 Rust 構建器
rust_bld = Builder(action=rust_builder)
env.Append(BUILDERS={'RustLib': rust_bld})

# 註冊 iOS Framework 構建器 (僅 macOS)
if system == 'Darwin':
    ios_bld = Builder(action=ios_framework_builder)
    env.Append(BUILDERS={'IOSFramework': ios_bld})

# 導出環境變數給 SConscript
Export('env')

# 調用子構建腳本（不使用 variant_dir 以避免路徑問題）
SConscript('dist/SConscript', exports='env')

# 別名（由 SConscript 設置）
Alias('examples', [f'dist/example', f'dist/test'])
Alias('all', ['lib', 'examples'])

# macOS Framework 別名
if env.get('LIB_EXT') == 'dylib':
    Alias('all-framework', ['framework', 'example-framework'])
    Alias('ios-frameworks', 'frameworks/RataMUD.xcframework/Info.plist')
    Alias('all-ios', 'ios-frameworks')

# 清理目標
if GetOption('clean'):
    import shutil
    # 清理 Rust 構建
    if os.path.exists('target'):
        print(f"{env['COLORS']['yellow']}Cleaning Rust target directory...{env['COLORS']['end']}")
    # 清理 dist 中的可執行文件
    for f in ['dist/example', 'dist/test']:
        if os.path.exists(f):
            print(f"{env['COLORS']['yellow']}Removing {f}{env['COLORS']['end']}")

# 默認目標
Default('all')

# 幫助資訊
Help(f"""
{colors['cyan']}RataMUD SCons 構建系統{colors['end']}

{colors['green']}用法:{colors['end']}
  scons [目標] [選項]

{colors['green']}目標:{colors['end']}
  all                - 構建所有（預設：lib + dylib 範例）
  lib                - 僅構建 Rust 動態函式庫
  examples           - 僅構建 C/C++ 範例（dylib 版本）
  
  {colors['yellow']}macOS Framework 目標:{colors['end']}
  framework          - 構建 macOS Framework
  example-framework  - 構建 Framework 版本的 C 範例
  all-framework      - 構建 Framework + Framework 範例
  
  {colors['yellow']}iOS Framework 目標:{colors['end']}
  ios-frameworks     - 構建 iOS Frameworks (真機 + 模擬器 + XCFramework)
  all-ios            - 同 ios-frameworks
  
  {colors['yellow']}運行目標:{colors['end']}
  run-c              - 運行 C 範例（dylib）
  run-framework      - 運行 C 範例（Framework）
  run-cpp            - 運行 C++ 測試
  
{colors['green']}選項:{colors['end']}
  mode=release|debug  - 構建模式（預設: release）
  -c                  - 清理構建產物
  -j N                - 使用 N 個並行任務

{colors['green']}範例:{colors['end']}
  {colors['cyan']}# 基本使用（dylib）{colors['end']}
  scons                    # 構建所有（release 模式）
  scons mode=debug         # 構建所有（debug 模式）
  scons run-c              # 構建並運行 C 範例
  
  {colors['cyan']}# macOS Framework{colors['end']}
  scons framework          # 僅構建 macOS Framework
  scons example-framework  # 構建 Framework 版本範例
  scons run-framework      # 運行 Framework 版本
  scons all-framework      # 構建所有 Framework 相關
  
  {colors['cyan']}# iOS Frameworks{colors['end']}
  scons ios-frameworks     # 構建 iOS Frameworks (真機 + 模擬器 + XCFramework)
  scons all-ios            # 同上
  
  {colors['cyan']}# 其他{colors['end']}
  scons -c                 # 清理
  scons -j 4               # 使用 4 個並行任務構建

{colors['green']}文件:{colors['end']}
  SConstruct          - 主構建文件
  dist/SConscript     - 範例程序構建文件
  
{colors['green']}輸出:{colors['end']}
  dist/libratamud.dylib                   - Rust 動態函式庫
  dist/example                            - C 範例（dylib）
  dist/example-framework                  - C 範例（Framework）
  frameworks/RataMUD.framework/           - macOS Framework
  frameworks/RataMUDiOS.framework/        - iOS Framework (真機)
  frameworks/RataMUDiOSSimulator.framework/  - iOS Framework (模擬器)
  frameworks/RataMUD.xcframework/         - XCFramework (真機 + 模擬器)

{colors['green']}關於 iOS Frameworks:{colors['end']}
  - iOS frameworks 使用 --no-default-features 編譯（無 TUI）
  - ❌ ratatui 和 crossterm 不支援 iOS（已排除）
  - ✅ 僅包含核心遊戲邏輯和 FFI 接口
  - 適合整合到 iOS/Swift 應用程式中
""")
