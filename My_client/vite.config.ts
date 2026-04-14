import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

// CarpTMS - Vite配置
export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 5173,
    host: true,
    proxy: {
      '/api': {
        target: 'http://localhost:8082',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:8082',
        ws: true,
        changeOrigin: true,
      },
      '/map': {
        target: 'file:///D:/studying/Code%20backcp/china2014',
        changeOrigin: false,
        rewrite: (path) => path.replace(/^\/map/, '/map'),
      },
    },
    // 生产部署建议：增加服务器超时设置
    timeout: 60000,
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: false,
    minify: 'terser',
    cssCodeSplit: true,
    // 生产部署建议：增加构建超时设置
    timeout: 120000,
    // 生产部署建议：启用持久化缓存
    cacheDir: 'node_modules/.vite',
    // 生产部署建议：配置Terser选项
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ['console.log', 'console.warn', 'console.error'],
      },
    },
    // 增加chunk大小警告限制
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      external: ['AMap'],
      output: {
        chunkFileNames: 'assets/js/[name]-[hash].js',
        entryFileNames: 'assets/js/[name]-[hash].js',
        assetFileNames: 'assets/[ext]/[name]-[hash].[ext]',
        // 生产部署建议：启用代码分割和共享块
        manualChunks: {
          vendor: ['vue', 'vue-router', 'pinia'],
          ui: ['element-plus', '@element-plus/icons-vue'],
          charts: ['echarts'],
          map: ['ol'],
          network: ['axios', 'dayjs'],
        },
      },
    },
  },
  optimizeDeps: {
    include: [
      'vue',
      'vue-router',
      'pinia',
      'element-plus',
      '@element-plus/icons-vue',
      'echarts',
      'ol',
      'axios',
      'dayjs',
    ],
    exclude: [],
    // 生产部署建议：启用依赖预构建
    force: false,
    // 生产部署建议：配置依赖扫描
    esbuildOptions: {
      target: 'es2015',
      define: {
        'process.env.NODE_ENV': '"production"',
      },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './vitest.setup.ts',
    alias: {
      '@': resolve(__dirname, 'src'),
    },
    coverage: {
      provider: 'v8',
      include: ['src/**/*.{ts,vue}'],
      exclude: ['src/main.ts', 'src/router/index.ts', 'src/App.vue'],
    },
  },
});