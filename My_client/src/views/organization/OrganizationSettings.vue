<template>
  <div class="org-settings-container">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span>组织相关设置</span>
        </div>
      </template>

      <el-form :model="form" label-position="right" label-width="200px" class="settings-form">
        <!-- 组织相关设置 -->
        <div class="settings-section">
          <!-- 个性化登录地址 -->
          <el-form-item label="个性化登录地址">
            <el-input v-model="form.loginUrl" placeholder="https://yourdomain.star366.cn" />
            <div class="form-tip">通过个性化登录地址，可显示个性化背景图，免于输入团队代码。</div>
          </el-form-item>

          <!-- 个性化登录页面副标题 -->
          <el-form-item label="个性化登录页面副标题">
            <el-input
              v-model="form.loginSubtitle"
              type="textarea"
              :rows="2"
              placeholder="请输入个性化登录页面副标题"
            />
          </el-form-item>

          <!-- 个性化登录页面底部公司名称 -->
          <el-form-item label="个性化登录页面底部&#10;公司名称">
            <el-input v-model="form.footerCompanyName" placeholder="请输入公司名称" />
          </el-form-item>

          <!-- 个性化登录页面背景图 -->
          <el-form-item label="个性化登录页面背景图">
            <div class="upload-area">
              <div v-if="!form.backgroundImage" class="upload-placeholder">
                <el-icon class="upload-icon"><Picture /></el-icon>
              </div>
              <img v-else :src="form.backgroundImage" class="preview-image" />
              <div class="upload-actions">
                <el-upload
                  action="#"
                  :auto-upload="false"
                  :on-change="handleImageChange"
                  :show-file-list="false"
                  accept="image/jpeg,image/gif,image/png"
                >
                  <el-button type="primary" plain>上传新图片</el-button>
                </el-upload>
                <div class="upload-tip">
                  图片格式 JPG, GIF 或 PNG. 最大 10M<br />
                  尺寸：1920*1080，尽量小于1M，深色背景为佳。
                  <el-button type="primary" link @click="useDefaultBackground">使用默认背景</el-button>
                </div>
              </div>
            </div>
          </el-form-item>

          <!-- 数字大屏标题 -->
          <el-form-item label="数字大屏标题">
            <el-input v-model="form.dashboardTitle" placeholder="请输入数字大屏标题" />
          </el-form-item>

          <!-- 行业分类 -->
          <el-form-item label="行业分类">
            <el-select v-model="form.industryType" placeholder="选择" class="full-width">
              <el-option label="智慧环卫" value="sanitation" />
              <el-option label="智慧物流" value="overload" />
              <el-option label="智慧交通" value="logistics" />
            </el-select>
          </el-form-item>
        </div>

        <!-- 货物分类相关设置 -->
        <div class="settings-section section-divider">
          <div class="section-title">货物分类相关设置</div>

          <!-- 首页看板标题1 -->
          <el-form-item label="首页看板标题1">
            <el-input v-model="form.dashboardTitle1" placeholder="请输入首页看板标题1" />
            <div class="form-tip">首页看板和数字大屏右侧作业量排行，第一个列表标题。</div>
          </el-form-item>

          <!-- 首页看板标题2 -->
          <el-form-item label="首页看板标题2">
            <el-input v-model="form.dashboardTitle2" placeholder="请输入首页看板标题2" />
            <div class="form-tip">首页看板和数字大屏右侧作业量排行，第二个列表标题。</div>
          </el-form-item>

          <!-- 货物分类标题 -->
          <el-form-item label="货物分类标题">
            <el-input v-model="form.cargoCategoryTitle" placeholder="请输入货物分类标题" />
            <div class="form-tip">货物分类的标题名称。</div>
          </el-form-item>

          <!-- 分类1 -->
          <el-form-item label="分类1">
            <el-input v-model="form.category1" placeholder="请输入分类1名称" />
            <div class="form-tip">输入【-】不启用本分类。</div>
          </el-form-item>

          <!-- 分类2 -->
          <el-form-item label="分类2">
            <el-input v-model="form.category2" placeholder="请输入分类2名称" />
            <div class="form-tip">输入【-】不启用本分类。</div>
          </el-form-item>

          <!-- 分类3 -->
          <el-form-item label="分类3">
            <el-input v-model="form.category3" placeholder="请输入分类3名称" />
            <div class="form-tip">输入【-】不启用本分类。</div>
          </el-form-item>

          <!-- 分类4 -->
          <el-form-item label="分类4">
            <el-input v-model="form.category4" placeholder="请输入分类4名称" />
            <div class="form-tip">输入【-】不启用本分类。</div>
          </el-form-item>

          <!-- 分类5 -->
          <el-form-item label="分类5">
            <el-input v-model="form.category5" placeholder="请输入分类5名称" />
            <div class="form-tip">输入【-】不启用本分类。</div>
          </el-form-item>
        </div>

        <!-- 提交按钮 -->
        <el-form-item>
          <el-button type="primary" :loading="saving" @click="handleSave">保存设置</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Picture } from '@element-plus/icons-vue';

