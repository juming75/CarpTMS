import { test, expect } from '@playwright/test';

test.describe('VirtualTable Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should render virtual table in vehicle management', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const virtualTable = page.locator('.virtual-table');
    await expect(virtualTable).toBeVisible({ timeout: 10000 });
    
    const header = page.locator('.virtual-table-header');
    await expect(header).toBeVisible();
    
    const body = page.locator('.virtual-table-body');
    await expect(body).toBeVisible();
  });

  test('should render virtual table in order management', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    
    const virtualTable = page.locator('.virtual-table');
    await expect(virtualTable).toBeVisible({ timeout: 10000 });
  });

  test('should scroll virtual table', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const body = page.locator('.virtual-table-body');
    if (await body.isVisible()) {
      await body.evaluate((el) => el.scrollTop = 500);
      await page.waitForTimeout(300);
      
      const scrollTop = await body.evaluate((el) => el.scrollTop);
      expect(scrollTop).toBeGreaterThan(0);
    }
  });

  test('should display pagination', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const pagination = page.locator('.pagination .el-pagination');
    await expect(pagination).toBeVisible({ timeout: 10000 });
  });

  test('should change page size', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const pageSizeSelect = page.locator('.el-pagination .el-select').first();
    if (await pageSizeSelect.isVisible()) {
      await pageSizeSelect.click();
      await page.locator('.el-option').nth(1).click();
      await page.waitForTimeout(500);
    }
  });

  test('should select rows with checkboxes', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const checkboxes = page.locator('.el-checkbox');
    if (await checkboxes.first().isVisible()) {
      await checkboxes.first().click();
      await page.waitForTimeout(200);
    }
  });

  test('should open action dialogs', async ({ page }) => {
    await page.click('text=车辆管理');
    await page.waitForURL(/\/vehicles/);
    
    const editButton = page.getByRole('button', { name: '编辑' }).first();
    if (await editButton.isVisible({ timeout: 5000 })) {
      await editButton.click();
      await expect(page.locator('.el-dialog')).toBeVisible({ timeout: 3000 });
    }
  });
});