// Simple HTTP server for serving WASM test files
const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 8080;
const ROOT_DIR = path.join(__dirname, '../..');

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.wasm': 'application/wasm',
  '.json': 'application/json',
  '.css': 'text/css',
};

const server = http.createServer((req, res) => {
  // Remove query string
  const url = req.url.split('?')[0];
  const filePath = path.join(ROOT_DIR, url);
  
  console.log(`Request: ${req.method} ${url}`);
  
  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404);
      res.end('404 Not Found');
      console.error(`404: ${filePath}`);
      return;
    }
    
    const ext = path.extname(filePath);
    const mimeType = MIME_TYPES[ext] || 'application/octet-stream';
    
    res.writeHead(200, {
      'Content-Type': mimeType,
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Access-Control-Allow-Origin': '*',
    });
    res.end(data);
    
    console.log(`200: ${filePath} (${mimeType})`);
  });
});

server.listen(PORT, () => {
  console.log(`HTTP server running at http://localhost:${PORT}/`);
  console.log(`Root directory: ${ROOT_DIR}`);
  console.log(`Test page: http://localhost:${PORT}/tests/wasm/test.html`);
});

// Graceful shutdown
process.on('SIGINT', () => {
  console.log('\nShutting down server...');
  server.close(() => {
    console.log('Server stopped');
    process.exit(0);
  });
});

