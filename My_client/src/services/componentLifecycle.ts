interface ComponentInfo {
  name: string;
  instance: any;
  mountedAt: number;
  isActive: boolean;
  cleanup: (() => void)[];
}

export class ComponentLifecycleManager {
  private components = new Map<string, ComponentInfo>();
  private componentIdCounter = 0;

  register(component: any, name: string): string {
    const id = `comp_${this.componentIdCounter++}_${Date.now()}`;
    this.components.set(id, {
      name,
      instance: component,
      mountedAt: Date.now(),
      isActive: true,
      cleanup: [],
    });
    return id;
  }

  addCleanup(id: string, cleanupFn: () => void) {
    const comp = this.components.get(id);
    if (comp) {
      comp.cleanup.push(cleanupFn);
    }
  }

  unregister(id: string) {
    const comp = this.components.get(id);
    if (comp) {
      comp.cleanup.forEach(fn => {
        try {
          fn();
        } catch (e) {
          console.error('Cleanup error:', e);
        }
      });
      comp.isActive = false;
      this.components.delete(id);
    }
  }

  getActiveComponents() {
    return Array.from(this.components.values()).filter(c => c.isActive);
  }

  cleanupAll() {
    this.components.forEach((comp, id) => {
      this.unregister(id);
    });
  }

  getStats() {
    const active = this.getActiveComponents();
    return {
      total: this.components.size,
      active: active.length,
      names: active.map(c => c.name),
    };
  }
}

export const componentLifecycleManager = new ComponentLifecycleManager();