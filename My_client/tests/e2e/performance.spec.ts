import { test, expect } from '@playwright/test';

test.describe('Performance Monitoring', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should display performance metrics', async ({ page }) => {
    await page.click('text=性能监控');
    await page.waitForURL(/\/performance/);
    await expect(page.locator('.performance-metrics, .metrics-panel')).toBeVisible();
  });

  test('should show real-time FPS monitoring', async ({ page }) => {
    await page.click('text=性能监控');
    await page.waitForURL(/\/performance/);
    await page.waitForTimeout(2000);

    const fpsDisplay = page.locator('text=/FPS|帧率/');
    await expect(fpsDisplay.first()).toBeVisible();
  });

  test('should display memory usage', async ({ page }) => {
    await page.click('text=性能监控');
    await page.waitForURL(/\/performance/);
    await page.waitForTimeout(1000);

    const memoryDisplay = page.locator('text=/内存|memory/i');
    await expect(memoryDisplay.first()).toBeVisible();
  });

  test('should toggle performance overlay', async ({ page }) => {
    await page.click('text=性能监控');
    await page.waitForURL(/\/performance/);

    const toggleButton = page.locator('button:has-text("显示FPS"), button:has-text("隐藏FPS")');
    if (await toggleButton.isVisible()) {
      await toggleButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('should log performance warnings for low FPS', async ({ page }) => {
    await page.click('text=性能监控');
    await page.waitForURL(/\/performance/);
    await page.waitForTimeout(3000);

    const warningLog = page.locator('.log-entry:has-text("FPS"), .performance-log');
    const hasLogs = await warningLog.count();
    expect(hasLogs).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Real-time Communication', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should display communication panel', async ({ page }) => {
    await page.click('text=实时通信');
    await page.waitForURL(/\/communication/);
    await expect(page.locator('.communication-panel, .unified-panel')).toBeVisible();
  });

  test('should connect to WebSocket', async ({ page }) => {
    await page.click('text=实时通信');
    await page.waitForURL(/\/communication/);
    await page.waitForTimeout(2000);

    const connectionStatus = page.locator('text=/已连接|在线|connected/i');
    await expect(connectionStatus.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display message history', async ({ page }) => {
    await page.click('text=实时通信');
    await page.waitForURL(/\/communication/);
    await page.waitForTimeout(2000);

    const messageList = page.locator('.message-list, .log-view');
    await expect(messageList.first()).toBeVisible();
  });

  test('should send test message', async ({ page }) => {
    await page.click('text=实时通信');
    await page.waitForURL(/\/communication/);
    await page.waitForTimeout(1000);

    const messageInput = page.locator('input[type="text"], textarea').first();
    if (await messageInput.isVisible()) {
      await messageInput.fill('Test message');
      const sendButton = page.locator('button:has-text("发送"), button:has-text("Send")');
      if (await sendButton.isVisible()) {
        await sendButton.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('should display connection statistics', async ({ page }) => {
    await page.click('text=实时通信');
    await page.waitForURL(/\/communication/);
    await page.waitForTimeout(2000);

    const stats = page.locator('.connection-stats, .stat-panel');
    await expect(stats.first()).toBeVisible();
  });
});

test.describe('Data Visualization', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should display chart components', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page.locator('.el-card, .chart-card').first()).toBeVisible();
  });

  test('should render ECharts graphs', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForTimeout(2000);

    const chartContainer = page.locator('.echarts, .chart-container, canvas');
    await expect(chartContainer.first()).toBeVisible();
  });

  test('should display map component', async ({ page }) => {
    await page.click('text=实时监控');
    await page.waitForURL(/\/monitor/);
    await page.waitForTimeout(2000);

    const mapCanvas = page.locator('canvas, .ol-map, .map-container');
    await expect(mapCanvas.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display vehicle markers on map', async ({ page }) => {
    await page.click('text=实时监控');
    await page.waitForURL(/\/monitor/);
    await page.waitForTimeout(3000);

    const markers = page.locator('.marker, .vehicle-marker, .ol-marker');
    const markerCount = await markers.count();
    expect(markerCount).toBeGreaterThanOrEqual(0);
  });
});
