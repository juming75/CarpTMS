/**
 * @deprecated 未被使用的 Store - 保留仅供参考
 * 如需使用车辆数据管理，请在相关视图中导入此 Store
 * 如果确认不再需要，可以删除此文件
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Vehicle } from '../types';

export const useVehicleStore = defineStore('vehicle', () => {
  const vehicles = ref<Vehicle[]>([]);
  const loading = ref(false);
  const last_updated = ref<Date | null>(null);

  const get_vehicle_by_id = computed(() => (vehicle_id: number) => {
    return vehicles.value.find(vehicle => vehicle.vehicle_id === vehicle_id);
  });

  const get_vehicles_by_status = computed(() => (status: string) => {
    return vehicles.value.filter(vehicle => vehicle.operation_status === status);
  });

  const set_vehicles = (new_vehicles: Vehicle[]) => {
    vehicles.value = new_vehicles;
    last_updated.value = new Date();
  };

  const add_vehicle = (vehicle: Vehicle) => {
    vehicles.value.push(vehicle);
    last_updated.value = new Date();
  };

  const update_vehicle = (updated_vehicle: Vehicle) => {
    const index = vehicles.value.findIndex(v => v.vehicle_id === updated_vehicle.vehicle_id);
    if (index !== -1) {
      vehicles.value[index] = updated_vehicle;
      last_updated.value = new Date();
    }
  };

  const remove_vehicle = (vehicle_id: number) => {
    const index = vehicles.value.findIndex(v => v.vehicle_id === vehicle_id);
    if (index !== -1) {
      vehicles.value.splice(index, 1);
      last_updated.value = new Date();
    }
  };

  const clear_vehicles = () => {
    vehicles.value = [];
    last_updated.value = null;
  };

  return {
    vehicles,
    loading,
    last_updated,
    get_vehicle_by_id,
    get_vehicles_by_status,
    set_vehicles,
    add_vehicle,
    update_vehicle,
    remove_vehicle,
    clear_vehicles
  };
});


