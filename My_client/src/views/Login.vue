<template>
  <div class="login-container">
    <!-- 背景装饰 -->
    <div class="bg-decoration">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>

    <div class="login-card">
      <!-- Logo 和标题 -->
      <div class="login-header">
        <div class="logo-wrapper">
          <div class="logo-inner">
            <el-icon class="logo-icon"><Van /></el-icon>
            <div class="logo-glow"></div>
          </div>
        </div>
        <h1 class="title">
          <span class="title-gradient">CarpTMS</span>
        </h1>
        <p class="subtitle">Welcome to TMS world!</p>
      </div>

      <!-- 服务器配置 -->
      <div class="server-section">
        <div class="server-display">
          <div class="server-info">
            <el-icon class="server-icon"><Connection /></el-icon>
            <span class="server-text">{{ serverConfig.ip }}:{{ serverConfig.port }}</span>
          </div>
          <el-button link class="config-btn" @click="showConfig = true">
            <el-icon><Setting /></el-icon>
            <span>配置</span>
          </el-button>
        </div>
      </div>

      <!-- 服务器配置弹窗 -->
      <el-dialog v-model="showConfig" title="服务器配置" width="400px" :close-on-click-modal="false">
        <el-form label-width="80px" label-position="left">
          <el-form-item label="服务器IP">
            <el-input v-model="configForm.ip" placeholder="请输入服务器IP地址" />
          </el-form-item>
          <el-form-item label="端口">
            <el-input v-model="configForm.port" type="number" placeholder="请输入端口号" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showConfig = false">取消</el-button>
          <el-button type="primary" @click="saveServerConfig">保存</el-button>
        </template>
      </el-dialog>

      <!-- 登录表单 -->
      <el-form
        ref="loginFormRef"
        :model="loginForm"
        :rules="loginRules"
        class="login-form"
        @submit.prevent="handleLogin"
      >
        <el-form-item prop="username">
          <el-input
            v-model="loginForm.username"
            placeholder="请输入用户名"
            size="large"
            :prefix-icon="User"
            clearable
            @keyup.enter="handleLogin"
          />
          <template #help>
            <span class="form-help">默认用户名: admin</span>
          </template>
        </el-form-item>

        <el-form-item prop="password">
          <el-input
            v-model="loginForm.password"
            type="password"
            placeholder="请输入密码"
            size="large"
            :prefix-icon="Lock"
            show-password
            @keyup.enter="handleLogin"
          />
          <template #help>
            <span class="form-help">默认密码: admin123</span>
          </template>
        </el-form-item>

        <!-- 选项 -->
        <div class="options-row">
          <el-checkbox v-model="loginForm.rememberPassword">记住密码</el-checkbox>
          <el-checkbox v-model="loginForm.autoLogin">自动登录</el-checkbox>
        </div>

        <!-- 登录按钮 -->
        <el-button type="primary" size="large" :loading="loading" class="login-button" @click="handleLogin">
          {{ loading ? '登录中...' : '登录' }}
        </el-button>
      </el-form>

      <!-- 底部信息 -->
      <div class="login-footer">
        <p>版本 {{ appVersion }} | Electron {{ electronVersion }}</p>
        <p>欢迎使用 CarpTMS 系统</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
// 全局声明
declare function btoa(str: string): string;
declare function atob(str: string): string;

