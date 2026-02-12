# RataMUD C/C++ 範例 Makefile

CXX = g++
CC = gcc
CXXFLAGS = -std=c++17 -Wall -Wextra
CFLAGS = -Wall -Wextra
LDFLAGS = -L. -lratamud
ROOT_DIR = ..

# macOS 特定設定
ifeq ($(shell uname -s),Darwin)
    LDFLAGS += -Wl,-rpath,.
    LIB_EXT = dylib
    # Framework 設定
    FRAMEWORK_DIR = $(ROOT_DIR)/frameworks
    FRAMEWORK_FLAGS = -F$(FRAMEWORK_DIR) -framework RataMUD -framework CoreFoundation -Wl,-rpath,$(FRAMEWORK_DIR)
endif

# Linux 特定設定
ifeq ($(shell uname -s),Linux)
    LDFLAGS += -Wl,-rpath,.
    LIB_EXT = so
endif

# Windows 特定設定
ifeq ($(OS),Windows_NT)
    LIB_EXT = dll
endif

.PHONY: all clean test run-c run-cpp run help example-framework run-framework

all: example test

# 編譯 C 範例 (使用 dylib)
example: example.c ratamud.h libratamud.$(LIB_EXT)
	@echo "編譯 C 範例 (使用 dylib)..."
	$(CC) $(CFLAGS) -o example example.c $(LDFLAGS)
	@echo "✓ C 範例編譯完成"

# 編譯 C 範例 (使用 macOS Framework)
example-framework: example.c
	@echo "編譯 C 範例 (使用 macOS Framework)..."
	@if [ ! -d "$(FRAMEWORK_DIR)/RataMUD.framework" ]; then \
		echo "❌ Framework 不存在，請先執行: ./build_frameworks.sh"; \
		exit 1; \
	fi
	$(CC) $(CFLAGS) -o example example.c $(FRAMEWORK_FLAGS)
	@echo "✓ C 範例編譯完成 (Framework 版本)"

# 編譯 C++ 測試
test: test.cpp ratamud.h libratamud.$(LIB_EXT)
	@echo "編譯 C++ 測試..."
	$(CXX) $(CXXFLAGS) -o test test.cpp $(LDFLAGS)
	@echo "✓ C++ 測試編譯完成"

# 運行 C 範例（從根目錄運行以訪問地圖文件）
run-c: example
	@echo "\n========================================"
	@echo "運行 C 範例程式 (dylib 版本)"
	@echo "========================================"
	@echo "注意：需要從專案根目錄運行以訪問地圖文件"
	cd $(ROOT_DIR) && dist/example

# 運行 C 範例 (Framework 版本)
run-framework: example-framework
	@echo "\n========================================"
	@echo "運行 C 範例程式 (Framework 版本)"
	@echo "========================================"
	cd $(ROOT_DIR) && dist/example

# 運行 C++ 測試（從根目錄運行以訪問地圖文件）
run-cpp: test
	@echo "\n========================================"
	@echo "運行 C++ 測試程式"
	@echo "========================================"
	@echo "注意：需要從專案根目錄運行以訪問地圖文件"
	cd $(ROOT_DIR) && dist/test

# 快捷命令：運行 C 範例
run: run-c

# 運行所有測試
run-all: run-c run-cpp

# 清理
clean:
	@echo "清理編譯產物..."
	rm -f example test
	@echo "✓ 清理完成"

# 顯示幫助
help:
	@echo "RataMUD C/C++ 範例 Makefile"
	@echo ""
	@echo "可用目標:"
	@echo "  all               - 編譯所有範例（預設）"
	@echo "  example           - 編譯 C 範例 (dylib)"
	@echo "  example-framework - 編譯 C 範例 (macOS Framework)"
	@echo "  test              - 編譯 C++ 測試"
	@echo "  run               - 運行 C 範例 (dylib)"
	@echo "  run-c             - 運行 C 範例 (dylib)"
	@echo "  run-framework     - 運行 C 範例 (Framework)"
	@echo "  run-cpp           - 運行 C++ 測試"
	@echo "  run-all           - 運行所有測試"
	@echo "  clean             - 清理編譯產物"
	@echo "  help              - 顯示此幫助資訊"
	@echo ""
	@echo "範例:"
	@echo "  make example-framework  # 編譯 Framework 版本"
	@echo "  make run-framework      # 運行 Framework 版本"
	@echo "  make run                # 運行 dylib 版本"
	@echo "  make clean              # 清理"
	@echo ""
	@echo "注意：Framework 版本需要先執行 ./build_frameworks.sh"
