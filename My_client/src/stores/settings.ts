import { defineStore } from 'pinia';
import { ref } from 'vue';
import api from '@/api';

export const useSettingsStore = defineStore('settings', () => {
  const home_page_name = ref('车辆运营监控平台');
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function load_settings() {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.get('/api/settings') as any;
      if (response) {
        home_page_name.value = response.home_page_name || '车辆运营监控平台';
      }
    } catch (err) {
      error.value = '加载设置失败';
      console.error('加载设置失败:', err);
    } finally {
      loading.value = false;
    }
  }

  async function loadHomePageName() {
    try {
      const response = await api.get('/api/settings') as any;
      if (response && response.home_page_name) {
        home_page_name.value = response.home_page_name;
      }
    } catch (err) {
      console.error('加载首页名称失败:', err);
    }
  }

  async function save_settings() {
    loading.value = true;
    error.value = null;
    try {
      await api.put('/api/settings', { home_page_name: home_page_name.value }) as any;
      return true;
    } catch (err) {
      error.value = '保存设置失败';
      console.error('保存设置失败:', err);
      return false;
    } finally {
      loading.value = false;
    }
  }

  return {
    home_page_name,
    loading,
    error,
    load_settings,
    save_settings,
    loadHomePageName,
  };
});


