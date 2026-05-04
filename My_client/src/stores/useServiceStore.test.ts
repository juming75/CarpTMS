import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useServiceStore } from './useServiceStore';

describe('useServiceStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('initial state', () => {
    it('should have null services initially', () => {
      const store = useServiceStore();
      expect(store.newServerService).toBeNull();
      expect(store.legacyServerService).toBeNull();
      expect(store.syncService).toBeNull();
      expect(store.monitoringService).toBeNull();
    });
  });

  describe('setNewServerService', () => {
    it('should set new server service', () => {
      const store = useServiceStore();
      const mockService = {
        connect: vi.fn(),
        disconnect: vi.fn(),
        send: vi.fn(),
        on: vi.fn(),
        off: vi.fn(),
      };

      store.setNewServerService(mockService);

      expect(store.newServerService).toBe(mockService);
    });

    it('should clear new server service when set to null', () => {
      const store = useServiceStore();
      const mockService = {
        connect: vi.fn(),
        disconnect: vi.fn(),
        send: vi.fn(),
        on: vi.fn(),
        off: vi.fn(),
      };

      store.setNewServerService(mockService);
      store.setNewServerService(null);

      expect(store.newServerService).toBeNull();
    });
  });

  describe('setLegacyServerService', () => {
    it('should set legacy server service', () => {
      const store = useServiceStore();
      const mockService = {
        connect: vi.fn(),
        disconnect: vi.fn(),
        send: vi.fn(),
        on: vi.fn(),
        off: vi.fn(),
      };

      store.setLegacyServerService(mockService);

      expect(store.legacyServerService).toBe(mockService);
    });
  });

  describe('setSyncService', () => {
    it('should set sync service', () => {
      const store = useServiceStore();
      const mockService = {
        initialize: vi.fn(),
        destroy: vi.fn(),
        triggerSync: vi.fn(),
      };

      store.setSyncService(mockService);

      expect(store.syncService).toBe(mockService);
    });
  });

  describe('setMonitoringService', () => {
    it('should set monitoring service', () => {
      const store = useServiceStore();
      const mockService = {
        init: vi.fn(),
        cleanup: vi.fn(),
      };

      store.setMonitoringService(mockService);

      expect(store.monitoringService).toBe(mockService);
    });
  });

  describe('clearAllServices', () => {
    it('should clear all services', () => {
      const store = useServiceStore();
      const mockService = {
        connect: vi.fn(),
        disconnect: vi.fn(),
        send: vi.fn(),
        on: vi.fn(),
        off: vi.fn(),
      };

      store.setNewServerService(mockService);
      store.setLegacyServerService(mockService);
      store.setSyncService({
        initialize: vi.fn(),
        destroy: vi.fn(),
        triggerSync: vi.fn(),
      });
      store.setMonitoringService({
        init: vi.fn(),
        cleanup: vi.fn(),
      });

      store.clearAllServices();

      expect(store.newServerService).toBeNull();
      expect(store.legacyServerService).toBeNull();
      expect(store.syncService).toBeNull();
      expect(store.monitoringService).toBeNull();
    });
  });
});
