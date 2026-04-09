#!/bin/bash
# 同步前端文件到 dist 目录
SOURCE_DIR="/usr/local/games/rust/web/web_vue"
DIST_DIR="$SOURCE_DIR/dist"

echo "Syncing frontend files to dist..."
cp "$SOURCE_DIR/css/app.css" "$DIST_DIR/css/app.css"
cp "$SOURCE_DIR/js/app.js" "$DIST_DIR/js/app.js"
echo "Done! Files synced."
