/**
 * @deprecated 未被使用的 Store - 保留仅供参考
 * 如需使用订单数据管理，请在相关视图中导入此 Store
 * 如果确认不再需要，可以删除此文件
 */
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Order } from '../types';

export const useOrderStore = defineStore('order', () => {
  const orders = ref<Order[]>([]);
  const loading = ref(false);
  const last_updated = ref<Date | null>(null);

  const get_order_by_id = computed(() => (order_id: number) => {
    return orders.value.find(order => order.order_id === order_id);
  });

  const get_orders_by_status = computed(() => (status: number) => {
    return orders.value.filter(order => order.order_status === status);
  });

  const set_orders = (new_orders: Order[]) => {
    orders.value = new_orders;
    last_updated.value = new Date();
  };

  const add_order = (order: Order) => {
    orders.value.push(order);
    last_updated.value = new Date();
  };

  const update_order = (updated_order: Order) => {
    const index = orders.value.findIndex(o => o.order_id === updated_order.order_id);
    if (index !== -1) {
      orders.value[index] = updated_order;
      last_updated.value = new Date();
    }
  };

  const remove_order = (order_id: number) => {
    const index = orders.value.findIndex(o => o.order_id === order_id);
    if (index !== -1) {
      orders.value.splice(index, 1);
      last_updated.value = new Date();
    }
  };

  const clear_orders = () => {
    orders.value = [];
    last_updated.value = null;
  };

  return {
    orders,
    loading,
    last_updated,
    get_order_by_id,
    get_orders_by_status,
    set_orders,
    add_order,
    update_order,
    remove_order,
    clear_orders
  };
});
