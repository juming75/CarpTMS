<template>
  <div class="location-settings">
    <div class="location-header">
      <h2>地图设置</h2>
    </div>

    <div class="location-tabs">
      <el-tabs v-model="activeTab" type="card">
        <!-- 地图设置 -->
        <el-tab-pane label="地图设置" name="mapSettings">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>地图API密钥设置</span>
                  <el-button type="primary" size="small" @click="handleSaveMapSettings">
                    <el-icon><Check /></el-icon> 保存设置
                  </el-button>
                </div>
              </template>
              <div class="map-settings-form">
                <el-form :model="mapSettings" label-width="150px">
                  <el-form-item label="天地图API Key">
                    <el-input
                      v-model="mapSettings.tiandituKey"
                      placeholder="请输入天地图API Key"
                      show-password
                      clearable
                    />
                    <div class="form-tip">
                      <el-link type="primary" href="https://console.tianditu.gov.cn/api/key" target="_blank">
                        获取天地图API Key
                      </el-link>
                    </div>
                  </el-form-item>

                  <el-form-item label="高德地图API Key">
                    <el-input
                      v-model="mapSettings.gaodeKey"
                      placeholder="请输入高德地图API Key"
                      show-password
                      clearable
                    />
                    <div class="form-tip">
                      <el-link type="primary" href="https://console.amap.com/dev/key/app" target="_blank">
                        获取高德地图API Key
                      </el-link>
                    </div>
                  </el-form-item>

                  <el-form-item label="百度地图API Key">
                    <el-input
                      v-model="mapSettings.baiduKey"
                      placeholder="请输入百度地图API Key"
                      show-password
                      clearable
                    />
                    <div class="form-tip">
                      <el-link type="primary" href="http://lbsyun.baidu.com/apiconsole/key" target="_blank">
                        获取百度地图API Key
                      </el-link>
                    </div>
                  </el-form-item>

                  <el-form-item label="默认地图类型">
                    <el-radio-group v-model="mapSettings.defaultMapType">
                      <el-radio value="tianditu">天地图</el-radio>
                      <el-radio value="gaode">高德地图</el-radio>
                      <el-radio value="baidu">百度地图</el-radio>
                    </el-radio-group>
                  </el-form-item>
                </el-form>

                <el-divider />

                <div class="map-test-section">
                  <h4>地图连接测试</h4>
                  <div class="test-buttons">
                    <el-button type="primary" size="small" @click="testTiandituConnection"> 测试天地图连接 </el-button>
                    <el-button type="success" size="small" @click="testGaodeConnection"> 测试高德地图连接 </el-button>
                    <el-button type="warning" size="small" @click="testBaiduConnection"> 测试百度地图连接 </el-button>
                  </div>
                  <div v-if="testResult" class="test-result" :class="testResult.type">
                    <el-alert :title="testResult.message" :type="testResult.type" show-icon />
                  </div>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
/* global fetch */
import { ref, reactive, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Check } from '@element-plus/icons-vue';

// 当前激活的标签页
const activeTab = ref('mapSettings');

// 地图设置
const mapSettings = reactive({
  tiandituKey: localStorage.getItem('tiandituKey') || '',
  gaodeKey: localStorage.getItem('gaodeKey') || '',
  baiduKey: localStorage.getItem('baiduKey') || '',
  defaultMapType: localStorage.getItem('defaultMapType') || 'tianditu',
});

// 测试结果
const testResult = ref<{ type: 'success' | 'error' | 'warning'; message: string } | null>(null);

// 保存地图设置
const handleSaveMapSettings = () => {
  localStorage.setItem('tiandituKey', mapSettings.tiandituKey);
  localStorage.setItem('gaodeKey', mapSettings.gaodeKey);
  localStorage.setItem('baiduKey', mapSettings.baiduKey);
  localStorage.setItem('defaultMapType', mapSettings.defaultMapType);
  ElMessage.success('地图设置保存成功');
};

interface TiandituTestResult {
  status: string;
  msg?: string;
}

interface _GaodeTestResult {
  status: string;
  info?: string;
}

interface _BaiduTestResult {
  status: number;
  msg?: string;
}

// 测试天地图连接
const testTiandituConnection = async () => {
  if (!mapSettings.tiandituKey) {
    testResult.value = { type: 'error', message: '请先输入天地图API Key' };
    return;
  }
  try {
    // 使用JSONP方式测试天地图
    const script = document.createElement('script');
    const callbackName = 'tiandituTestCallback_' + Date.now();

    window[callbackName] = (result: TiandituTestResult) => {
      if (result && result.status === '0') {
        testResult.value = { type: 'success', message: '天地图连接测试成功！' };
      } else {
        testResult.value = { type: 'error', message: '天地图连接测试失败：' + (result?.msg || '未知错误') };
      }
      delete window[callbackName];
      document.head.removeChild(script);
    };

    script.src = `https://api.tianditu.gov.cn/geocoder?ds={"keyWord":"北京市"}&tk=${mapSettings.tiandituKey}&callback=${callbackName}`;
    script.onerror = () => {
      testResult.value = { type: 'error', message: '天地图连接测试失败：网络错误' };
      delete window[callbackName];
      document.head.removeChild(script);
    };

    document.head.appendChild(script);

    // 5秒超时
    setTimeout(() => {
      if (window[callbackName]) {
        testResult.value = { type: 'warning', message: '天地图连接测试超时' };
        delete window[callbackName];
        if (script.parentNode) {
          document.head.removeChild(script);
        }
      }
    }, 5000);
  } catch (error) {
    testResult.value = { type: 'error', message: '天地图连接测试失败：' + error };
  }
};

// 测试高德地图连接
const testGaodeConnection = async () => {
  if (!mapSettings.gaodeKey) {
    testResult.value = { type: 'error', message: '请先输入高德地图API Key' };
    return;
  }
  try {
    const response = await fetch(`https://restapi.amap.com/v3/geocode/geo?address=北京市&key=${mapSettings.gaodeKey}`);
    const data = await response.json();
    if (data.status === '1') {
      testResult.value = { type: 'success', message: '高德地图连接测试成功！' };
    } else {
      testResult.value = { type: 'error', message: '高德地图连接测试失败：' + (data.info || '未知错误') };
    }
  } catch (_error) {
    testResult.value = { type: 'error', message: '高德地图连接测试失败：网络错误' };
  }
};

// 测试百度地图连接
const testBaiduConnection = async () => {
  if (!mapSettings.baiduKey) {
    testResult.value = { type: 'error', message: '请先输入百度地图API Key' };
    return;
  }
  try {
    const response = await fetch(
      `https://api.map.baidu.com/geocoding/v3/?address=北京市&output=json&ak=${mapSettings.baiduKey}`
    );
    const data = await response.json();
    if (data.status === 0) {
      testResult.value = { type: 'success', message: '百度地图连接测试成功！' };
    } else {
      testResult.value = { type: 'error', message: '百度地图连接测试失败：' + (data.msg || '未知错误') };
    }
  } catch (_error) {
    testResult.value = { type: 'error', message: '百度地图连接测试失败：网络错误' };
  }
};
</script>

<style scoped>
.location-settings {
  padding: 20px;
}

.location-header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.location-tabs {
  margin-top: 20px;
}

.tab-content {
  min-height: 400px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.map-settings-form {
  max-width: 600px;
  padding: 20px;
}

.form-tip {
  margin-top: 8px;
  font-size: 12px;
}

.map-test-section {
  margin-top: 20px;
}

.map-test-section h4 {
  margin-bottom: 16px;
  color: #303133;
}

.test-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.test-result {
  margin-top: 16px;
}
</style>
