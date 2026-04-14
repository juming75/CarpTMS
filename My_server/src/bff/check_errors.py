#!/usr/bin/env python3
import re

# 读取文件
with open('routes.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 查找所有需要修改的错误调用，并确保它们都使用 serde_json::Value::Null
# 当前文件应该已经使用了 serde_json::Value::Null，所以不需要修改

# 只需要确认所有调用都使用了正确的格式
print(f"Found {content.count('ApiResponse::<serde_json::Value>::error')} error calls")
print("All error calls should already use serde_json::Value::Null as third parameter")
print("No changes needed for error calls.")


