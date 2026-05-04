interface BlacklistedToken {
  token: string;
  expiresAt: number;
  blacklistedAt: number;
  reason?: string;
}

export class JWTBlacklist {
  private blacklist: Map<string, BlacklistedToken> = new Map();
  private readonly STORAGE_KEY = 'jwt_blacklist';
  private cleanupInterval: number | null = null;

  constructor() {
    this.loadFromStorage();
    this.startCleanup();
  }

  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (stored) {
        const tokens: BlacklistedToken[] = JSON.parse(stored);
        const now = Date.now();
        tokens.forEach((t) => {
          if (t.expiresAt > now) {
            this.blacklist.set(t.token, t);
          }
        });
      }
    } catch (error) {
      console.error('Failed to load JWT blacklist from storage:', error);
    }
  }

  private saveToStorage(): void {
    try {
      const tokens = Array.from(this.blacklist.values());
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(tokens));
    } catch (error) {
      console.error('Failed to save JWT blacklist to storage:', error);
    }
  }

  private extractTokenExpiry(token: string): number | null {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;
      const payload = JSON.parse(atob(parts[1]));
      return payload.exp ? payload.exp * 1000 : null;
    } catch {
      return null;
    }
  }

  addToBlacklist(token: string, reason?: string, customExpiry?: number): boolean {
    try {
      const expiry = customExpiry || this.extractTokenExpiry(token);
      if (!expiry || expiry < Date.now()) {
        // Token already expired
        return false;
      }

      const blacklistedToken: BlacklistedToken = {
        token,
        expiresAt: expiry,
        blacklistedAt: Date.now(),
        reason,
      };

      this.blacklist.set(token, blacklistedToken);
      this.saveToStorage();
      return true;
    } catch (error) {
      console.error('Failed to add token to blacklist:', error);
      return false;
    }
  }

  isBlacklisted(token: string): boolean {
    const blacklisted = this.blacklist.get(token);
    if (!blacklisted) return false;
    if (Date.now() > blacklisted.expiresAt) {
      this.blacklist.delete(token);
      this.saveToStorage();
      return false;
    }
    return true;
  }

  removeFromBlacklist(token: string): boolean {
    if (this.blacklist.delete(token)) {
      this.saveToStorage();
      return true;
    }
    return false;
  }

  clearExpired(): void {
    const now = Date.now();
    let cleared = 0;
    this.blacklist.forEach((token, key) => {
      if (token.expiresAt < now) {
        this.blacklist.delete(key);
        cleared++;
      }
    });
    if (cleared > 0) {
      this.saveToStorage();
    }
  }

  clearAll(): void {
    this.blacklist.clear();
    localStorage.removeItem(this.STORAGE_KEY);
  }

  getBlacklistSize(): number {
    return this.blacklist.size;
  }

  getBlacklistedTokens(): BlacklistedToken[] {
    return Array.from(this.blacklist.values());
  }

  private startCleanup(): void {
    this.cleanupInterval = window.setInterval(() => {
      this.clearExpired();
    }, 60 * 60 * 1000); // 每小时清理一次
  }

  destroy(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
  }
}

export const jwtBlacklist = new JWTBlacklist();
