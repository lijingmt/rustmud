#!/usr/bin/env node
/**
 * Vue前端构建脚本
 *
 * 功能：
 * 1. 编译/复制文件到 web/web_vue/ 目录
 * 2. 生成版本号
 */

const fs = require('fs');
const path = require('path');

const isProd = process.argv.includes('--prod');

// 颜色输出
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m'
};

function log(msg, color = 'reset') {
  console.log(`${colors[color]}${msg}${colors.reset}`);
}

// 输出目录: web/web_vue/
const distDir = path.join(__dirname, '..', 'web', 'web_vue');
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// 版本号
const version = `v${Date.now()}`;
log(`构建版本: ${version}`, 'blue');

// 复制文件
function copyFile(src, dest, transform = null) {
  const content = fs.readFileSync(src, 'utf8');
  const processed = transform ? transform(content) : content;
  fs.writeFileSync(dest, processed);
  log(`✓ ${path.basename(src)}`, 'green');
}

// 处理HTML - 注入版本号
function processHTML(content) {
  return content.replace(/(\.css\?v=)[^"]+/, `$1${version}`)
             .replace(/(\.js\?v=)[^"]+/, `$1${version}`);
}

// 构建步骤
log('开始构建...', 'blue');

// 1. 复制HTML
log('\n1. HTML:', 'yellow');
copyFile('index.html', path.join(distDir, 'index.html'), processHTML);

// 2. 复制并处理CSS
log('\n2. CSS:', 'yellow');
fs.mkdirSync(path.join(distDir, 'css'), { recursive: true });
copyFile('css/app.css', path.join(distDir, 'css/app.css'));

// 3. 复制JS
log('\n3. JS:', 'yellow');
fs.mkdirSync(path.join(distDir, 'js'), { recursive: true });
copyFile('js/app.js', path.join(distDir, 'js/app.js'));

// 4. 生成manifest.json (PWA支持)
log('\n4. Manifest:', 'yellow');
const manifest = {
  name: '天下AI网游',
  short_name: 'TXPike9',
  version: version,
  start_url: '/',
  display: 'standalone',
  background_color: '#1a1a2e',
  theme_color: '#667eea',
  icons: [{
    src: '/icon.png',
    sizes: '192x192',
    type: 'image/png'
  }]
};
fs.writeFileSync(
  path.join(distDir, 'manifest.json'),
  JSON.stringify(manifest, null, 2)
);
log('✓ manifest.json', 'green');

// 完成
log('\n✓ 构建完成!', 'green');
log(`输出目录: ${distDir}`, 'blue');
