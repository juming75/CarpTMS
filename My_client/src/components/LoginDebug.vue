<template>
  <div class="login-debug">
    <el-dialog
      v-model="visible"
      title="登录状态调试"
      width="800px"
      :close-on-click-modal="false"
    >
      <div class="debug-content">
        <!-- 登录状态概览 -->
        <el-card class="status-card">
          <template #header>
            <div class="card-header">
              <span>登录状态概览</span>
              <el-button type="primary" size="small" @click="refreshStatus">刷新状态</el-button>
            </div>
          </template>
          <div class="status-grid">
            <div class="status-item">
              <label>Token存在:</label>
              <el-tag :type="hasToken ? 'success' : 'danger'">
                {{ hasToken ? '是' : '否' }}
              </el-tag>
            </div>

            <div class="status-item">
              <label>UserInfo存在:</label>
              <el-tag :type="hasUserInfo ? 'success' : 'danger'">
                {{ hasUserInfo ? '是' : '否' }}
              </el-tag>
            </div>
            <div class="status-item">
              <label>当前路径:</label>
              <el-tag type="info">{{ currentPath }}</el-tag>
            </div>
          </div>
        </el-card>

        <!-- Token信息 -->
        <el-card class="token-card" v-if="hasToken">
          <template #header>
            <div class="card-header">
              <span>Token信息</span>
              <el-button type="danger" size="small" @click="clearToken">清除Token</el-button>
            </div>
          </template>
          <div class="token-content">
            <el-input
              v-model="token"
              type="textarea"
              rows="3"
              readonly
              placeholder="Token信息"
            />
          </div>
        </el-card>

        <!-- UserInfo -->
        <el-card class="userinfo-card" v-if="hasUserInfo">
          <template #header>
            <div class="card-header">
              <span>用户信息</span>
              <el-button type="danger" size="small" @click="clearUserInfo">清除用户信息</el-button>
            </div>
          </template>
          <div class="userinfo-content">
            <el-input
              v-model="userInfoStr"
              type="textarea"
              rows="5"
              readonly
              placeholder="用户信息"
            />
          </div>
        </el-card>

        <!-- API请求日志 -->
        <el-card class="logs-card">
          <template #header>
            <div class="card-header">
              <span>API请求日志</span>
              <el-button type="danger" size="small" @click="clearLogs">清空日志</el-button>
            </div>
          </template>
          <div class="logs-content">
            <el-scrollbar height="300px">
              <div class="log-item" v-for="(log, index) in logs" :key="index">
                <div class="log-header" :class="log.type">
                  <span class="log-time">{{ log.timestamp }}</span>
                  <span class="log-type">{{ log.type === 'success' ? '✅' : '❌' }}</span>
                  <span class="log-url">{{ log.url }}</span>
                  <el-tag size="small" :type="log.type">
                    {{ log.method }}
                  </el-tag>
                </div>
                <div class="log-details" v-if="log.details">
                  <pre>{{ JSON.stringify(log.details, null, 2) }}</pre>
                </div>
              </div>
              <div class="no-logs" v-if="logs.length === 0">
                暂无API请求日志
              </div>
            </el-scrollbar>
          </div>
        </el-card>
      </div>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="visible = false">关闭</el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 触发按钮 -->
    <el-button
      class="debug-trigger"
      type="primary"
      size="small"
      circle
      @click="visible = true"
      title="登录状态调试"
    >
      <el-icon><Setting /></el-icon>
    </el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Setting } from '@element-plus/icons-vue'

const route = useRoute()
const visible = ref(false)
const logs = ref<any[]>([])

// 刷新状态
const refreshStatus = () => {
  checkStatus()
  ElMessage.success('状态已刷新')
}

// 检查登录状态
const token = ref('')
const userInfo = ref<any>(null)
const hasToken = ref(false)

const hasUserInfo = ref(false)
const currentPath = ref(route.path)

const checkStatus = () => {
  const storedToken = localStorage.getItem('token')
  const storedUserInfo = localStorage.getItem('userInfo')
  
  token.value = storedToken || ''
  hasToken.value = !!storedToken
  
  try {
    userInfo.value = storedUserInfo ? JSON.parse(storedUserInfo) : null
    hasUserInfo.value = !!userInfo.value
  } catch {
    userInfo.value = null
    hasUserInfo.value = false
  }
  
  currentPath.value = route.path
}

// 用户信息字符串形式
const userInfoStr = computed(() => {
  return JSON.stringify(userInfo.value, null, 2)
})

// 监听路由变化
watch(
  () => route.path,
  (newPath) => {
    currentPath.value = newPath
  }
)

// 清除Token
const clearToken = () => {
  localStorage.removeItem('token')
  checkStatus()
  ElMessage.warning('Token已清除')
}

// 清除用户信息
const clearUserInfo = () => {
  localStorage.removeItem('userInfo')
  checkStatus()
  ElMessage.warning('用户信息已清除')
}

// 清除日志
const clearLogs = () => {
  logs.value = []
  ElMessage.warning('日志已清空')
}

// 监听API请求日志
const originalConsoleLog = console.log
const originalConsoleError = console.error

// 重写console.log和console.error来捕获API日志
console.log = (...args) => {
  originalConsoleLog(...args)
  // 检查是否是API请求日志
  if (args[0] && typeof args[0] === 'string') {
    if (args[0].includes('✅ API请求成功')) {
      logs.value.push({
        type: 'success',
        timestamp: new Date().toLocaleTimeString(),
        ...args[1]
      })
    }
  }
}

console.error = (...args) => {
  originalConsoleError(...args)
  // 检查是否是API请求错误日志
  if (args[0] && typeof args[0] === 'string') {
    if (args[0].includes('❌ API请求错误')) {
      logs.value.push({
        type: 'error',
        timestamp: new Date().toLocaleTimeString(),
        ...args[1]
      })
    }
  }
}

onMounted(() => {
  checkStatus()
})
</script>

<style scoped>
.login-debug {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 9999;
}

.debug-trigger {
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
}

.debug-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item label {
  font-weight: 500;
  color: #606266;
  width: 120px;
}

.token-content,
.userinfo-content {
  margin-top: 12px;
}

.logs-content {
  margin-top: 12px;
}

.log-item {
  margin-bottom: 12px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  overflow: hidden;
}

.log-header {
  padding: 8px 16px;
  background-color: #f5f7fa;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.log-header.success {
  background-color: #f0f9eb;
  border-left: 4px solid #67c23a;
}

.log-header.error {
  background-color: #fef0f0;
  border-left: 4px solid #f56c6c;
}

.log-time {
  color: #909399;
  font-size: 12px;
}

.log-type {
  font-weight: bold;
}

.log-url {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-details {
  padding: 12px 16px;
  background-color: #fff;
}

.log-details pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: #303133;
  white-space: pre-wrap;
  word-break: break-all;
}

.no-logs {
  text-align: center;
  color: #909399;
  padding: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>