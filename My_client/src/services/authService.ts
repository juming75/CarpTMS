/**
 * 统一认证服务
 * 
 * 认证策略：
 * 1. 优先使用 HttpOnly Cookie 方案（安全）
 * 2. localStorage 作为降级/同步检查方案
 * 3. Pinia Store 作为状态管理
 */

import { useAuthStore } from '@/stores/useAuthStore';

// Cookie 名称
const AUTH_CHECK_COOKIE = 'auth_check';
const REDIRECT_QUERY = 'redirect';

/**
 * 获取 Cookie 值
 */
function getCookie(name: string): string | null {
  if (typeof document === 'undefined') {
    return null;
  }
  const match = document.cookie.match(new RegExp(`(?:^|; )${name}=([^;]*)`));
  return match ? decodeURIComponent(match[1]) : null;
}

/**
 * 统一认证检查
 * 优先检查 HttpOnly Cookie，确保与后端一致
 */
export function isAuthenticated(): boolean {
  // 1. 首先检查 HttpOnly Cookie（主要方式）
  if (getCookie(AUTH_CHECK_COOKIE) === '1') {
    return true;
  }

  // 2. 降级检查：检查 localStorage token（仅用于快速同步检查）
  // 注意：这只是辅助检查，实际认证由 Cookie 决定
  const hasLocalToken = !!(localStorage.getItem('access_token') || sessionStorage.getItem('access_token'));
  
  // 如果有 localStorage token 但没有 Cookie，可能是 Cookie 未设置
  // 这种情况下允许请求继续，让后端验证
  if (hasLocalToken) {
    // 添加调试日志（生产环境可移除）
    if (import.meta.env.DEV) {
      console.warn('认证检查：localStorage 有 token 但 Cookie 未设置');
    }
    return true;
  }

  // 3. 最后尝试 Pinia store
  try {
    const authStore = useAuthStore();
    if (authStore.is_authenticated) {
      return true;
    }
  } catch {
    // Pinia store 未初始化，忽略
  }

  return false;
}

/**
 * 获取认证 Token（优先从 Cookie 获取）
 */
export function getAuthToken(): string | null {
  // 1. 优先从 Cookie 获取
  const cookieToken = getCookie('access_token');
  if (cookieToken) {
    return cookieToken;
  }

  // 2. 降级从 localStorage 获取
  return localStorage.getItem('access_token');
}

/**
 * 设置认证 Token
 */
export function setAuthToken(token: string, refreshToken?: string): void {
  localStorage.setItem('access_token', token);
  if (refreshToken) {
    localStorage.setItem('refresh_token', refreshToken);
  }
  if (typeof document !== 'undefined') {
    document.cookie = `${AUTH_CHECK_COOKIE}=1; path=/; max-age=86400; SameSite=Lax`;
  }
}

/**
 * 清除认证状态
 */
export function clearAuthentication(): void {
  // 清除 Pinia store
  try {
    const authStore = useAuthStore();
    authStore.clear_auth();
  } catch {
    // ignore if store is not available yet
  }

  // 清除 localStorage token
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
  sessionStorage.removeItem('access_token');
  sessionStorage.removeItem('refresh_token');

  // 清除 Cookie
  if (typeof document !== 'undefined') {
    document.cookie = `${AUTH_CHECK_COOKIE}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax`;
    document.cookie = `access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax`;
  }
}

/**
 * 重定向到登录页
 * 使用 hash 模式兼容的路径格式
 */
export function redirectToLogin(redirectPath = window.location.hash.slice(1) || '/home'): void {
  const encoded = encodeURIComponent(redirectPath);
  // hash 模式下使用 hash 路径
  window.location.hash = `#/login?${REDIRECT_QUERY}=${encoded}`;
}

/**
 * 强制登出
 */
export function forceLogout(redirectPath = window.location.pathname): void {
  clearAuthentication();
  redirectToLogin(redirectPath);
}

/**
 * 验证认证状态（异步）
 */
export async function validateAuth(): Promise<boolean> {
  // 如果 Cookie 检查已通过，认为已认证
  if (getCookie(AUTH_CHECK_COOKIE) === '1') {
    return true;
  }

  // 否则清除状态并返回 false
  if (isAuthenticated()) {
    // 有 localStorage token 但无 Cookie，可能是状态不同步
    // 清除本地状态，让用户重新登录
    clearAuthentication();
  }

  return false;
}