import { ref, reactive, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import type { FormInstance } from 'element-plus';
import { Van, Setting, Connection, User, Lock } from '@element-plus/icons-vue';
import { getAppVersion, getElectronVersion } from '@/services/localDB';
import api from '@/api';
import type { FormRule, LoginForm, ServerConfig, ConfigForm, LoginRules } from '@/types/form';

// 从 cookie 中获取 token
const getTokenFromCookie = (name: string): string | null => {
  const cookieValue = document.cookie
    .split('; ')
    .find(row => row.startsWith(`${name}=`))
    ?.split('=')[1];
  return cookieValue ? decodeURIComponent(cookieValue) : null;
};

// 检查是否已认证
const isAuthenticated = (): boolean => {
  const token = getTokenFromCookie('access_token') || localStorage.getItem('access_token') || sessionStorage.getItem('access_token');
  return !!token;
};

const router = useRouter();

// 表单引用
const loginFormRef = ref<FormInstance>();
const loading = ref(false);
const showConfig = ref(false);
const appVersion = ref('1.0.0');
const electronVersion = ref('');

// 登录表单
const loginForm = reactive<LoginForm>({
  username: '',
  password: '',
  rememberPassword: false,
  autoLogin: false,
});

// 服务器配置
const serverConfig = reactive<ServerConfig>({
  ip: '127.0.0.1',
  port: '8082',
});

// 配置表单（用于编辑）
const configForm = reactive<ConfigForm>({
  ip: '',
  port: '',
});

// 加载保存的配置和用户信息
const loadSavedConfig = () => {
  // 加载服务器配置
  const savedIp = sessionStorage.getItem('serverIp');
  const savedPort = sessionStorage.getItem('serverPort');
  if (savedIp) serverConfig.ip = savedIp;
  if (savedPort) serverConfig.port = savedPort;

  configForm.ip = serverConfig.ip;
  configForm.port = serverConfig.port;

  // 加载保存的密码
  const savedUsername = sessionStorage.getItem('savedUsername');
  const savedPassword = sessionStorage.getItem('savedPassword');
  const rememberPassword = sessionStorage.getItem('rememberPassword') === 'true';

  if (savedUsername && rememberPassword && savedPassword) {
    loginForm.username = savedUsername;
    loginForm.password = savedPassword;
    loginForm.rememberPassword = true;
  }

  // 检查自动登录
  const autoLogin = sessionStorage.getItem('autoLogin') === 'true';
  loginForm.autoLogin = autoLogin;

  // 如果自动登录且有保存的密码，则自动登录
  if (autoLogin && savedUsername && savedPassword && rememberPassword) {
    setTimeout(() => handleLogin(), 500);
  }
};

// 保存服务器配置
const saveServerConfig = () => {
  // 验证 IP 地址格式
  const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
  if (!ipRegex.test(configForm.ip)) {
    ElMessage({
      message: '请输入正确的 IP 地址格式，例如：127.0.0.1',
      type: 'error',
      duration: 3000,
      showClose: true,
    });
    return;
  }

  // 验证端口
  const port = parseInt(configForm.port);
  if (isNaN(port) || port < 1 || port > 65535) {
    ElMessage({
      message: '请输入正确的端口号 (1-65535)',
      type: 'error',
      duration: 3000,
      showClose: true,
    });
    return;
  }

  try {
    // 保存配置
    serverConfig.ip = configForm.ip;
    serverConfig.port = configForm.port;
    sessionStorage.setItem('serverIp', serverConfig.ip);
    sessionStorage.setItem('serverPort', serverConfig.port);

    // 更新 API 配置
    ElMessage({
      message: '服务器配置已保存，系统将使用新的服务器地址',
      type: 'success',
      duration: 2000,
      showClose: false,
    });
    showConfig.value = false;
  } catch (_error) {
    ElMessage({
      message: '保存服务器配置失败，请重试',
      type: 'error',
      duration: 3000,
      showClose: true,
    });
  }
};

// 保存用户信息
const saveUserInfo = () => {
  if (loginForm.rememberPassword) {
    sessionStorage.setItem('savedUsername', loginForm.username);
    sessionStorage.setItem('savedPassword', loginForm.password);
    sessionStorage.setItem('rememberPassword', 'true');
  } else {
    sessionStorage.removeItem('savedUsername');
    sessionStorage.removeItem('savedPassword');
    sessionStorage.setItem('rememberPassword', 'false');
  }

  sessionStorage.setItem('autoLogin', loginForm.autoLogin.toString());
};

// 密码验证规则（简化版）
const validatePassword = (rule: FormRule, value: string, callback: (error?: Error) => void) => {
  if (!value) {
    callback(new Error('请输入密码'));
  } else {
    // 接受简单密码，只验证非空
    callback();
  }
};

// 登录验证规则
const loginRules = reactive<LoginRules>({
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { validator: validatePassword, trigger: 'blur' },
  ],
});

