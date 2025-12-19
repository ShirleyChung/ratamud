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
env['CC'] = 'gcc'
env['CXX'] = 'g++'
env['CFLAGS'] = ['-Wall', '-Wextra']
env['CXXFLAGS'] = ['-std=c++17', '-Wall', '-Wextra']

# 構建模式（release 或 debug）
build_mode = ARGUMENTS.get('mode', 'release')
env['BUILD_MODE'] = build_mode

if build_mode == 'debug':
    env.Append(CFLAGS=['-g', '-O0'])
    env.Append(CXXFLAGS=['-g', '-O0'])
    env['RUST_TARGET_DIR'] = env['RUST_DEBUG_DIR']
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
    return 0

# 註冊 Rust 構建器
rust_bld = Builder(action=rust_builder)
env.Append(BUILDERS={'RustLib': rust_bld})

# 導出環境變數給 SConscript
Export('env')

# 調用子構建腳本（不使用 variant_dir 以避免路徑問題）
SConscript('dist/SConscript', exports='env')

# 別名（由 SConscript 設置）
Alias('examples', [f'dist/example', f'dist/test'])
Alias('all', ['lib', 'examples'])

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
  all       - 構建所有（預設）
  lib       - 僅構建 Rust 函式庫
  examples  - 僅構建 C/C++ 範例
  
{colors['green']}選項:{colors['end']}
  mode=release|debug  - 構建模式（預設: release）
  -c                  - 清理構建產物
  -j N                - 使用 N 個並行任務

{colors['green']}範例:{colors['end']}
  scons                    # 構建所有（release 模式）
  scons mode=debug         # 構建所有（debug 模式）
  scons lib                # 僅構建函式庫
  scons examples           # 僅構建範例
  scons -c                 # 清理
  scons -j 4               # 使用 4 個並行任務構建

{colors['green']}文件:{colors['end']}
  SConstruct          - 主構建文件
  dist/SConscript     - 範例程序構建文件
""")
