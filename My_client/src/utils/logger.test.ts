import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import logger from './logger';

describe('Logger', () => {
  const originalConsole = {
    debug: console.debug,
    info: console.info,
    warn: console.warn,
    error: console.error,
    group: console.group,
    groupEnd: console.groupEnd,
    time: console.time,
    timeEnd: console.timeEnd,
    table: console.table,
  };

  beforeEach(() => {
    console.debug = vi.fn();
    console.info = vi.fn();
    console.warn = vi.fn();
    console.error = vi.fn();
    console.group = vi.fn();
    console.groupEnd = vi.fn();
    console.time = vi.fn();
    console.timeEnd = vi.fn();
    console.table = vi.fn();
  });

  afterEach(() => {
    console.debug = originalConsole.debug;
    console.info = originalConsole.info;
    console.warn = originalConsole.warn;
    console.error = originalConsole.error;
    console.group = originalConsole.group;
    console.groupEnd = originalConsole.groupEnd;
    console.time = originalConsole.time;
    console.timeEnd = originalConsole.timeEnd;
    console.table = originalConsole.table;
  });

  describe('debug', () => {
    it('should log debug messages when enabled', () => {
      logger.setEnabled(true);
      logger.setLevel('debug');
      logger.debug('test message');

      expect(console.debug).toHaveBeenCalled();
    });

    it('should not log debug messages when disabled', () => {
      logger.setEnabled(false);
      logger.debug('test message');

      expect(console.debug).not.toHaveBeenCalled();
    });
  });

  describe('info', () => {
    it('should log info messages when enabled', () => {
      logger.setEnabled(true);
      logger.setLevel('info');
      logger.info('test message');

      expect(console.info).toHaveBeenCalled();
    });
  });

  describe('warn', () => {
    it('should log warn messages when enabled', () => {
      logger.setEnabled(true);
      logger.setLevel('warn');
      logger.warn('test message');

      expect(console.warn).toHaveBeenCalled();
    });
  });

  describe('error', () => {
    it('should log error messages when enabled', () => {
      logger.setEnabled(true);
      logger.setLevel('error');
      logger.error('test message');

      expect(console.error).toHaveBeenCalled();
    });

    it('should log error messages even at error level', () => {
      logger.setEnabled(true);
      logger.setLevel('error');
      logger.error('test error');

      expect(console.error).toHaveBeenCalled();
    });
  });

  describe('setLevel', () => {
    it('should filter messages based on level', () => {
      logger.setEnabled(true);
      logger.setLevel('warn');

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      expect(console.debug).not.toHaveBeenCalled();
      expect(console.info).not.toHaveBeenCalled();
      expect(console.warn).toHaveBeenCalled();
      expect(console.error).toHaveBeenCalled();
    });
  });

  describe('setEnabled', () => {
    it('should disable all logging when set to false', () => {
      logger.setEnabled(false);
      logger.setLevel('debug');

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      expect(console.debug).not.toHaveBeenCalled();
      expect(console.info).not.toHaveBeenCalled();
      expect(console.warn).not.toHaveBeenCalled();
      expect(console.error).not.toHaveBeenCalled();
    });
  });

  describe('group/groupEnd', () => {
    it('should create and end console groups', () => {
      logger.setEnabled(true);
      logger.setLevel('debug');

      logger.group('test group');
      logger.groupEnd();

      expect(console.group).toHaveBeenCalledWith('test group');
      expect(console.groupEnd).toHaveBeenCalled();
    });
  });

  describe('time/timeEnd', () => {
    it('should track time', () => {
      logger.setEnabled(true);
      logger.setLevel('debug');

      logger.time('test timer');
      logger.timeEnd('test timer');

      expect(console.time).toHaveBeenCalledWith('test timer');
      expect(console.timeEnd).toHaveBeenCalledWith('test timer');
    });
  });

  describe('table', () => {
    it('should log table data', () => {
      logger.setEnabled(true);
      logger.setLevel('debug');

      const data = [{ id: 1, name: 'test' }];
      logger.table(data);

      expect(console.table).toHaveBeenCalledWith(data);
    });
  });
});
