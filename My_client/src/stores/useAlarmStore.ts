/**
 * @deprecated 未被使用的 Store - 保留仅供参考
 * 如需使用报警数据管理，请在相关视图中导入此 Store
 * 如果确认不再需要，可以删除此文件
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

interface Alarm {
  id: number;
  vehicle_id: number;
  plate_number: string;
  alarm_type: string;
  alarm_level: string;
  alarm_time: string;
  location?: {
    latitude: number;
    longitude: number;
  };
  status: string;
  handled: boolean;
}

export const useAlarmStore = defineStore('alarm', () => {
  const alarms = ref<Alarm[]>([]);
  const loading = ref(false);
  const last_updated = ref<Date | null>(null);

  const unhandled_alarms = computed(() => {
    return alarms.value.filter(alarm => !alarm.handled);
  });

  const get_alarms_by_vehicle_id = computed(() => (vehicle_id: number) => {
    return alarms.value.filter(alarm => alarm.vehicle_id === vehicle_id);
  });

  const get_alarms_by_type = computed(() => (type: string) => {
    return alarms.value.filter(alarm => alarm.alarm_type === type);
  });

  const set_alarms = (new_alarms: Alarm[]) => {
    alarms.value = new_alarms;
    last_updated.value = new Date();
  };

  const add_alarm = (alarm: Alarm) => {
    alarms.value.unshift(alarm); // 添加到开头，最新的报警显示在前面
    last_updated.value = new Date();
  };

  const update_alarm = (updated_alarm: Alarm) => {
    const index = alarms.value.findIndex(a => a.id === updated_alarm.id);
    if (index !== -1) {
      alarms.value[index] = updated_alarm;
      last_updated.value = new Date();
    }
  };

  const mark_as_handled = (id: number) => {
    const alarm = alarms.value.find(a => a.id === id);
    if (alarm) {
      alarm.handled = true;
      alarm.status = '已处理';
      last_updated.value = new Date();
    }
  };

  const clear_alarms = () => {
    alarms.value = [];
    last_updated.value = null;
  };

  return {
    alarms,
    loading,
    last_updated,
    unhandled_alarms,
    get_alarms_by_vehicle_id,
    get_alarms_by_type,
    set_alarms,
    add_alarm,
    update_alarm,
    mark_as_handled,
    clear_alarms
  };
});


