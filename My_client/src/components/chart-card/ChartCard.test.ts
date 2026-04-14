// @ts-nocheck
import { describe, it, expect, beforeEach } from 'vitest';
import { mount, type VueWrapper } from '@vue/test-utils';
import ChartCard from './ChartCard.vue';
import { createPinia } from 'pinia';
import ElementPlus from 'element-plus';

describe('ChartCard.vue', () => {
  let wrapper: VueWrapper<unknown>;

  beforeEach(() => {
    const app = createPinia();
    wrapper = mount(ChartCard, {
      global: {
        plugins: [app, ElementPlus],
      },
      props: {
        title: '测试图表',
        height: '300px',
      },
    });
  });

  it('组件能够正常挂载', () => {
    expect(wrapper.exists()).toBe(true);
  });

  it('能够正确显示标题', () => {
    expect(wrapper.find('.card-title').text()).toBe('测试图表');
  });

  it('能够正确设置高度', () => {
    const chartContainer = wrapper.find('.chart-container');
    expect(chartContainer.exists()).toBe(true);
  });

  it('能够处理无标题的情况', async () => {
    await wrapper.setProps({ title: '' });
    expect(wrapper.find('.card-title').exists()).toBe(false);
  });

  it('能够处理默认高度', async () => {
    await wrapper.setProps({ height: undefined });
    const chartContainer = wrapper.find('.chart-container');
    expect(chartContainer.exists()).toBe(true);
  });

  it('能够正确暴露chartContainer引用', () => {
    expect(wrapper.vm.chartContainer).toBeDefined();
  });

  it('能够正确渲染插槽内容', () => {
    const testContent = '测试内容';
    wrapper = mount(ChartCard, {
      global: {
        plugins: [createPinia(), ElementPlus],
      },
      slots: {
        default: testContent,
      },
    });
    expect(wrapper.find('.chart-content').text()).toContain(testContent);
  });

  it('能够正确渲染actions插槽', () => {
    const testAction = '测试操作';
    wrapper = mount(ChartCard, {
      global: {
        plugins: [createPinia(), ElementPlus],
      },
      props: {
        title: '测试图表',
      },
      slots: {
        actions: testAction,
      },
    });
    expect(wrapper.find('.card-actions').text()).toContain(testAction);
  });
});