// 处理登录
const handleLogin = async () => {
  if (!loginFormRef.value || loading.value) return;

  await loginFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return;

    loading.value = true;

    try {
      // 调用登录API - 使用独立输入避免自动输入问题
      console.log('开始登录，用户名:', loginForm.username);
      const response = await api.post('/api/auth/login', { username: loginForm.username, password: loginForm.password });
      console.log('登录API响应:', response);

      // 标准化处理响应：支持标准格式 { code, message, data: {...} } 和直接返回 { access_token, ... }
      let loginData = response || {};
      
      // 检查响应结构
      if (response && response.code !== undefined && response.data && typeof response.data === 'object') {
        // 标准格式：{ code, message, data: {...} }
        loginData = response.data;
      }
      
      const token = loginData.access_token || loginData.token;
      const refreshToken = loginData.refresh_token;
      const userInfo = loginData.user || loginData.userInfo;

      console.log('登录数据处理结果:', { token: token ? '存在' : '不存在', refreshToken: refreshToken ? '存在' : '不存在', userInfo: userInfo ? '存在' : '不存在' });

      if (token) {
        // 保存 token 到 localStorage，确保即使设置 HTTP-Only cookie 失败也能正常登录
        console.log('保存token到localStorage');
        localStorage.setItem('access_token', token);
        if (refreshToken) {
          localStorage.setItem('refresh_token', refreshToken);
        }
        
        // 尝试保存 token 到 httpOnly cookie（可选）
        try {
          document.cookie = `access_token=${token}; path=/; HttpOnly; SameSite=Strict; Secure`;
          if (refreshToken) {
            document.cookie = `refresh_token=${refreshToken}; path=/; HttpOnly; SameSite=Strict; Secure`;
          }
          document.cookie = `auth_check=1; path=/; SameSite=Strict; Secure`;
          console.log('Cookie设置成功');
        } catch (error) {
          console.error('Cookie设置失败:', error);
        }
        
        // 保存用户信息到 localStorage
        if (userInfo) {
          console.log('保存用户信息到localStorage');
          localStorage.setItem('userInfo', JSON.stringify(userInfo));
          // 保存用户ID到本地存储，方便后续API调用
          if (userInfo.user_id) {
            localStorage.setItem('userId', userInfo.user_id.toString());
          }
        }
        saveUserInfo();

        // 登录成功的动画反馈
        const loginCard = document.querySelector('.login-card') as HTMLElement;
        if (loginCard) {
          loginCard.classList.add('login-success');
        }

        ElMessage({
          message: '登录成功，正在进入系统...',
          type: 'success',
          duration: 2000,
          showClose: false,
        });

        // 跳转使用replace，防止浏览器返回按钮回到登录页
        setTimeout(() => {
          console.log('登录成功，跳转到主页面');
          router.replace('/map-window');
        }, 1000);
      } else {
        console.error('登录成功但数据结构异常:', loginData);
        ElMessage.error('登录成功但数据结构异常');
      }
    } catch (error: unknown) {
      // 登录失败的详细错误提示
      let errorMessage = '登录失败，请检查用户名和密码';

      if (error instanceof Error) {
        console.error('登录错误:', error);
        if (error.message.includes('401')) {
          errorMessage = '用户名或密码错误';
        } else if (error.message.includes('403')) {
          errorMessage = '没有权限访问该资源';
        } else if (error.message.includes('404')) {
          errorMessage = '请求的资源不存在';
        } else if (error.message.includes('500')) {
          errorMessage = '服务器内部错误，请稍后重试';
        } else if (error.message.includes('Network Error')) {
          errorMessage = '网络连接失败，请检查网络设置';
        } else {
          errorMessage = error.message;
        }
      }

      ElMessage({
        message: errorMessage,
        type: 'error',
        duration: 3000,
        showClose: true,
      });

      // 登录失败的动画反馈
      const loginCard = document.querySelector('.login-card') as HTMLElement;
      if (loginCard) {
        loginCard.classList.add('login-error');
        setTimeout(() => {
          loginCard.classList.remove('login-error');
        }, 1000);
      }
    } finally {
      loading.value = false;
    }
  });
};

onMounted(async () => {
  appVersion.value = await getAppVersion();
  electronVersion.value = await getElectronVersion();
  loadSavedConfig();
});
</script>

<style scoped>
.login-container {
  width: 100%;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  position: relative;
  overflow: hidden;
}

/* 背景装饰 */
.bg-decoration {
  position: absolute;
  width: 100%;
  height: 100%;
  overflow: hidden;
  pointer-events: none;
}

/* 增强的背景动画 */
.bg-decoration::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
  animation: backgroundShift 20s ease-in-out infinite;
}

@keyframes backgroundShift {
  0%,
  100% {
    transform: scale(1) rotate(0deg);
  }
  50% {
    transform: scale(1.1) rotate(180deg);
  }
}

.circle {
  position: absolute;
  border-radius: 50%;
  animation: float 20s infinite ease-in-out;
  filter: blur(30px);
  mix-blend-mode: overlay;
}

