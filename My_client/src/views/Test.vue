<template>
  <div class="test-page">
    <h1>测试页面</h1>
    <p>这是一个简单的测试页面，用于测试路由是否正常工作。</p>
    <el-button type="primary" @click="goToHome">返回首页</el-button>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { ElButton } from 'element-plus';

const router = useRouter();

const goToHome = () => {
  router.push('/map-window');
};
</script>

<style scoped>
.test-page {
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.08);
}

h1 {
  margin-bottom: 20px;
  color: #303133;
}

p {
  margin-bottom: 20px;
  color: #606266;
}
</style>


