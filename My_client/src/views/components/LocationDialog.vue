<template>
  <el-dialog
    :model-value="visible"
    :title="getDialogTitle()"
    width="600px"
    @update:model-value="$emit('update:visible', $event)"
  >
    <!-- 电子围栏表单 -->
    <el-form v-if="type === 'fence'" ref="formRef" :model="formData" label-width="120px">
      <el-form-item label="围栏名称" required>
        <el-input v-model="formData.fence_name" placeholder="请输入围栏名称" />
      </el-form-item>
      <el-form-item label="围栏类型" required>
        <el-select v-model="formData.fence_type" placeholder="请选择围栏类型">
          <el-option label="圆形" value="circle" />
          <el-option label="多边形" value="polygon" />
          <el-option label="矩形" value="rectangle" />
        </el-select>
      </el-form-item>
      <el-form-item label="中心纬度" v-if="formData.fence_type === 'circle'">
        <el-input-number v-model="formData.center_latitude" :precision="8" :step="0.000001" />
      </el-form-item>
      <el-form-item label="中心经度" v-if="formData.fence_type === 'circle'">
        <el-input-number v-model="formData.center_longitude" :precision="8" :step="0.000001" />
      </el-form-item>
      <el-form-item label="半径 (米)" v-if="formData.fence_type === 'circle'">
        <el-input-number v-model="formData.radius" :min="0" />
      </el-form-item>
      <el-form-item label="状态">
        <el-switch v-model="formData.status" active-value="active" inactive-value="inactive" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="formData.description" type="textarea" :rows="3" />
      </el-form-item>
    </el-form>

    <!-- 位置表单 -->
    <el-form v-else-if="type === 'position'" ref="formRef" :model="formData" label-width="120px">
      <el-form-item label="位置名称" required>
        <el-input v-model="formData.place_name" placeholder="请输入位置名称" />
      </el-form-item>
      <el-form-item label="纬度" required>
        <el-input-number v-model="formData.latitude" :precision="8" :min="-90" :max="90" />
      </el-form-item>
      <el-form-item label="经度" required>
        <el-input-number v-model="formData.longitude" :precision="8" :min="-180" :max="180" />
      </el-form-item>
      <el-form-item label="地址">
        <el-input v-model="formData.address" placeholder="请输入详细地址" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="formData.description" type="textarea" :rows="3" />
      </el-form-item>
    </el-form>

    <!-- 地点表单 -->
    <el-form v-else-if="type === 'place'" ref="formRef" :model="formData" label-width="120px">
      <el-form-item label="地点名称" required>
        <el-input v-model="formData.place_name" placeholder="请输入地点名称" />
      </el-form-item>
      <el-form-item label="地址" required>
        <el-input v-model="formData.address" placeholder="请输入详细地址" />
      </el-form-item>
      <el-form-item label="联系人">
        <el-input v-model="formData.contact_person" placeholder="请输入联系人姓名" />
      </el-form-item>
      <el-form-item label="联系电话">
        <el-input v-model="formData.contact_phone" placeholder="请输入联系电话" />
      </el-form-item>
      <el-form-item label="联系邮箱">
        <el-input v-model="formData.contact_email" placeholder="请输入联系邮箱" />
      </el-form-item>
      <el-form-item label="纬度">
        <el-input-number v-model="formData.latitude" :precision="8" :min="-90" :max="90" />
      </el-form-item>
      <el-form-item label="经度">
        <el-input-number v-model="formData.longitude" :precision="8" :min="-180" :max="180" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="formData.description" type="textarea" :rows="3" />
      </el-form-item>
    </el-form>

    <!-- 路线表单 -->
    <el-form v-else-if="type === 'route'" ref="formRef" :model="formData" label-width="120px">
      <el-form-item label="路线名称" required>
        <el-input v-model="formData.route_name" placeholder="请输入路线名称" />
      </el-form-item>
      <el-form-item label="起点" required>
        <el-input v-model="formData.start_point" placeholder="请输入起点名称" />
      </el-form-item>
      <el-form-item label="起点纬度">
        <el-input-number v-model="formData.start_latitude" :precision="8" :min="-90" :max="90" />
      </el-form-item>
      <el-form-item label="起点经度">
        <el-input-number v-model="formData.start_longitude" :precision="8" :min="-180" :max="180" />
      </el-form-item>
      <el-form-item label="终点" required>
        <el-input v-model="formData.end_point" placeholder="请输入终点名称" />
      </el-form-item>
      <el-form-item label="终点纬度">
        <el-input-number v-model="formData.end_latitude" :precision="8" :min="-90" :max="90" />
      </el-form-item>
      <el-form-item label="终点经度">
        <el-input-number v-model="formData.end_longitude" :precision="8" :min="-180" :max="180" />
      </el-form-item>
      <el-form-item label="途经点">
        <el-input v-model="formData.waypoints" type="textarea" :rows="3" placeholder="请输入途经点，格式为 JSON 数组" />
      </el-form-item>
      <el-form-item label="距离 (km)">
        <el-input-number v-model="formData.distance" :min="0" :precision="2" />
      </el-form-item>
      <el-form-item label="预计耗时 (分钟)">
        <el-input-number v-model="formData.estimated_duration" :min="0" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="formData.description" type="textarea" :rows="3" />
      </el-form-item>
    </el-form>

    <template #footer>
      <span class="dialog-footer">
        <el-button @click="$emit('update:visible', false)">取消</el-button>
        <el-button type="primary" @click="handleSave">保存</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, watch } from 'vue';
import type { LocationFormData, TypeMap } from '../../types/location';

const props = defineProps({
  visible: Boolean,
  type: String,
  data: Object,
});

const emit = defineEmits(['update:visible', 'save']);

// 表单数据
const formData = ref<LocationFormData>({});
const formRef = ref();

// 监听数据和类型变化
watch(
  () => props.data,
  (newData) => {
    if (newData) {
      formData.value = { ...newData };
    } else {
      // 初始化空表单
      if (props.type === 'fence') {
        formData.value = {
          fence_name: '',
          fence_type: 'circle',
          center_latitude: null,
          center_longitude: null,
          radius: null,
          status: 'active',
          description: '',
        };
      } else if (props.type === 'position') {
        formData.value = {
          place_name: '',
          latitude: 39.9042,
          longitude: 116.4074,
          address: '',
          description: '',
        };
      } else if (props.type === 'place') {
        formData.value = {
          place_name: '',
          address: '',
          contact_person: '',
          contact_phone: '',
          contact_email: '',
          latitude: null,
          longitude: null,
          description: '',
        };
      } else if (props.type === 'route') {
        formData.value = {
          route_name: '',
          start_point: '',
          start_latitude: null,
          start_longitude: null,
          end_point: '',
          end_latitude: null,
          end_longitude: null,
          waypoints: null,
          distance: null,
          estimated_duration: null,
          description: '',
        };
      }
    }
  },
  { immediate: true }
);

// 获取对话框标题
const getDialogTitle = () => {
  const prefix = props.data ? '编辑' : '新建';
  const typeMap: TypeMap = {
    fence: '电子围栏',
    position: '位置',
    place: '地点',
    route: '路线',
  };
  return `${prefix}${typeMap[props.type as keyof TypeMap] || ''}`;
};

// 保存处理
const handleSave = () => {
  emit('save', props.type, formData.value);
};
</script>

<style scoped>
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


