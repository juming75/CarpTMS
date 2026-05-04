import { test, expect } from '@playwright/test';

test.describe('Dashboard Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should display dashboard statistics cards', async ({ page }) => {
    await expect(page.locator('.stats-card, .el-card').first()).toBeVisible();
  });

  test('should navigate between menu items', async ({ page }) => {
    await page.click('text=报警中心');
    await expect(page).toHaveURL(/\/alarms/);
    
    await page.click('text=系统设置');
    await expect(page).toHaveURL(/\/settings/);
  });

  test('should display sidebar navigation', async ({ page }) => {
    await expect(page.locator('.sidebar, .el-menu, nav').first()).toBeVisible();
  });
});