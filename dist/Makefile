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

.PHONY: all clean test run-c run-cpp run help

all: example test

# 編譯 C 範例
example: example.c ratamud.h libratamud.$(LIB_EXT)
	@echo "編譯 C 範例..."
	$(CC) $(CFLAGS) -o example example.c $(LDFLAGS)
	@echo "✓ C 範例編譯完成"

# 編譯 C++ 測試
test: test.cpp ratamud.h libratamud.$(LIB_EXT)
	@echo "編譯 C++ 測試..."
	$(CXX) $(CXXFLAGS) -o test test.cpp $(LDFLAGS)
	@echo "✓ C++ 測試編譯完成"

# 運行 C 範例（從根目錄運行以訪問地圖文件）
run-c: example
	@echo "\n========================================"
	@echo "運行 C 範例程式"
	@echo "========================================"
	@echo "注意：需要從專案根目錄運行以訪問地圖文件"
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
	@echo "  all       - 編譯所有範例（預設）"
	@echo "  example   - 編譯 C 範例"
	@echo "  test      - 編譯 C++ 測試"
	@echo "  run       - 運行 C 範例（從根目錄）"
	@echo "  run-c     - 運行 C 範例（從根目錄）"
	@echo "  run-cpp   - 運行 C++ 測試（從根目錄）"
	@echo "  run-all   - 運行所有測試"
	@echo "  clean     - 清理編譯產物"
	@echo "  help      - 顯示此幫助資訊"
	@echo ""
	@echo "範例:"
	@echo "  make              # 編譯所有範例"
	@echo "  make run          # 運行 C 範例"
	@echo "  make run-cpp      # 運行 C++ 測試"
	@echo "  make clean        # 清理"
	@echo ""
	@echo "注意：程式會從專案根目錄執行以訪問 worlds/初始世界/maps/ 目錄"
