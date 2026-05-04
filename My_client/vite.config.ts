import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

// CarpTMS - Vite配置
export default defineConfig(({ mode }) => {
  // 根据模式加载环境变量
  const env = loadEnv(mode, process.cwd(), '');

  // 根据模式设置输出目录
  const outDir = mode === 'development' ? 'dist/dev'
    : mode === 'production-cargo' ? 'dist/cargo'
    : mode === 'production-sanitation' ? 'dist/sanitation'
    : mode === 'production-logistics' ? 'dist/logistics'
    : 'dist';

  return {
    base: './',
    plugins: [vue()],
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    define: {
      // 暴露环境变量给前端
      'import.meta.env.VITE_ENABLE_REMOTE_OPS': JSON.stringify(env.VITE_ENABLE_REMOTE_OPS || 'false'),
      'import.meta.env.VITE_APP_TITLE': JSON.stringify(env.VITE_APP_TITLE || 'CarpTMS'),
      'import.meta.env.VITE_TIANDITU_KEY': JSON.stringify(env.VITE_TIANDITU_KEY || ''),
    },
    server: {
      port: 5173,
      host: true,
      proxy: {
        '/api': {
          target: 'http://localhost:8082',
          changeOrigin: true,
          secure: false,
          cookieDomainRewrite: 'localhost',
          configure: (proxy) => {
            proxy.on('proxyRes', (proxyRes) => {
              // 确保 Set-Cookie 头被转发
              const setCookie = proxyRes.headers['set-cookie'];
              if (setCookie) {
                console.log('[Vite Proxy] Set-Cookie received:', setCookie);
              }
            });
          },
          headers: {
            'Origin': 'http://localhost:5173',
          },
        },
        '/ws': {
          target: 'ws://localhost:8082',
          ws: true,
          changeOrigin: true,
        },
        '/tianditu': {
          target: 'https://t0.tianditu.gov.cn',
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/tianditu/, ''),
        },
        '/map': {
          target: 'http://localhost:8082',
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/map/, '/api/map'),
        },
      },
      timeout: 60000,
    },
    build: {
      outDir,
      assetsDir: 'assets',
      sourcemap: false,
      minify: 'esbuild',
      cssCodeSplit: true,
      timeout: 120000,
      cacheDir: 'node_modules/.vite',
      chunkSizeWarningLimit: 1500,
      rollupOptions: {
        external: ['AMap'],
        output: {
          chunkFileNames: 'assets/js/[name]-[hash].js',
          entryFileNames: 'assets/js/[name]-[hash].js',
          assetFileNames: 'assets/[ext]/[name]-[hash].[ext]',
          manualChunks: {
            vendor: ['vue', 'vue-router', 'pinia'],
            ui: ['element-plus', '@element-plus/icons-vue'],
            charts: ['echarts'],
            map: ['ol'],
            network: ['axios', 'dayjs'],
            video: ['flv.js'],
            terminal: ['@xterm/xterm', '@xterm/addon-fit', '@xterm/addon-web-links'],
          },
        },
      },
      commonjsOptions: {
        include: [/node_modules/],
        transformMixedEsModules: true,
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
        'dompurify',
      ],
      exclude: ['AMap'],
      force: true,
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
      include: ['src/**/*.{test,spec}.{ts,tsx}'],
      exclude: ['node_modules', 'tests/e2e/**/*'],
      alias: {
        '@': resolve(__dirname, 'src'),
      },
      coverage: {
        provider: 'v8',
        include: ['src/**/*.{ts,vue}'],
        exclude: ['src/main.ts', 'src/router/index.ts', 'src/App.vue'],
      },
    },
  };
});
