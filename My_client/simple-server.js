// 简单的 HTTP 服务器，用于提供前端文件
import http from 'http';
import fs from 'fs';
import path from 'path';

const PORT = 5173;
const PUBLIC_DIR = path.join(process.cwd(), 'public');
const DIST_DIR = path.join(process.cwd(), 'dist');

// 静态文件类型映射
const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.jsx': 'application/javascript',
  '.ts': 'application/javascript',
  '.tsx': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon'
};

// 创建 HTTP 服务器
const server = http.createServer((req, res) => {
  let filePath = '';
  
  // 处理根路径
  if (req.url === '/') {
    filePath = path.join(PUBLIC_DIR, 'index.html');
  } else if (req.url === '/src/main.ts') {
    // 特殊处理 main.ts
    filePath = path.join(process.cwd(), 'src', 'main.ts');
  } else if (req.url.startsWith('/src/')) {
    // 处理 src 目录下的文件
    filePath = path.join(process.cwd(), req.url);
  } else {
    // 处理其他路径
    filePath = path.join(PUBLIC_DIR, req.url);
  }
  
  // 读取文件
  fs.readFile(filePath, (err, content) => {
    if (err) {
      // 如果文件不存在，返回 index.html（用于 SPA 路由）
      fs.readFile(path.join(PUBLIC_DIR, 'index.html'), (err, indexContent) => {
        if (err) {
          res.writeHead(500);
          res.end('Internal Server Error');
        } else {
          res.writeHead(200, { 'Content-Type': 'text/html' });
          res.end(indexContent, 'utf-8');
        }
      });
    } else {
      // 确定文件类型
      const extname = path.extname(filePath);
      const contentType = MIME_TYPES[extname] || 'application/octet-stream';
      
      // 返回文件内容
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content, 'utf-8');
    }
  });
});

// 启动服务器
server.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}/`);
  console.log('Note: This is a simple HTTP server, not a full Vite development server.');
  console.log('Some Vite features like HMR may not work.');
});