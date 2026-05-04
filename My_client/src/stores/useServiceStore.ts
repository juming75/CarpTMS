import { defineStore } from 'pinia';
import { ref } from 'vue';

interface UnifiedCommunicationService {
  connect: () => Promise<void>;
  disconnect: () => void;
  send: (data: unknown) => Promise<void>;
  on: (event: string, callback: (data: unknown) => void) => void;
  off: (event: string, callback: (data: unknown) => void) => void;
}

interface SyncServiceInterface {
  initialize: () => Promise<void>;
  destroy: () => void;
  triggerSync: () => Promise<void>;
}

interface MonitoringServiceInterface {
  init: () => void;
  cleanup: () => void;
}

export const useServiceStore = defineStore('services', () => {
  const newServerService = ref<UnifiedCommunicationService | null>(null);
  const legacyServerService = ref<UnifiedCommunicationService | null>(null);
  const syncService = ref<SyncServiceInterface | null>(null);
  const monitoringService = ref<MonitoringServiceInterface | null>(null);

  const setNewServerService = (service: UnifiedCommunicationService | null): void => {
    newServerService.value = service;
  };

  const setLegacyServerService = (service: UnifiedCommunicationService | null): void => {
    legacyServerService.value = service;
  };

  const setSyncService = (service: SyncServiceInterface | null): void => {
    syncService.value = service;
  };

  const setMonitoringService = (service: MonitoringServiceInterface | null): void => {
    monitoringService.value = service;
  };

  const clearAllServices = (): void => {
    newServerService.value = null;
    legacyServerService.value = null;
    syncService.value = null;
    monitoringService.value = null;
  };

  return {
    newServerService,
    legacyServerService,
    syncService,
    monitoringService,
    setNewServerService,
    setLegacyServerService,
    setSyncService,
    setMonitoringService,
    clearAllServices,
  };
});
