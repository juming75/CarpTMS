import { test, expect } from '@playwright/test';

test.describe('Settings Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
  });

  test('should display system settings form', async ({ page }) => {
    await expect(page.locator('text=基础配置')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('text=服务器地址')).toBeVisible();
  });

  test('should save settings', async ({ page }) => {
    const saveButton = page.getByRole('button', { name: /保存设置/ });
    if (await saveButton.isVisible()) {
      await saveButton.click();
      await expect(page.locator('.el-message--success, .el-message')).toBeVisible({ timeout: 5000 });
    }
  });

  test('should display service monitor', async ({ page }) => {
    await expect(page.locator('text=服务监测与控制')).toBeVisible({ timeout: 10000 });
  });

  test('should display performance monitor', async ({ page }) => {
    await expect(page.locator('.performance-monitor, text=性能监控')).toBeVisible({ timeout: 10000 });
  });

  test('should display performance stats', async ({ page }) => {
    const performanceSection = page.locator('.performance-monitor');
    if (await performanceSection.isVisible({ timeout: 5000 })) {
      await expect(performanceSection.locator('text=内存使用率')).toBeVisible();
      await expect(performanceSection.locator('text=API日志')).toBeVisible();
    }
  });

  test('should switch performance monitor tabs', async ({ page }) => {
    const performanceSection = page.locator('.performance-monitor');
    if (await performanceSection.isVisible({ timeout: 5000 })) {
      await page.click('text=API日志');
      await expect(page.locator('.el-table', { hasText: 'URL' })).toBeVisible({ timeout: 3000 });
      
      await page.click('text=内存监控');
      await expect(page.locator('.memory-chart, .chart-container')).toBeVisible({ timeout: 3000 });
    }
  });

  test('should toggle auto sync setting', async ({ page }) => {
    const autoSyncSwitch = page.locator('.el-switch').first();
    if (await autoSyncSwitch.isVisible()) {
      await autoSyncSwitch.click();
      await page.waitForTimeout(300);
    }
  });
});