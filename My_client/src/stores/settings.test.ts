import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore } from './settings';

// Mock the API
vi.mock('@/api', () => ({
  default: {
    get: vi.fn(),
    put: vi.fn(),
  },
}));

import api from '@/api';

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('initial state', () => {
    it('should have correct initial values', () => {
      const store = useSettingsStore();
      expect(store.home_page_name).toBe('车辆运营监控平台');
      expect(store.loading).toBe(false);
      expect(store.error).toBeNull();
    });
  });

  describe('load_settings', () => {
    it('should load settings successfully', async () => {
      const store = useSettingsStore();
      const mockResponse = { home_page_name: '新平台名称' };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      await store.load_settings();

      expect(api.get).toHaveBeenCalledWith('/api/settings');
      expect(store.home_page_name).toBe('新平台名称');
      expect(store.loading).toBe(false);
      expect(store.error).toBeNull();
    });

    it('should use default if no home_page_name in response', async () => {
      const store = useSettingsStore();
      (api.get as any).mockResolvedValueOnce({});

      await store.load_settings();

      expect(store.home_page_name).toBe('车辆运营监控平台');
    });

    it('should handle load error', async () => {
      const store = useSettingsStore();
      (api.get as any).mockRejectedValueOnce(new Error('网络错误'));

      await store.load_settings();

      expect(store.error).toBe('加载设置失败');
      expect(store.loading).toBe(false);
    });
  });

  describe('loadHomePageName', () => {
    it('should load homepage name successfully', async () => {
      const store = useSettingsStore();
      const mockResponse = { home_page_name: '测试平台' };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      await store.loadHomePageName();

      expect(store.home_page_name).toBe('测试平台');
    });

    it('should not change name if not provided', async () => {
      const store = useSettingsStore();
      store.home_page_name = '已设置名称';
      (api.get as any).mockResolvedValueOnce({});

      await store.loadHomePageName();

      expect(store.home_page_name).toBe('已设置名称');
    });

    it('should handle error without setting error state', async () => {
      const store = useSettingsStore();
      (api.get as any).mockRejectedValueOnce(new Error('网络错误'));

      await store.loadHomePageName();

      expect(store.error).toBeNull();
    });
  });

  describe('save_settings', () => {
    it('should save settings successfully', async () => {
      const store = useSettingsStore();
      store.home_page_name = '要保存的名称';
      (api.put as any).mockResolvedValueOnce({});

      const result = await store.save_settings();

      expect(result).toBe(true);
      expect(api.put).toHaveBeenCalledWith('/api/settings', {
        home_page_name: '要保存的名称',
      });
      expect(store.loading).toBe(false);
    });

    it('should handle save error', async () => {
      const store = useSettingsStore();
      (api.put as any).mockRejectedValueOnce(new Error('保存失败'));

      const result = await store.save_settings();

      expect(result).toBe(false);
      expect(store.error).toBe('保存设置失败');
      expect(store.loading).toBe(false);
    });
  });
});
