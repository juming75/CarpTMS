import { describe, it, expect, beforeEach } from 'vitest';
import { jwtBlacklist } from './jwtBlacklist';

const createValidToken = (expDays: number = 1): string => {
  const payload = {
    sub: 'test-user',
    exp: Math.floor(Date.now() / 1000) + expDays * 24 * 60 * 60,
  };
  const header = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
  const payloadB64 = btoa(JSON.stringify(payload));
  return `${header}.${payloadB64}.signature`;
};

describe('jwtBlacklist', () => {
  beforeEach(() => {
    jwtBlacklist.clearAll();
  });

  describe('addToBlacklist', () => {
    it('should add a valid token to the blacklist', () => {
      const token = createValidToken();
      const result = jwtBlacklist.addToBlacklist(token);
      expect(result).toBe(true);
      expect(jwtBlacklist.isBlacklisted(token)).toBe(true);
    });

    it('should add multiple tokens to the blacklist', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      const token3 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      jwtBlacklist.addToBlacklist(token2);
      jwtBlacklist.addToBlacklist(token3);
      
      expect(jwtBlacklist.isBlacklisted(token1)).toBe(true);
      expect(jwtBlacklist.isBlacklisted(token2)).toBe(true);
      expect(jwtBlacklist.isBlacklisted(token3)).toBe(true);
    });

    it('should not add expired tokens', () => {
      const expiredToken = createValidToken(-1);
      const result = jwtBlacklist.addToBlacklist(expiredToken);
      expect(result).toBe(false);
    });

    it('should not affect tokens not in the blacklist', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      expect(jwtBlacklist.isBlacklisted(token2)).toBe(false);
    });
  });

  describe('isBlacklisted', () => {
    it('should return true for blacklisted tokens', () => {
      const token = createValidToken();
      jwtBlacklist.addToBlacklist(token);
      expect(jwtBlacklist.isBlacklisted(token)).toBe(true);
    });

    it('should return false for non-blacklisted tokens', () => {
      const token = createValidToken();
      expect(jwtBlacklist.isBlacklisted(token)).toBe(false);
    });

    it('should return false for empty string', () => {
      expect(jwtBlacklist.isBlacklisted('')).toBe(false);
    });
  });

  describe('removeFromBlacklist', () => {
    it('should remove a token from the blacklist', () => {
      const token = createValidToken();
      jwtBlacklist.addToBlacklist(token);
      expect(jwtBlacklist.isBlacklisted(token)).toBe(true);
      
      jwtBlacklist.removeFromBlacklist(token);
      expect(jwtBlacklist.isBlacklisted(token)).toBe(false);
    });

    it('should not affect other tokens when removing', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      jwtBlacklist.addToBlacklist(token2);
      jwtBlacklist.removeFromBlacklist(token1);
      
      expect(jwtBlacklist.isBlacklisted(token1)).toBe(false);
      expect(jwtBlacklist.isBlacklisted(token2)).toBe(true);
    });
  });

  describe('clearAll', () => {
    it('should clear all tokens from the blacklist', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      const token3 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      jwtBlacklist.addToBlacklist(token2);
      jwtBlacklist.addToBlacklist(token3);
      jwtBlacklist.clearAll();
      
      expect(jwtBlacklist.isBlacklisted(token1)).toBe(false);
      expect(jwtBlacklist.isBlacklisted(token2)).toBe(false);
      expect(jwtBlacklist.isBlacklisted(token3)).toBe(false);
    });
  });

  describe('getBlacklistSize', () => {
    it('should return correct size', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      jwtBlacklist.addToBlacklist(token2);
      
      expect(jwtBlacklist.getBlacklistSize()).toBe(2);
    });

    it('should return 0 when empty', () => {
      expect(jwtBlacklist.getBlacklistSize()).toBe(0);
    });
  });

  describe('clearExpired', () => {
    it('should remove expired tokens', () => {
      const expiredToken = createValidToken(-1);
      const validToken = createValidToken();
      
      jwtBlacklist.addToBlacklist(expiredToken);
      jwtBlacklist.addToBlacklist(validToken);
      
      jwtBlacklist.clearExpired();
      
      expect(jwtBlacklist.isBlacklisted(expiredToken)).toBe(false);
      expect(jwtBlacklist.isBlacklisted(validToken)).toBe(true);
    });
  });

  describe('getBlacklistedTokens', () => {
    it('should return all blacklisted tokens', () => {
      const token1 = createValidToken();
      const token2 = createValidToken();
      
      jwtBlacklist.addToBlacklist(token1);
      jwtBlacklist.addToBlacklist(token2);
      
      const tokens = jwtBlacklist.getBlacklistedTokens();
      const tokenStrings = tokens.map(t => t.token);
      
      expect(tokenStrings).toContain(token1);
      expect(tokenStrings).toContain(token2);
    });

    it('should return empty array when no tokens', () => {
      const tokens = jwtBlacklist.getBlacklistedTokens();
      expect(tokens).toEqual([]);
    });
  });
});
