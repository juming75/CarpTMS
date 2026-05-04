const { defineConfig } = require('vite');
const vue = require('@vitejs/plugin-vue');
const path = require('path');

// CarpTMS - Vite配置 (CommonJS版本)
module.exports = defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
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
        target: 'ws://localhost:8089',
        ws: true,
        changeOrigin: true,
      },
      '/map': {
        target: 'file:///D:/studying/Code%20backcp/china2014',
        changeOrigin: false,
        rewrite: (path) => path.replace(/^\/map/, '/map'),
      },
    },
    timeout: 60000,
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: false,
    minify: 'terser',
    cssCodeSplit: true,
    timeout: 120000,
    cacheDir: 'node_modules/.vite',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ['console.log', 'console.warn', 'console.error'],
      },
    },
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
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
    force: false,
    esbuildOptions: {
      target: 'es2015',
      define: {
        'process.env.NODE_ENV': '"production"',
      },
    },
  },
});