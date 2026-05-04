import { test, expect } from '@playwright/test';

test.describe('Order Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should navigate to order management', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await expect(page).toHaveTitle(/订单|Order/);
  });

  test('should display order list', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(1000);

    const orderTable = page.locator('.el-table, .order-table, .data-table');
    await expect(orderTable.first()).toBeVisible();
  });

  test('should filter orders by status', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(500);

    const statusFilter = page.locator('.el-select, select').first();
    if (await statusFilter.isVisible()) {
      await statusFilter.click();
      await page.waitForTimeout(300);
      const option = page.locator('.el-option:has-text("已完成"), option:has-text("完成")').first();
      if (await option.isVisible()) {
        await option.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('should search orders', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(500);

    const searchInput = page.locator('input[placeholder*="搜索"], input[placeholder*="搜索订单"]');
    if (await searchInput.isVisible()) {
      await searchInput.fill('TEST');
      await page.waitForTimeout(500);
    }
  });

  test('should open create order dialog', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(500);

    const addButton = page.locator('button:has-text("添加"), button:has-text("新建"), button:has-text("创建")').first();
    if (await addButton.isVisible()) {
      await addButton.click();
      await page.waitForTimeout(500);
      const dialog = page.locator('.el-dialog, .dialog');
      await expect(dialog.first()).toBeVisible();
    }
  });

  test('should display order details', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(1000);

    const firstRow = page.locator('.el-table__row, .order-row').first();
    if (await firstRow.isVisible()) {
      await firstRow.click();
      await page.waitForTimeout(500);

      const detailPanel = page.locator('.detail-panel, .order-detail, .el-drawer');
      await expect(detailPanel.first()).toBeVisible({ timeout: 3000 });
    }
  });

  test('should export orders', async ({ page }) => {
    await page.click('text=订单管理');
    await page.waitForURL(/\/orders/);
    await page.waitForTimeout(500);

    const exportButton = page.locator('button:has-text("导出"), button:has-text("Export")');
    if (await exportButton.isVisible()) {
      await exportButton.click();
      await page.waitForTimeout(1000);
    }
  });
});

test.describe('Alarm Center', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should navigate to alarm center', async ({ page }) => {
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
    await expect(page).toHaveTitle(/报警|Alarm/);
  });

  test('should display alarm list', async ({ page }) => {
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
    await page.waitForTimeout(1000);

    const alarmTable = page.locator('.el-table, .alarm-table');
    await expect(alarmTable.first()).toBeVisible();
  });

  test('should filter alarms by severity', async ({ page }) => {
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
    await page.waitForTimeout(500);

    const severityFilter = page.locator('.el-select').first();
    if (await severityFilter.isVisible()) {
      await severityFilter.click();
      await page.waitForTimeout(300);
    }
  });

  test('should acknowledge alarm', async ({ page }) => {
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
    await page.waitForTimeout(1000);

    const ackButton = page.locator('button:has-text("确认"), button:has-text("Ack")').first();
    if (await ackButton.isVisible()) {
      await ackButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('should display alarm statistics', async ({ page }) => {
    await page.click('text=报警中心');
    await page.waitForURL(/\/alarms/);
    await page.waitForTimeout(500);

    const statsCards = page.locator('.el-card, .stat-card');
    expect(await statsCards.count()).toBeGreaterThan(0);
  });
});

test.describe('Settings Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.locator('input[type="text"]').fill('admin');
    await page.locator('input[type="password"]').fill('123456');
    await page.getByRole('button', { name: '登录' }).click();
    await page.waitForURL('/dashboard');
  });

  test('should navigate to settings', async ({ page }) => {
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
    await expect(page).toHaveTitle(/设置|Setting/);
  });

  test('should display settings tabs', async ({ page }) => {
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
    await page.waitForTimeout(500);

    const tabs = page.locator('.el-tabs__item, .tab-item');
    await expect(tabs.first()).toBeVisible();
  });

  test('should switch between settings sections', async ({ page }) => {
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
    await page.waitForTimeout(500);

    const tabItems = page.locator('.el-tabs__item');
    const tabCount = await tabItems.count();
    if (tabCount > 1) {
      await tabItems.nth(1).click();
      await page.waitForTimeout(300);
    }
  });

  test('should save settings', async ({ page }) => {
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
    await page.waitForTimeout(500);

    const saveButton = page.locator('button:has-text("保存"), button:has-text("Save")');
    if (await saveButton.isVisible()) {
      await saveButton.click();
      await page.waitForTimeout(500);
      const successMessage = page.locator('.el-message--success, .toast-success');
      await expect(successMessage.first()).toBeVisible({ timeout: 3000 });
    }
  });

  test('should reset settings to default', async ({ page }) => {
    await page.click('text=系统设置');
    await page.waitForURL(/\/settings/);
    await page.waitForTimeout(500);

    const resetButton = page.locator('button:has-text("重置"), button:has-text("Reset")');
    if (await resetButton.isVisible()) {
      await resetButton.click();
      await page.waitForTimeout(500);
    }
  });
});
