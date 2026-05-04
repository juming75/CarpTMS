/**
 * 地图配置工具
 * 统一管理地图 API 密钥和其他配置
 */

/**
 * 获取天地图 API 密钥
 * 优先从环境变量获取，fallback 到用户本地存储，最后使用空字符串
 */
export function getTiandituKey(): string {
  // 1. 优先使用环境变量
  const envKey = import.meta.env.VITE_TIANDITU_KEY;
  if (envKey && envKey !== 'your_tianditu_api_key_here') {
    return envKey;
  }

  // 2. Fallback 到 localStorage（用户手动配置的密钥）
  const localKey = localStorage.getItem('tiandituKey');
  if (localKey) {
    return localKey;
  }

  // 3. 无可用密钥
  console.warn('天地图 API 密钥未配置，请设置 VITE_TIANDITU_KEY 环境变量');
  return '';
}

/**
 * 检查地图密钥是否已配置
 */
export function hasMapKey(): boolean {
  return !!getTiandituKey();
}

/**
 * 天地图瓦片代理路径（开发环境通过 Vite proxy 转发，生产环境通过 nginx 转发）
 * 使用代理而非直连，可避免 CORS 跨域限制（天地图服务器不支持 CORS）
 */
const TIAN_DITU_PROXY = '/tianditu';

/**
 * 获取地图瓦片服务 URL（通过代理路径，避免 CORS 问题）
 * 开发环境: Vite proxy (localhost:5173/tianditu/ → t0.tianditu.gov.cn/)
 * 生产环境: nginx proxy (yourdomain.com/tianditu/ → t0.tianditu.gov.cn/)
 */
export function getTiandituVecUrl(): string {
  const key = getTiandituKey();
  if (!key) {
    return '';
  }
  return `${TIAN_DITU_PROXY}/vec_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=vec&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${key}`;
}

export function getTiandituCvaUrl(): string {
  const key = getTiandituKey();
  if (!key) {
    return '';
  }
  return `${TIAN_DITU_PROXY}/cva_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=cva&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${key}`;
}

/**
 * 获取天地图瓦片 URL 的跨域配置
 * 使用代理后为同源请求，无需 crossOrigin，但保留以防直接调用
 */
export function getTiandituCrossOrigin(): 'anonymous' | '' {
  const isUsingProxy = location.origin.includes('localhost') || location.origin.includes('127.0.0.1');
  // 走代理时不需要 crossOrigin（同源），但保留 anonymous 确保兼容
  return isUsingProxy ? '' : 'anonymous';
}

/**
 * 获取地图配置
 */
export function getMapConfig() {
  return {
    tiandituKey: getTiandituKey(),
    hasKey: hasMapKey(),
    vecUrl: getTiandituVecUrl(),
    cvaUrl: getTiandituCvaUrl(),
  };
}