interface OrgSettingsForm {
  loginUrl: string;
  loginSubtitle: string;
  footerCompanyName: string;
  backgroundImage: string;
  dashboardTitle: string;
  industryType: string;
  dashboardTitle1: string;
  dashboardTitle2: string;
  cargoCategoryTitle: string;
  category1: string;
  category2: string;
  category3: string;
  category4: string;
  category5: string;
}

const saving = ref(false);
const form = reactive<OrgSettingsForm>({
  loginUrl: '',
  loginSubtitle: '',
  footerCompanyName: '',
  backgroundImage: '',
  dashboardTitle: '',
  industryType: '',
  dashboardTitle1: '',
  dashboardTitle2: '',
  cargoCategoryTitle: '',
  category1: '',
  category2: '',
  category3: '',
  category4: '',
  category5: '',
});

// 原始数据，用于重置
let originalData: OrgSettingsForm | null = null;

onMounted(() => {
  loadSettings();
});

async function loadSettings() {
  try {
    // TODO: 调用后端 API 获取设置
    // const res = await api.get('/api/organization/settings');
    // Object.assign(form, res.data);

    // 清空所有默认值
    Object.assign(form, {
      loginUrl: '',
      loginSubtitle: '',
      footerCompanyName: '',
      backgroundImage: '',
      dashboardTitle: '',
      industryType: '',
      dashboardTitle1: '',
      dashboardTitle2: '',
      cargoCategoryTitle: '',
      category1: '',
      category2: '',
      category3: '',
      category4: '',
      category5: '',
    });

    originalData = { ...form };
  } catch (err) {
    ElMessage.error('加载设置失败');
  }
}

function handleImageChange(file: any) {
  const reader = new FileReader();
  reader.onload = (e) => {
    form.backgroundImage = e.target?.result as string;
  };
  reader.readAsDataURL(file.raw);
}

function useDefaultBackground() {
  form.backgroundImage = '';
  ElMessage.info('已使用默认背景');
}

async function handleSave() {
  saving.value = true;
  try {
    // TODO: 调用后端 API 保存设置
    // await api.post('/api/organization/settings', form);
    
    // 模拟保存
    await new Promise(resolve => setTimeout(resolve, 500));
    
    originalData = { ...form };
    ElMessage.success('保存成功');
  } catch (err) {
    ElMessage.error('保存失败');
  } finally {
    saving.value = false;
  }
}

function handleReset() {
  if (originalData) {
    Object.assign(form, originalData);
    ElMessage.info('已重置');
  }
}
</script>

<style scoped>
.org-settings-container {
  padding: 20px;
}

.settings-card {
  max-width: 900px;
}

.card-header {
  font-size: 16px;
  font-weight: 500;
}

.settings-form {
  padding: 20px 0;
}

.settings-section {
  margin-bottom: 40px;
}

.section-divider {
  border-top: 1px solid #ebeef5;
  padding-top: 30px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  margin-bottom: 20px;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
  line-height: 1.5;
}

.upload-area {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.upload-placeholder {
  width: 120px;
  height: 80px;
  border: 1px dashed #dcdfe6;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f7fa;
}

.upload-icon {
  font-size: 32px;
  color: #c0c4cc;
}

.preview-image {
  width: 120px;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
}

.upload-actions {
  flex: 1;
}

.upload-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 8px;
  line-height: 1.6;
}

.full-width {
  width: 100%;
}
</style>
