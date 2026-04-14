<template>
  <div class="history-data">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>历史数据</span>
          <el-button type="primary" size="small">导出数据</el-button>
        </div>
      </template>
      <div class="history-content">
        <el-empty description="历史数据查询功能开发中..." />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// 历史数据页面
</script>

<style scoped>
.history-content {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>


