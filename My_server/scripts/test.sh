#!/bin/bash

set -e

echo "=== TMS Server Test Script ==="

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

# 运行所有测试
echo "Running all tests..."
cargo test --all

# 运行特定测试（可选，根据需要取消注释）
# echo "Running specific tests..."
# cargo test test_user_crud
# cargo test test_vehicle_crud

# 运行集成测试
echo "Running integration tests..."
cargo test --test users_test
cargo test --test vehicles_test

# 显示测试覆盖率（可选，需要安装 cargo-tarpaulin）
if command -v cargo-tarpaulin &> /dev/null
then
    echo "Generating test coverage report..."
    cargo tarpaulin --out Html --out Xml
    echo "✅ Coverage report generated!"
else
    echo "⚠️  cargo-tarpaulin not installed, skipping coverage report. Run 'cargo install cargo-tarpaulin' to install."
fi

echo "=== All tests completed successfully! ==="


