// 启动 vite 服务的脚本
import { createServer } from 'vite';

async function start() {
  try {
    const server = await createServer({
      configFile: 'vite.config.ts'
    });
    await server.listen();
    console.log('Vite server started successfully!');
  } catch (error) {
    console.error('Error starting Vite server:', error);
    process.exit(1);
  }
}

start();