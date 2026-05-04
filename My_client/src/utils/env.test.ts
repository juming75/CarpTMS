import { describe, it, expect, vi, beforeEach } from 'vitest';
import { isRemoteOpsEnabled } from './env';

describe('env', () => {
  beforeEach(() => {
    vi.unstubAllEnvs();
  });

  describe('isRemoteOpsEnabled', () => {
    it('should return true when VITE_ENABLE_REMOTE_OPS is "true"', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', 'true');
      expect(isRemoteOpsEnabled()).toBe(true);
    });

    it('should return true when VITE_ENABLE_REMOTE_OPS is "1"', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', '1');
      expect(isRemoteOpsEnabled()).toBe(true);
    });

    it('should return false when VITE_ENABLE_REMOTE_OPS is "false"', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', 'false');
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should return false when VITE_ENABLE_REMOTE_OPS is "0"', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', '0');
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should return false when VITE_ENABLE_REMOTE_OPS is empty string', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', '');
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should return false when VITE_ENABLE_REMOTE_OPS is undefined', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', undefined);
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should return false when VITE_ENABLE_REMOTE_OPS is not set', () => {
      vi.unstubAllEnvs();
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should handle case-insensitive comparison', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', 'TRUE');
      expect(isRemoteOpsEnabled()).toBe(true);

      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', 'False');
      expect(isRemoteOpsEnabled()).toBe(false);
    });

    it('should trim whitespace', () => {
      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', '  true  ');
      expect(isRemoteOpsEnabled()).toBe(true);

      vi.stubEnv('VITE_ENABLE_REMOTE_OPS', '  false  ');
      expect(isRemoteOpsEnabled()).toBe(false);
    });
  });
});