.circle-1 {
  width: 400px;
  height: 400px;
  top: -150px;
  left: -150px;
  background: radial-gradient(circle, rgba(102, 126, 234, 0.3) 0%, transparent 70%);
  animation-delay: 0s;
  animation: float1 15s ease-in-out infinite;
}

@keyframes float1 {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(50px, -50px) scale(1.1);
  }
}

.circle-2 {
  width: 300px;
  height: 300px;
  bottom: -100px;
  right: -100px;
  background: radial-gradient(circle, rgba(118, 75, 162, 0.3) 0%, transparent 70%);
  animation-delay: -5s;
  animation: float2 12s ease-in-out infinite;
}

@keyframes float2 {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(-40px, 40px) scale(1.2);
  }
}

.circle-3 {
  width: 250px;
  height: 250px;
  top: 50%;
  right: 20%;
  background: radial-gradient(circle, rgba(255, 107, 107, 0.2) 0%, transparent 70%);
  animation-delay: -10s;
  animation: float3 18s ease-in-out infinite;
}

@keyframes float3 {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(-30px, -30px) scale(1.15);
  }
}

.circle-4 {
  position: absolute;
  width: 200px;
  height: 200px;
  top: 20%;
  left: 20%;
  background: radial-gradient(circle, rgba(52, 211, 153, 0.2) 0%, transparent 70%);
  border-radius: 50%;
  filter: blur(30px);
  mix-blend-mode: overlay;
  animation: float4 14s ease-in-out infinite;
}

@keyframes float4 {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(40px, 40px) scale(1.1);
  }
}

/* 登录卡片 */
.login-card {
  width: 100%;
  max-width: 420px;
  padding: 48px 40px 40px;
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow: 0 25px 80px rgba(0, 0, 0, 0.35);
  position: relative;
  z-index: 1;
  border: 1px solid rgba(255, 255, 255, 0.3);
  animation: cardSlideUp 0.8s ease-out;
  overflow: hidden;
}

