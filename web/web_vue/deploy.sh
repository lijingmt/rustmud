#!/bin/bash
#
# Vue前端部署脚本
#
# 功能：构建并部署到web/dist
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_ROOT/web/dist"

echo "========================================="
echo "  Vue前端部署"
echo "========================================="
echo ""

# 1. 构建
echo "1. 构建前端..."
cd "$SCRIPT_DIR"
node build.js --prod

# 2. 创建目标目录
echo ""
echo "2. 创建目标目录..."
mkdir -p "$DIST_DIR"

# 3. 复制文件
echo "3. 复制文件..."
cp -r dist/* "$DIST_DIR/"

echo ""
echo "✓ 部署完成!"
echo ""
echo "访问地址:"
echo "  http://your-server/web/dist/"
echo ""
