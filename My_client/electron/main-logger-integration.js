/**
 * main.js 日志集成指南
 *
 * 此文件展示了如何在 main.js 中集成增强的日志系统
 *
 * 使用方法：
 * 1. 在 main.js 顶部添加导入：import logger from './logger.js'
 * 2. 替换所有 console.log 为 logger.info
 * 3. 替换所有 console.error 为 logger.error
 * 4. 在关键操作处添加上下文信息
 *
 * 示例：
 *
 * // 原代码：
 * console.log('TCP连接中:', { host, port })
 *
 * // 新代码：
 * logger.logTcpConnection('连接中', { host, port })
 *
 * // 原代码：
 * console.error('TCP连接错误:', error)
 *
 * // 新代码：
 * logger.error('TCP连接错误', { host, port }, error)
 */

/**
 * TCP 连接函数 - 集成日志的版本
 */
function connectTcpWithLogging(host, port) {
  const requestId = logger.generateRequestId();

  logger.info('TCP 连接开始', {
    requestId,
    host,
    port,
    type: 'tcp',
  });

  return new Promise((resolve) => {
    // 关闭现有连接
    if (tcpSocket) {
      logger.debug('关闭现有 TCP 连接', { requestId });
      tcpSocket.destroy();
      tcpSocket = null;
    }

    // 创建新连接
    tcpSocket = new net.Socket();
    tcpHost = host;
    tcpPort = port;

    // 设置超时
    const timeout = setTimeout(() => {
      logger.error('TCP 连接超时', {
        requestId,
        host,
        port,
        timeout: 10000,
      });
      tcpConnected = false;
      if (tcpSocket) {
        tcpSocket.destroy();
        tcpSocket = null;
      }
      resolve(false);
    }, 10000);

    // 连接成功
    tcpSocket.connect(port, host, () => {
      clearTimeout(timeout);
      tcpConnected = true;
      logger.logTcpConnection('连接成功', {
        requestId,
        host,
        port,
      });
      resolve(true);
    });

    // 连接关闭
    tcpSocket.on('close', () => {
      clearTimeout(timeout);
      tcpConnected = false;
      logger.logTcpConnection('连接关闭', { requestId });
    });

    // 连接错误
    tcpSocket.on('error', (error) => {
      clearTimeout(timeout);
      tcpConnected = false;
      logger.error(
        'TCP 连接错误',
        {
          requestId,
          host,
          port,
          errorType: error.code,
          errorMessage: error.message,
        },
        error
      );
      resolve(false);
    });

    // 接收数据
    tcpSocket.on('data', (data) => {
      logger.debug('TCP 接收数据', {
        requestId,
        dataSize: data.length,
        type: 'tcp',
      });

      try {
        const jsonStr = data.toString('utf8');
        const response = JSON.parse(jsonStr);

        logger.debug('TCP 接收到响应', {
          requestId,
          responseType: response.command || 'unknown',
          hasData: !!response.data,
        });

        // 处理消息队列
        if (tcpMessageQueue.length > 0) {
          const { resolve } = tcpMessageQueue.shift();
          resolve(response);
        }
      } catch (error) {
        logger.error(
          'TCP 数据解析失败',
          {
            requestId,
            dataSize: data.length,
          },
          error
        );

        if (tcpMessageQueue.length > 0) {
          const { reject } = tcpMessageQueue.shift();
          reject(new Error('数据解析失败'));
        }
      }
    });
  });
}

/**
 * TCP 发送数据函数 - 集成日志的版本
 */
