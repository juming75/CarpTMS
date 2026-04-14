#!/bin/bash

set -e

echo "=== TMS Server Build Script ==="

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null
then
    echo "ERROR: Rust/cargo is not installed. Please install Rust first."
    exit 1
fi

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Cargo.toml not found. Please run this script from the project root directory."
    exit 1
fi

# 更新依赖
echo "Updating dependencies..."
cargo update

# 检查代码格式
echo "Checking code formatting..."
cargo fmt --all -- --check

# 运行 Clippy 进行静态分析
echo "Running Clippy static analysis..."
cargo clippy --all-targets -- -D warnings

# 构建项目
echo "Building project in release mode..."
cargo build --release

# 验证构建结果
if [ -f "target/release/tms_server" ]; then
    echo "✅ Build successful! Binary created at target/release/tms_server"
else
    echo "❌ Build failed! Binary not found."
    exit 1
fi

echo "=== Build completed successfully! ==="


