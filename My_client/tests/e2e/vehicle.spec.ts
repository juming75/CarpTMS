import { test, expect } from '@playwright/test';

test.describe('Vehicle Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should navigate to vehicle management page', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL('/vehicles');
    await expect(page).toHaveTitle(/车辆管理/);
  });

  test('should display vehicle list with VirtualTable', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL('/vehicles');
    await expect(page.locator('.virtual-table')).toBeVisible();
    await expect(page.locator('.virtual-table-header')).toBeVisible();
    await expect(page.locator('.virtual-table-body')).toBeVisible();
  });

  test('should search vehicles', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL('/vehicles');
    await page.locator('input[placeholder="搜索车辆"]').fill('测试车辆');
    await page.waitForTimeout(500);
  });

  test('should open add vehicle dialog', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL('/vehicles');
    await page.getByRole('button', { name: '添加车辆' }).click();
    await expect(page.locator('.el-dialog')).toBeVisible();
    await expect(page.locator('.el-dialog__title')).toContainText('添加车辆');
  });
});