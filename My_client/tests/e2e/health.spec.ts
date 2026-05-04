import { test, expect } from '@playwright/test';

test.describe('Health Check API Endpoints', () => {
  const baseUrl = 'http://localhost:8080';

  test('should return health status from basic endpoint', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health`);
    expect([200, 503]).toContain(response.status());
    const body = await response.json();
    expect(body).toHaveProperty('status');
    expect(['ok', 'error', 'warn']).toContain(body.status);
  });

  test('should return liveness status', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/live`);
    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body).toHaveProperty('status', 'alive');
  });

  test('should return readiness status', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/ready`);
    expect([200, 503]).toContain(response.status());
  });

  test('should return enhanced health check with detailed metrics', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/enhanced`);
    expect([200, 503]).toContain(response.status());
    const body = await response.json();

    expect(body).toHaveProperty('status');
    expect(body).toHaveProperty('service');
    expect(body).toHaveProperty('version');
    expect(body).toHaveProperty('timestamp');
    expect(body).toHaveProperty('hostname');
    expect(body).toHaveProperty('system_metrics');
    expect(body).toHaveProperty('dependencies');

    const metrics = body.system_metrics;
    expect(metrics).toHaveProperty('cpu_usage');
    expect(metrics).toHaveProperty('memory_usage');
    expect(metrics).toHaveProperty('disk_usage');
    expect(typeof metrics.cpu_usage).toBe('number');
    expect(typeof metrics.memory_usage).toBe('number');
    expect(typeof metrics.disk_usage).toBe('number');
  });

  test('should return health check history', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/history`);
    expect(response.status()).toBe(200);
    const history = await response.json();
    expect(Array.isArray(history)).toBeTruthy();
  });

  test('should get health check configuration', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/config`);
    expect(response.status()).toBe(200);
    const config = await response.json();

    expect(config).toHaveProperty('enable_all_checks');
    expect(config).toHaveProperty('enabled_checks');
    expect(Array.isArray(config.enabled_checks)).toBeTruthy();
    expect(config).toHaveProperty('check_interval_seconds');
  });

  test('should update health check configuration', async ({ request }) => {
    const newConfig = {
      enable_all_checks: true,
      enabled_checks: ['cpu', 'memory', 'disk', 'database', 'redis'],
      check_interval_seconds: 60,
      custom_thresholds: {
        cpu: { warning: 70.0, critical: 85.0 }
      },
      notification_config: {
        enabled: true,
        channels: ['log', 'webhook'],
        min_severity: 'warning'
      }
    };

    const response = await request.put(`${baseUrl}/api/health/config`, {
      data: newConfig,
      headers: { 'Content-Type': 'application/json' }
    });
    expect(response.status()).toBe(200);

    const getResponse = await request.get(`${baseUrl}/api/health/config`);
    const updatedConfig = await getResponse.json();
    expect(updatedConfig.check_interval_seconds).toBe(60);
  });

  test('should check dependencies status in enhanced health', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/enhanced`);
    const body = await response.json();
    const dependencies = body.dependencies;

    expect(dependencies).toHaveProperty('database');
    expect(dependencies).toHaveProperty('redis');

    expect(['ok', 'error', 'warn']).toContain(dependencies.database.status);
    expect(['ok', 'error', 'warn']).toContain(dependencies.redis.status);

    if (dependencies.database.response_time_ms) {
      expect(typeof dependencies.database.response_time_ms).toBe('number');
    }
  });

  test('should include alerts when thresholds exceeded', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/enhanced`);
    const body = await response.json();

    if (body.alerts && body.alerts.length > 0) {
      body.alerts.forEach((alert: any) => {
        expect(alert).toHaveProperty('id');
        expect(alert).toHaveProperty('severity');
        expect(alert).toHaveProperty('message');
        expect(alert).toHaveProperty('triggered_at');
        expect(['warning', 'critical']).toContain(alert.severity);
      });
    }
  });

  test('should include detailed checks in enhanced health', async ({ request }) => {
    const response = await request.get(`${baseUrl}/api/health/enhanced`);
    const body = await response.json();
    const checks = body.checks;

    expect(checks).toHaveProperty('cpu');
    expect(checks).toHaveProperty('memory');
    expect(checks).toHaveProperty('disk');

    const cpuCheck = checks.cpu;
    expect(cpuCheck).toHaveProperty('name', 'cpu');
    expect(cpuCheck).toHaveProperty('status');
    expect(cpuCheck).toHaveProperty('current_value');
    expect(cpuCheck).toHaveProperty('thresholds');
    expect(cpuCheck.thresholds).toHaveProperty('warning');
    expect(cpuCheck.thresholds).toHaveProperty('critical');
  });
});

test.describe('Metrics Endpoint', () => {
  const baseUrl = 'http://localhost:8080';

  test('should return Prometheus metrics format', async ({ request }) => {
    const response = await request.get(`${baseUrl}/metrics`);
    expect(response.status()).toBe(200);
    expect(response.headers()['content-type']).toContain('text/plain');

    const text = await response.text();
    expect(text).toContain('# HELP');
    expect(text).toContain('# TYPE');
  });
});
