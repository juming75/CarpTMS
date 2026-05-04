import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useAuthStore } from './useAuthStore';

describe('useAuthStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  describe('initial state', () => {
    it('should have null user initially', () => {
      const store = useAuthStore();
      expect(store.user).toBeNull();
    });

    it('should not be authenticated initially', () => {
      const store = useAuthStore();
      expect(store.is_authenticated).toBe(false);
    });
  });

  describe('login', () => {
    it('should set user and authenticate', () => {
      const store = useAuthStore();
      const testUser = {
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read', 'write'],
      };

      store.login(testUser);

      expect(store.user).toEqual(testUser);
      expect(store.is_authenticated).toBe(true);
    });

    it('should persist user to localStorage', () => {
      const store = useAuthStore();
      const testUser = {
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read', 'write'],
      };

      store.login(testUser);

      const stored = localStorage.getItem('user');
      expect(stored).not.toBeNull();
      expect(JSON.parse(stored!)).toEqual(testUser);
    });
  });

  describe('logout', () => {
    it('should clear user and deauthenticate', async () => {
      const store = useAuthStore();
      store.login({
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read'],
      });

      await store.logout();

      expect(store.user).toBeNull();
      expect(store.is_authenticated).toBe(false);
    });

    it('should clear localStorage', async () => {
      const store = useAuthStore();
      store.login({
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read'],
      });

      await store.logout();

      expect(localStorage.getItem('user')).toBeNull();
      expect(localStorage.getItem('userId')).toBeNull();
    });
  });

  describe('update_user', () => {
    it('should update user properties', () => {
      const store = useAuthStore();
      store.login({
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read'],
      });

      store.update_user({ role: 'superadmin' });

      expect(store.user?.role).toBe('superadmin');
    });

    it('should not update if user is null', () => {
      const store = useAuthStore();

      store.update_user({ role: 'superadmin' });

      expect(store.user).toBeNull();
    });
  });

  describe('clear_auth', () => {
    it('should clear all auth data', () => {
      const store = useAuthStore();
      store.login({
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read'],
      });
      localStorage.setItem('access_token', 'test-token');
      localStorage.setItem('refresh_token', 'refresh-token');

      store.clear_auth();

      expect(store.user).toBeNull();
      expect(localStorage.getItem('user')).toBeNull();
      expect(localStorage.getItem('access_token')).toBeNull();
      expect(localStorage.getItem('refresh_token')).toBeNull();
    });
  });

  describe('restore_session', () => {
    it('should clear user if no userId in localStorage', async () => {
      const store = useAuthStore();
      store.login({
        id: 1,
        username: 'testuser',
        role: 'admin',
        permissions: ['read'],
      });

      await store.restore_session();

      expect(store.user).toBeNull();
    });
  });
});
