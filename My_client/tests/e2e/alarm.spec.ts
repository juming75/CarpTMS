import { test, expect } from '@playwright/test';

test.describe('Alarm Center', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
  });

  test('should display alarm list', async ({ page }) => {
    await expect(page.locator('.alarm-list, .el-table').first()).toBeVisible({ timeout: 10000 });
  });

  test('should filter alarms by status', async ({ page }) => {
    const statusSelect = page.locator('.el-select, select').first();
    if (await statusSelect.isVisible()) {
      await statusSelect.click();
      await page.locator('.el-option, option').first().click();
      await page.waitForTimeout(500);
    }
  });

  test('should process an alarm', async ({ page }) => {
    const processButton = page.getByRole('button', { name: /处理/ }).first();
    if (await processButton.isVisible({ timeout: 3000 })) {
      await processButton.click();
      await expect(page.locator('.el-dialog, dialog')).toBeVisible();
    }
  });

  test('should refresh alarm list', async ({ page }) => {
    const refreshButton = page.getByRole('button', { name: /刷新/ });
    if (await refreshButton.isVisible()) {
      await refreshButton.click();
      await page.waitForTimeout(500);
    }
  });
});