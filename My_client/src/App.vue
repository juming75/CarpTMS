<template>
  <el-config-provider :locale="zhCn">
    <div id="app">
      <router-view />
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import zhCn from 'element-plus/dist/locale/zh-cn.mjs';

onMounted(() => {
  console.log('CarpTMS Client started');
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  font-family: 'Microsoft YaHei', 'Segoe UI', sans-serif;
}

#app {
  background: #f0f2f5;
}
</style>


