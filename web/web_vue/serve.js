#!/usr/bin/env node
/**
 * Vue开发服务器
 *
 * 功能：
 * 1. 启动HTTP服务器
 * 2. 热重载支持
 * 3. 代理API请求到8080端口
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 3000;
const API_PORT = 8080;

// MIME类型
const mimeTypes = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.svg': 'image/svg+xml'
};

const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  blue: '\x1b[34m',
  yellow: '\x1b[33m'
};

function log(msg, color = 'reset') {
  console.log(`${colors[color]}${msg}${colors.reset}`);
}

// 创建服务器
const server = http.createServer((req, res) => {
  // 处理API代理
  if (req.url.startsWith('/api')) {
    const options = {
      hostname: 'localhost',
      port: API_PORT,
      path: req.url.split('?')[1] ? req.url : req.url,
      method: req.method,
      headers: req.headers
    };

    const proxyReq = http.request(options, (proxyRes) => {
      res.writeHead(proxyRes.statusCode, proxyRes.headers);
      proxyRes.pipe(res);
    });

    proxyReq.on('error', (err) => {
      res.writeHead(502, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'API服务器未启动，请先启动游戏服务器' }));
    });

    req.pipe(proxyReq);
    return;
  }

  // 处理静态文件
  let filePath = '.' + req.url;
  if (req.url === '/') {
    filePath = './index.html';
  }

  const ext = path.extname(filePath);
  const contentType = mimeTypes[ext] || 'text/plain';

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/html' });
      res.end('<h1>404 - 文件未找到</h1>');
      return;
    }

    // 添加缓存破坏参数
    if (ext === '.html') {
      data = data.replace(/(\.css\?v=)[^"]+/g, '$1' + Date.now())
                  .replace(/(\.js\?v=)[^"]+/g, '$1' + Date.now());
    }

    res.writeHead(200, {
      'Content-Type': contentType,
      'Cache-Control': 'no-cache'
    });
    res.end(data);
  });
});

server.listen(PORT, () => {
  log(`开发服务器启动: http://localhost:${PORT}`, 'green');
  log(`API代理: http://localhost:${API_PORT}`, 'blue');
  log('\n按 Ctrl+C 停止服务器', 'yellow');
});
