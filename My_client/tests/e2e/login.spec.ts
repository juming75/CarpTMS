import { test, expect } from '@playwright/test';

test.describe('Login Page', () => {
  test('should display login form', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/登录/);
    await expect(page.locator('input[type="text"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.getByRole('button', { name: '登录' })).toBeVisible();
  });

  test('should show error for empty credentials', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: '登录' }).click();
    await expect(page.locator('.el-message--error')).toBeVisible();
  });

  test('should navigate to dashboard after login', async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
    await expect(page).toHaveTitle(/仪表盘/);
  });
});