function sendTcpDataWithLogging(data) {
  const requestId = logger.generateRequestId();

  logger.info('TCP 发送数据', {
    requestId,
    command: data.command || 'unknown',
    connected: tcpConnected,
    type: 'tcp',
  });

  return new Promise((resolve, reject) => {
    if (!tcpConnected || !tcpSocket) {
      logger.warn('TCP 未连接，尝试重连', { requestId });
      tcpMessageQueue.push({ data, resolve, reject, requestId });

      connectTcpWithLogging(tcpHost, tcpPort).then((connected) => {
        if (!connected) {
          logger.error('TCP 重连失败', { requestId });
          reject(new Error('无法连接到服务器'));
        }
      });
      return;
    }

    try {
      const jsonData = JSON.stringify(data);
      tcpSocket.write(jsonData);

      logger.debug('TCP 数据已发送', {
        requestId,
        dataSize: jsonData.length,
        command: data.command,
      });

      tcpMessageQueue.push({ data, resolve, reject, requestId });

      // 设置响应超时
      setTimeout(() => {
        if (tcpMessageQueue.length > 0) {
          const { reject } = tcpMessageQueue.shift();
          logger.error('TCP 发送超时', { requestId });
          reject(new Error('发送超时'));
        }
      }, 30000);
    } catch (error) {
      logger.error('TCP 发送数据失败', { requestId }, error);
      reject(error);
    }
  });
}

/**
 * IPC 处理器设置 - 集成日志的版本
 */
function setupIpcHandlersWithLogging() {
  // ====== TCP 客户端操作 ======

  ipcMain.handle('tcp-init', async (event, host, port) => {
    const requestId = logger.generateRequestId();
    logger.logIpcRequest('tcp-init', { requestId, host, port });

    try {
      tcpHost = host;
      tcpPort = port;

      logger.info('TCP 客户端初始化', {
        requestId,
        host,
        port,
      });

      const result = { success: true };
      logger.logIpcResponse('tcp-init', true, { requestId });
      return result;
    } catch (error) {
      logger.error('TCP 客户端初始化失败', { requestId }, error);
      logger.logIpcResponse('tcp-init', false, { requestId });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-connect', async () => {
    const requestId = logger.generateRequestId();
    logger.logIpcRequest('tcp-connect', { requestId });

    try {
      logger.info('TCP 客户端连接中...', { requestId });
      const connected = await connectTcpWithLogging(tcpHost, tcpPort);

      const result = { success: true, connected };
      logger.logIpcResponse('tcp-connect', true, { requestId, connected });
      return result;
    } catch (error) {
      logger.error('TCP 客户端连接失败', { requestId }, error);
      logger.logIpcResponse('tcp-connect', false, { requestId });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-send', async (event, data) => {
    const requestId = logger.generateRequestId();
    logger.logIpcRequest('tcp-send', { requestId, command: data.command });

    try {
      const response = await sendTcpDataWithLogging(data);

      logger.info('TCP 数据发送成功', {
        requestId,
        responseType: response.command || 'unknown',
      });

      const result = { success: true, response };
      logger.logIpcResponse('tcp-send', true, { requestId });
      return result;
    } catch (error) {
      logger.error('TCP 数据发送失败', { requestId }, error);
      logger.logIpcResponse('tcp-send', false, { requestId });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-is-connected', async () => {
    const requestId = logger.generateRequestId();
    logger.logIpcRequest('tcp-is-connected', { requestId });

    try {
      const result = { success: true, connected: tcpConnected };
      logger.logIpcResponse('tcp-is-connected', true, { requestId, connected: tcpConnected });
      return result;
    } catch (error) {
      logger.error('检查 TCP 连接状态失败', { requestId }, error);
      logger.logIpcResponse('tcp-is-connected', false, { requestId });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-disconnect', async () => {
    const requestId = logger.generateRequestId();
    logger.logIpcRequest('tcp-disconnect', { requestId });

    try {
      if (tcpSocket) {
        logger.info('关闭 TCP 连接', { requestId });
        tcpSocket.destroy();
        tcpSocket = null;
        tcpConnected = false;
      }

      const result = { success: true };
      logger.logIpcResponse('tcp-disconnect', true, { requestId });
      return result;
    } catch (error) {
      logger.error('TCP 客户端断开连接失败', { requestId }, error);
      logger.logIpcResponse('tcp-disconnect', false, { requestId });
      return { success: false, error: error.message };
    }
  });
}

export { connectTcpWithLogging, sendTcpDataWithLogging, setupIpcHandlersWithLogging };