@keyframes cardSlideUp {
  from {
    opacity: 0;
    transform: translateY(50px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 卡片顶部装饰 */
.login-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
  animation: gradientShift 3s ease-in-out infinite;
}

@keyframes gradientShift {
  0%,
  100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}

/* 头部 */
.login-header {
  text-align: center;
  margin-bottom: 32px;
  animation: fadeInDown 0.6s ease-out;
}

@keyframes fadeInDown {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.logo-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100px;
  height: 100px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 50%;
  margin-bottom: 20px;
  box-shadow: 0 12px 30px rgba(102, 126, 234, 0.5);
  position: relative;
  animation: logoPulse 2s ease-in-out infinite;
}

@keyframes logoPulse {
  0%,
  100% {
    box-shadow: 0 12px 30px rgba(102, 126, 234, 0.5);
  }
  50% {
    box-shadow: 0 15px 40px rgba(102, 126, 234, 0.7);
  }
}

.logo-inner {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  backdrop-filter: blur(10px);
  border: 2px solid rgba(255, 255, 255, 0.2);
  z-index: 2;
}

.logo-glow {
  position: absolute;
  top: -10px;
  left: -10px;
  right: -10px;
  bottom: -10px;
  background: conic-gradient(from 0deg, transparent, #667eea, transparent);
  border-radius: 50%;
  z-index: 1;
  animation: rotateGlow 3s linear infinite;
}

@keyframes rotateGlow {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.logo-icon {
  font-size: 48px;
  color: #fff;
  text-shadow: 0 0 20px rgba(255, 255, 255, 0.8);
  animation: iconBounce 1.5s ease-in-out infinite;
}

@keyframes iconBounce {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

.title {
  font-size: 40px;
  font-weight: 800;
  margin: 0 0 12px;
  letter-spacing: -0.5px;
  position: relative;
}

.title-gradient {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  display: inline-block;
  animation: titleShine 3s ease-in-out infinite;
}

@keyframes titleShine {
  0%,
  100% {
    filter: brightness(1);
  }
  50% {
    filter: brightness(1.1);
  }
}

.subtitle {
  font-size: 16px;
  color: #666;
  margin: 0;
  font-weight: 500;
  background: linear-gradient(135deg, #666 0%, #999 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: subtitleFade 3s ease-in-out infinite;
}

@keyframes subtitleFade {
  0%,
  100% {
    opacity: 0.8;
  }
  50% {
    opacity: 1;
  }
}

/* 服务器配置 */
.server-section {
  margin-bottom: 24px;
}

.server-display {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8eb 100%);
  border-radius: 10px;
  border: 1px solid #e4e8eb;
}

.server-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.server-icon {
  font-size: 16px;
  color: #667eea;
}

.server-text {
  font-size: 13px;
  font-weight: 600;
  color: #333;
  font-family: 'Consolas', 'Monaco', monospace;
}

.config-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  color: #667eea;
  transition: all 0.3s;
}

.config-btn:hover {
  background: rgba(102, 126, 234, 0.1);
}

/* 登录表单 */
.login-form {
  margin-bottom: 24px;
  animation: fadeInUp 0.8s ease-out 0.3s both;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.login-form :deep(.el-input__wrapper) {
  border-radius: 12px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.08);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(228, 232, 235, 0.8);
}

.login-form :deep(.el-input__wrapper:hover),
.login-form :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.25);
  border-color: rgba(102, 126, 234, 0.5);
  transform: translateY(-2px);
}

.login-form :deep(.el-input__inner) {
  font-size: 14px;
  letter-spacing: 0.5px;
  background: transparent;
  color: #333;
}

.login-form :deep(.el-input__prefix-inner) {
  color: #667eea;
}

.login-form :deep(.el-form-item) {
  margin-bottom: 20px;
}

/* 表单帮助文字 */
.form-help {
  font-size: 12px;
  color: #667eea;
  font-weight: 500;
  margin-top: 4px;
  display: block;
}

.login-form :deep(.el-form-item__help) {
  color: #667eea;
  font-size: 12px;
  margin-top: 4px;
}

/* 选项行 */
.options-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.options-row :deep(.el-checkbox) {
  margin-right: 0;
}

.options-row :deep(.el-checkbox__label) {
  font-size: 13px;
  color: #666;
}

/* 登录按钮 */
.login-button {
  width: 100%;
  height: 52px;
  font-size: 17px;
  font-weight: 700;
  border-radius: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 6px 25px rgba(102, 126, 234, 0.5);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  letter-spacing: 1px;
  position: relative;
  overflow: hidden;
  animation: buttonFadeIn 0.8s ease-out 0.5s both;
}

@keyframes buttonFadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.login-button::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
  transition: left 0.5s;
}

.login-button:hover::before {
  left: 100%;
}

.login-button:hover {
  transform: translateY(-3px);
  box-shadow: 0 10px 35px rgba(102, 126, 234, 0.6);
}

.login-button:active {
  transform: translateY(-1px);
  box-shadow: 0 6px 25px rgba(102, 126, 234, 0.5);
}

.login-button:disabled {
  transform: none;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
  opacity: 0.8;
}

/* 底部信息 */
.login-footer {
  text-align: center;
  padding-top: 24px;
  border-top: 1px solid rgba(228, 232, 235, 0.6);
  animation: footerFadeIn 0.8s ease-out 0.7s both;
}

@keyframes footerFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.login-footer p {
  margin: 6px 0;
  font-size: 13px;
  color: #888;
  transition: all 0.3s;
}

.login-footer p:first-child {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 600;
  font-size: 14px;
}

.login-footer p:hover {
  color: #667eea;
}

/* 登录成功动画 */
.login-card.login-success {
  animation: loginSuccess 1s ease-out;
  box-shadow: 0 25px 80px rgba(52, 211, 153, 0.4);
  border-color: rgba(52, 211, 153, 0.5);
}

@keyframes loginSuccess {
  0% {
    transform: scale(1);
    box-shadow: 0 25px 80px rgba(0, 0, 0, 0.35);
  }
  50% {
    transform: scale(1.02);
    box-shadow: 0 30px 90px rgba(52, 211, 153, 0.6);
  }
  100% {
    transform: scale(1);
    box-shadow: 0 25px 80px rgba(52, 211, 153, 0.4);
  }
}

/* 登录失败动画 */
.login-card.login-error {
  animation: loginError 0.5s ease-in-out;
  box-shadow: 0 25px 80px rgba(239, 68, 68, 0.4);
  border-color: rgba(239, 68, 68, 0.5);
}

@keyframes loginError {
  0%,
  100% {
    transform: translateX(0);
  }
  25% {
    transform: translateX(-5px);
  }
  75% {
    transform: translateX(5px);
  }
}

/* 响应式 */
@media (max-width: 480px) {
  .login-card {
    margin: 20px;
    padding: 32px 24px 24px;
  }

  .title {
    font-size: 28px;
  }

  .circle {
    display: none;
  }
}
</style>
