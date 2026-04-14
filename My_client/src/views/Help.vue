<template>
  <div class="help-page">
    <div class="help-header">
      <h2>帮助中心</h2>
    </div>

    <div class="help-content">
      <el-row :gutter="20">
        <!-- 帮助导航 -->
        <el-col :span="8">
          <el-card shadow="hover" class="help-nav">
            <el-menu default-active="1" class="help-menu" @select="handleMenuSelect">
              <el-menu-item index="1">
                <el-icon><Document /></el-icon>
                <span>系统介绍</span>
              </el-menu-item>
              <el-menu-item index="2">
                <el-icon><Operation /></el-icon>
                <span>操作指南</span>
              </el-menu-item>
              <el-menu-item index="3">
                <el-icon><QuestionFilled /></el-icon>
                <span>常见问题</span>
              </el-menu-item>
              <el-menu-item index="4">
                <el-icon><Phone /></el-icon>
                <span>联系我们</span>
              </el-menu-item>
            </el-menu>
          </el-card>
        </el-col>

        <!-- 帮助内容 -->
        <el-col :span="16">
          <el-card shadow="hover" class="help-main">
            <!-- 系统介绍 -->
            <div v-if="activeTab === '1'" class="help-section">
              <h3>系统介绍</h3>
              <div class="content-box">
                <p>CarpTMS是一款高效的运输管理系统，为您提供全方位的物流管理解决方案。</p>
                <el-divider />
                <h4>主要功能</h4>
                <ul>
                  <li>订单管理：创建、查询、更新、删除订单，支持订单状态跟踪</li>
                  <li>物流跟踪：实时车辆位置监控、路线规划、历史轨迹查询</li>
                  <li>司机管理：司机信息管理、资质审核、排班管理</li>
                  <li>财务管理：费用核算、发票管理、收支统计</li>
                  <li>车辆管理：车辆信息管理、维护记录、状态监控</li>
                  <li>实时监控：实时数据展示、异常警报、统计分析</li>
                </ul>
              </div>
            </div>

            <!-- 操作指南 -->
            <div v-if="activeTab === '2'" class="help-section">
              <h3>操作指南</h3>
              <div class="content-box">
                <h4>1. 订单创建</h4>
                <p>进入订单管理页面，点击"新建订单"按钮，填写订单信息并保存。</p>

                <h4>2. 物流跟踪</h4>
                <p>在物流跟踪页面，您可以查看车辆实时位置、规划路线以及查询历史轨迹。</p>

                <h4>3. 司机管理</h4>
                <p>进入司机管理页面，您可以添加、编辑司机信息，审核司机资质，以及进行排班管理。</p>

                <h4>4. 财务管理</h4>
                <p>在财务管理页面，您可以进行费用核算、发票管理以及查看收支统计。</p>
              </div>
            </div>

            <!-- 常见问题 -->
            <div v-if="activeTab === '3'" class="help-section">
              <h3>常见问题</h3>
              <div class="content-box">
                <el-collapse v-model="activeFaq">
                  <el-collapse-item title="如何创建新订单？" name="1">
                    <div>进入订单管理页面，点击"新建订单"按钮，填写订单信息并保存即可。</div>
                  </el-collapse-item>
                  <el-collapse-item title="如何查看车辆实时位置？" name="2">
                    <div>进入地图窗口页面，您可以在左侧车辆列表中选择车辆查看实时位置。</div>
                  </el-collapse-item>
                  <el-collapse-item title="如何添加新司机？" name="3">
                    <div>进入司机管理页面，点击"添加司机"按钮，填写司机信息并保存即可。</div>
                  </el-collapse-item>
                  <el-collapse-item title="如何生成财务报表？" name="4">
                    <div>进入报表中心页面，选择报表类型和时间范围，点击"生成报表"按钮即可。</div>
                  </el-collapse-item>
                  <el-collapse-item title="忘记密码怎么办？" name="5">
                    <div>在登录页面点击"忘记密码"链接，按照提示进行密码重置操作。</div>
                  </el-collapse-item>
                </el-collapse>
              </div>
            </div>

            <!-- 联系我们 -->
            <div v-if="activeTab === '4'" class="help-section">
              <h3>联系我们</h3>
              <div class="content-box">
                <el-descriptions :column="1" border>
                  <el-descriptions-item label="公司名称">四川星驰通联科技有限公司</el-descriptions-item>
                  <el-descriptions-item label="联系微信">CQ751229</el-descriptions-item>
                  <el-descriptions-item label="电子邮箱">294400617@qq.com</el-descriptions-item>
                  <el-descriptions-item label="公司地址">四川凉山西昌市航天大道一段</el-descriptions-item>
                  <el-descriptions-item label="工作时间">周一至周五 9:00-18:00</el-descriptions-item>
                </el-descriptions>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Document, Operation, QuestionFilled, Phone } from '@element-plus/icons-vue';

// 当前激活的标签页
const activeTab = ref('1');
// 当前激活的FAQ
const activeFaq = ref(['1']);

// 菜单选择处理
const handleMenuSelect = (key: string) => {
  activeTab.value = key;
};
</script>

<style scoped>
.help-page {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.help-header {
  margin-bottom: 20px;
}

.help-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

.help-nav {
  height: fit-content;
}

.help-menu {
  border: none;
}

.help-section h3 {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 18px;
  color: #303133;
}

.help-section h4 {
  margin-top: 20px;
  margin-bottom: 10px;
  font-size: 16px;
  color: #606266;
}

.content-box {
  line-height: 1.8;
  color: #303133;
}

.content-box ul {
  padding-left: 20px;
}

.content-box li {
  margin-bottom: 8px;
}
</style>


