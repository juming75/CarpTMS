<template>
  <el-card style="margin-top: 20px">
    <template #header>
      <div class="card-header">
        <span>OpenAPI开放平台</span>
        <el-button type="primary" size="small" @click="addPlatform">添加平台</el-button>
      </div>
    </template>

    <div class="openapi-section">
      <!-- 平台设置 -->
      <div
        v-for="(platform, index) in openapiSettings.platforms"
        :key="platform.id"
        class="platform-item"
        style="margin-bottom: 20px; padding: 16px; border: 1px solid #e4e7ed; border-radius: 8px"
      >
        <div
          class="platform-header"
          style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px"
        >
          <h3>平台 {{ index + 1 }}</h3>
          <el-button type="danger" size="small" @click="removePlatform(platform.id)">删除平台</el-button>
        </div>

        <el-form label-width="120px" style="max-width: 600px">
          <el-form-item label="平台名称">
            <el-input v-model="platform.name" placeholder="请输入平台名称" />
          </el-form-item>

          <el-form-item label="外部平台IP">
            <el-input v-model="platform.externalIp" placeholder="请输入外部平台IP" />
          </el-form-item>

          <el-form-item label="外部平台端口">
            <el-input-number v-model="platform.externalPort" :min="1" :max="65535" />
          </el-form-item>
        </el-form>

        <!-- API接口选择 -->
        <div class="openapi-apis" style="margin-top: 20px">
          <h4 style="margin-bottom: 12px">API接口信息</h4>

          <el-table :data="openapiApis" style="width: 100%">
            <el-table-column width="50">
              <template #default="scope">
                <el-checkbox
                  v-model="platform.selectedApis"
                  :label="scope.row.id"
                  :disabled="scope.row.required"
                  >{{ scope.row.required ? '必选' : '' }}</el-checkbox
                >
              </template>
            </el-table-column>

            <el-table-column prop="name" label="接口名称" width="200" />
            <el-table-column prop="path" label="API路径" />
            <el-table-column prop="method" label="请求方法" width="100" />
            <el-table-column prop="required" label="是否必选" width="100">
              <template #default="scope">
                <el-tag :type="scope.row.required ? 'danger' : 'success'">
                  {{ scope.row.required ? '必选' : '可选' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="description" label="接口描述" />
          </el-table>
        </div>
      </div>

      <!-- 输出文件格式设置 -->
      <div
        class="output-format"
        style="margin-top: 20px; padding: 16px; border: 1px solid #e4e7ed; border-radius: 8px"
      >
        <h3 style="margin-bottom: 16px">输出文件格式</h3>

        <el-form label-width="120px" style="max-width: 600px">
          <el-form-item label="文件格式">
            <el-select v-model="openapiSettings.outputFormat" placeholder="请选择输出文件格式">
              <el-option label="JSON" value="json" />
              <el-option label="YAML" value="yaml" />
              <el-option label="XML" value="xml" />
            </el-select>
          </el-form-item>

          <el-form-item>
            <el-button type="primary" @click="generateOpenapiSpec">生成OpenAPI规范文件</el-button>
          </el-form-item>
        </el-form>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { ElMessage } from 'element-plus';

// OpenAPI平台类型
interface OpenapiPlatform {
  id: number;
  name: string;
  externalIp: string;
  externalPort: number;
  selectedApis: number[];
  [key: string]: unknown;
}

// OpenAPI接口类型
interface OpenapiApi {
  id: number;
  name: string;
  path: string;
  method: string;
  required: boolean;
  description: string;
  [key: string]: unknown;
}

// OpenAPI设置类型
interface OpenapiSettings {
  platforms: OpenapiPlatform[];
  outputFormat: string;
  [key: string]: unknown;
}

// OpenAPI设置
const openapiSettings = reactive<OpenapiSettings>({
  platforms: [
    {
      id: 1,
      name: '平台1',
      externalIp: '',
      externalPort: 8080,
      selectedApis: [],
    },
  ],
  outputFormat: 'json', // json, yaml, xml
});

// OpenAPI接口列表
const openapiApis = ref<OpenapiApi[]>([
  {
    id: 1,
    name: '指定车辆信息查询',
    path: '/api/openapi/vehicles/{vehicle_id}',
    method: 'GET',
    required: true,
    description: '查询指定车辆的详细信息，包括车牌、额定总重、额定载重、所属公司、司机、联系电话、保险等基础信息',
  },
  {
    id: 2,
    name: '指定车辆轨迹信息查询',
    path: '/api/openapi/vehicles/{vehicle_id}/track',
    method: 'GET',
    required: true,
    description: '查询指定车辆的轨迹信息，包括时间、经纬度、重量',
  },
  {
    id: 3,
    name: '在线车辆信息查询',
    path: '/api/openapi/vehicles/online',
    method: 'GET',
    required: false,
    description: '查询所有在线车辆的信息',
  },
  {
    id: 4,
    name: '装卸点信息查询',
    path: '/api/openapi/loading-points',
    method: 'GET',
    required: false,
    description: '查询所有装卸点的信息',
  },
  {
    id: 5,
    name: '车辆汇总报表',
    path: '/api/openapi/reports/vehicles',
    method: 'GET',
    required: false,
    description: '获取车辆汇总报表',
  },
  {
    id: 6,
    name: '车队汇总报表',
    path: '/api/openapi/reports/vehicle-groups',
    method: 'GET',
    required: false,
    description: '获取车队汇总报表',
  },
  {
    id: 7,
    name: '区域汇总报表',
    path: '/api/openapi/reports/areas',
    method: 'GET',
    required: false,
    description: '获取区域汇总报表',
  },
  {
    id: 8,
    name: '装载点作业报表',
    path: '/api/openapi/reports/loading-points',
    method: 'GET',
    required: false,
    description: '获取装载点作业报表',
  },
  {
    id: 9,
    name: '卸载点作业报表',
    path: '/api/openapi/reports/unloading-points',
    method: 'GET',
    required: false,
    description: '获取卸载点作业报表',
  },
  {
    id: 10,
    name: '运输任务报表',
    path: '/api/openapi/reports/tasks',
    method: 'GET',
    required: false,
    description: '获取运输任务报表',
  },
]);

// 添加平台
const addPlatform = () => {
  const newId = openapiSettings.platforms.length > 0 ? Math.max(...openapiSettings.platforms.map((p) => p.id)) + 1 : 1;

  openapiSettings.platforms.push({
    id: newId,
    name: `平台${openapiSettings.platforms.length + 1}`,
    externalIp: '',
    externalPort: 8080,
    selectedApis: [],
  });
};

// 删除平台
const removePlatform = (id: number) => {
  const index = openapiSettings.platforms.findIndex((p) => p.id === id);
  if (index > -1) {
    openapiSettings.platforms.splice(index, 1);
  }
};

// 生成OpenAPI规范文件
const generateOpenapiSpec = () => {
  try {
    // 构建OpenAPI规范对象
    const spec: {
      openapi: string;
      info: { title: string; version: string; description: string };
      servers: { url: string; description: string }[];
      paths: Record<string, any>;
      components: { schemas: Record<string, any> };
    } = {
      openapi: '3.0.0',
      info: {
        title: 'CarpTMS OpenAPI',
        version: '1.0.0',
        description: 'CarpTMS 开放平台API规范',
      },
      servers: openapiSettings.platforms.map((platform) => ({
        url: `http://${platform.externalIp}:${platform.externalPort}`,
        description: platform.name,
      })),
      paths: {},
      components: {
        schemas: {},
      },
    };

    // 添加选择的API接口
    openapiSettings.platforms.forEach((platform) => {
      platform.selectedApis.forEach((apiId) => {
        const apiDef = openapiApis.value.find((a) => a.id === apiId);
        if (apiDef) {
          // 构建路径
          const path = apiDef.path;
          if (!spec.paths[path]) {
            spec.paths[path] = {};
          }

          // 添加方法
          spec.paths[path][apiDef.method.toLowerCase()] = {
            summary: apiDef.name,
            description: apiDef.description,
            responses: {
              '200': {
                description: '成功',
              },
            },
          };
        }
      });
    });

    // 根据选择的格式生成文件
    let content = '';
    let filename = 'openapi';

    switch (openapiSettings.outputFormat) {
      case 'json':
        content = JSON.stringify(spec, null, 2);
        filename += '.json';
        break;
      case 'yaml':
        content = JSON.stringify(spec, null, 2); // 这里简化处理，实际应该转换为YAML
        filename += '.yaml';
        break;
      case 'xml':
        content = JSON.stringify(spec, null, 2); // 这里简化处理，实际应该转换为XML
        filename += '.xml';
        break;
      default:
        content = JSON.stringify(spec, null, 2);
        filename += '.json';
    }

    // 创建并下载文件
    const blob = new Blob([content], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    ElMessage.success('OpenAPI规范文件生成成功');
  } catch (error) {
    console.error('生成OpenAPI规范文件失败:', error);
    ElMessage.error('生成OpenAPI规范文件失败');
  }
};
</script>

<style scoped>
/* 样式可以根据需要添加 */
</style>
