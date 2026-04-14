<template>
  <div class="file-record-report">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>鏂囦欢璁板綍鎶ヨ〃</span>
        </div>
      </template>
      <div class="report-content">
        <el-form :model="searchForm" label-width="100px" class="search-form">
          <el-form-item label="鏃ユ湡鑼冨洿">
            <el-date-picker
              v-model="searchForm.dateRange"
              type="daterange"
              range-separator="鑷?
              start-placeholder="寮€濮嬫棩鏈?
              end-placeholder="缁撴潫鏃ユ湡"
              style="width: 300px"
            />
          </el-form-item>
          <el-form-item label="鏂囦欢绫诲瀷">
            <el-select v-model="searchForm.fileType" placeholder="璇烽€夋嫨鏂囦欢绫诲瀷" style="width: 200px">
              <el-option label="鎵€鏈夌被鍨? value="" />
              <el-option label="鍥剧墖" value="image" />
              <el-option label="瑙嗛" value="video" />
              <el-option label="鏂囨。" value="document" />
              <el-option label="鍏朵粬" value="other" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="search">鏌ヨ</el-button>
            <el-button @click="reset">閲嶇疆</el-button>
            <el-button type="success" @click="exportReport">瀵煎嚭鎶ヨ〃</el-button>
          </el-form-item>
        </el-form>

        <el-table :data="fileRecordsData" style="width: 100%" border>
          <el-table-column prop="id" label="鏂囦欢ID" width="100" />
          <el-table-column prop="fileName" label="鏂囦欢鍚嶇О" />
          <el-table-column prop="fileType" label="鏂囦欢绫诲瀷" width="100" />
          <el-table-column prop="fileSize" label="鏂囦欢澶у皬" width="120" />
          <el-table-column prop="uploadTime" label="涓婁紶鏃堕棿" width="180" />
          <el-table-column prop="uploadUser" label="涓婁紶鐢ㄦ埛" width="120" />
          <el-table-column prop="vehicleId" label="鍏宠仈杞﹁締" width="100" />
          <el-table-column prop="description" label="鏂囦欢鎻忚堪" />
          <el-table-column label="鎿嶄綔" width="150" fixed="right">
            <template #default="scope">
              <el-button type="primary" size="small" @click="viewFile(scope.row)">鏌ョ湅</el-button>
              <el-button size="small" @click="downloadFile(scope.row)">涓嬭浇</el-button>
            </template>
          </el-table-column>
        </el-table>

        <div class="pagination">
          <el-pagination
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
            :page-sizes="[10, 20, 50, 100]"
            layout="total, sizes, prev, pager, next, jumper"
            :total="total"
            @size-change="handleSizeChange"
            @current-change="handleCurrentChange"
          />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// 鎼滅储琛ㄥ崟
const searchForm = {
  dateRange: [] as [Date, Date] | null,
  fileType: '',
};

// 琛ㄦ牸鏁版嵁
const fileRecordsData = ref([]);
const total = ref(0);
const currentPage = ref(1);
const pageSize = ref(10);

// 鍔犺浇鏂囦欢璁板綍鏁版嵁
const loadFileRecordsData = async () => {
  try {
    // 鏋勫缓鏌ヨ鍙傛暟
    const params = {
      page: currentPage.value,
      page_size: pageSize.value,
    };
    
    if (searchForm.dateRange) {
      params.start_date = searchForm.dateRange[0] instanceof Date ? searchForm.dateRange[0].toISOString().split('T')[0] : searchForm.dateRange[0];
      params.end_date = searchForm.dateRange[1] instanceof Date ? searchForm.dateRange[1].toISOString().split('T')[0] : searchForm.dateRange[1];
    }
    
    if (searchForm.fileType) {
      params.file_type = searchForm.fileType;
    }
    
    // 璋冪敤API鑾峰彇鏂囦欢璁板綍鏁版嵁
    const response = await api.get('/api/reports/file-records', { params }) as any;
    fileRecordsData.value = response?.items || [];
    total.value = response?.total || 0;
  } catch (error) {
    console.error('鍔犺浇鏂囦欢璁板綍鏁版嵁澶辫触:', error);
    ElMessage.error('鍔犺浇鏂囦欢璁板綍鏁版嵁澶辫触');
  }
};

// 鎼滅储
const search = () => {
  currentPage.value = 1;
  loadFileRecordsData();
};

// 閲嶇疆
const reset = () => {
  searchForm.dateRange = null;
  searchForm.fileType = '';
  currentPage.value = 1;
  loadFileRecordsData();
};

// 瀵煎嚭鎶ヨ〃
const exportReport = async () => {
  try {
    // 鏋勫缓鏌ヨ鍙傛暟
    const params = {};
    
    if (searchForm.dateRange) {
      params.start_date = searchForm.dateRange[0] instanceof Date ? searchForm.dateRange[0].toISOString().split('T')[0] : searchForm.dateRange[0];
      params.end_date = searchForm.dateRange[1] instanceof Date ? searchForm.dateRange[1].toISOString().split('T')[0] : searchForm.dateRange[1];
    }
    
    if (searchForm.fileType) {
      params.file_type = searchForm.fileType;
    }
    
    // 璋冪敤API瀵煎嚭鎶ヨ〃
    const response = await api.get('/api/reports/file-records/export', { 
      params,
      responseType: 'blob'
    }) as any;
    
    // 澶勭悊瀵煎嚭鏂囦欢
    const blob = new Blob([response], { type: 'application/vnd.ms-excel' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `鏂囦欢璁板綍鎶ヨ〃_${new Date().toISOString().split('T')[0]}.xlsx`;
    link.click();
    URL.revokeObjectURL(url);
    
    ElMessage.success('瀵煎嚭鎶ヨ〃鎴愬姛');
  } catch (error) {
    console.error('瀵煎嚭鎶ヨ〃澶辫触:', error);
    ElMessage.error('瀵煎嚭鎶ヨ〃澶辫触');
  }
};

// 鏌ョ湅鏂囦欢
const viewFile = (row) => {
  console.log('鏌ョ湅鏂囦欢:', row);
  // 杩欓噷鍙互瀹炵幇鏂囦欢棰勮鍔熻兘
  ElMessage.info('鏌ョ湅鏂囦欢鍔熻兘寮€鍙戜腑');
};

// 涓嬭浇鏂囦欢
const downloadFile = (row) => {
  console.log('涓嬭浇鏂囦欢:', row);
  // 杩欓噷鍙互瀹炵幇鏂囦欢涓嬭浇鍔熻兘
  ElMessage.info('涓嬭浇鏂囦欢鍔熻兘寮€鍙戜腑');
};

// 鍒嗛〉澶勭悊
const handleSizeChange = (size) => {
  pageSize.value = size;
  loadFileRecordsData();
};

const handleCurrentChange = (current) => {
  currentPage.value = current;
  loadFileRecordsData();
};

// 鍒濆鍖?
onMounted(() => {
  loadFileRecordsData();
});
</script>

<style scoped>
.file-record-report {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-form {
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f9f9f9;
  border-radius: 4px;
}

.report-content {
  min-height: 500px;
}

.pagination {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}
</style>


