interface JwtKeyCache {
  kid: string;
  publicKey: string;
  algorithm: string;
  expiresAt: number;
  fetchedAt: number;
}

export class JwtKeyCache {
  private cache = new Map<string, JwtKeyCache>();
  private defaultTtl = 3600 * 1000;
  private fetchingKeys = new Set<string>();

  getKey(kid: string): JwtKeyCache | null {
    const cached = this.cache.get(kid);
    if (!cached) return null;
    if (Date.now() > cached.expiresAt) {
      this.cache.delete(kid);
      return null;
    }
    return cached;
  }

  setKey(kid: string, publicKey: string, algorithm: string, ttl?: number) {
    const expiresAt = Date.now() + (ttl || this.defaultTtl);
    this.cache.set(kid, {
      kid,
      publicKey,
      algorithm,
      expiresAt,
      fetchedAt: Date.now(),
    });
  }

  deleteKey(kid: string) {
    this.cache.delete(kid);
  }

  clear() {
    this.cache.clear();
  }

  isFetching(kid: string): boolean {
    return this.fetchingKeys.has(kid);
  }

  markFetching(kid: string, fetching: boolean) {
    if (fetching) {
      this.fetchingKeys.add(kid);
    } else {
      this.fetchingKeys.delete(kid);
    }
  }

  getStats() {
    return {
      size: this.cache.size,
      keys: Array.from(this.cache.keys()),
    };
  }
}

export const jwtKeyCache = new JwtKeyCache();