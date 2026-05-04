import { describe, it, expect } from 'vitest';
import {
  maskPhoneNumber,
  maskIdCard,
  maskBankCard,
  maskEmail,
  maskName,
  maskAddress,
  maskLicensePlate,
  maskPassword,
  maskSensitiveData,
  maskObjectFields,
  maskVehicleInfo,
  maskDriverInfo,
  maskUserInfo,
} from '../utils/dataMask';

describe('dataMask', () => {
  describe('maskPhoneNumber', () => {
    it('should mask phone number correctly', () => {
      expect(maskPhoneNumber('13812345678')).toBe('138****5678');
    });

    it('should handle phone numbers with special characters', () => {
      expect(maskPhoneNumber('138-1234-5678')).toBe('138****5678');
    });

    it('should return original for invalid phone numbers', () => {
      expect(maskPhoneNumber('12345')).toBe('12345');
      expect(maskPhoneNumber(null)).toBe('');
      expect(maskPhoneNumber(undefined)).toBe('');
    });
  });

  describe('maskIdCard', () => {
    it('should mask ID card correctly', () => {
      expect(maskIdCard('110101199001011234')).toBe('110101********1234');
    });

    it('should return original for invalid ID cards', () => {
      expect(maskIdCard('12345')).toBe('12345');
      expect(maskIdCard(null)).toBe('');
    });
  });

  describe('maskBankCard', () => {
    it('should mask bank card correctly', () => {
      const card = '6222021234567890123';
      const result = maskBankCard(card);
      expect(result.startsWith('622202')).toBe(true);
      expect(result.endsWith('0123')).toBe(true);
      expect(result.length).toBe(card.length);
    });

    it('should return original for short card numbers', () => {
      expect(maskBankCard('123456789012')).toBe('123456789012');
    });

    it('should return empty for null input', () => {
      expect(maskBankCard(null)).toBe('');
    });
  });

  describe('maskEmail', () => {
    it('should mask email correctly', () => {
      expect(maskEmail('zhangsan@example.com')).toBe('z****n@example.com');
    });

    it('should handle short local part', () => {
      expect(maskEmail('ab@example.com')).toBe('**@example.com');
    });

    it('should return original for invalid email', () => {
      expect(maskEmail('invalid-email')).toBe('invalid-email');
    });
  });

  describe('maskName', () => {
    it('should mask single character name', () => {
      expect(maskName('张')).toBe('*');
    });

    it('should mask two character name', () => {
      expect(maskName('张三')).toBe('张*');
    });

    it('should mask compound surname correctly', () => {
      expect(maskName('欧阳锋')).toBe('欧**锋');
    });

    it('should mask regular surname correctly', () => {
      expect(maskName('司马迁')).toBe('司**迁');
    });

    it('should return empty for null input', () => {
      expect(maskName(null)).toBe('');
    });
  });

  describe('maskAddress', () => {
    it('should mask house number in address', () => {
      const address = '北京市朝阳区建国路88号1号楼201室';
      const result = maskAddress(address);
      expect(result).toContain('*');
    });

    it('should return original for null input', () => {
      expect(maskAddress(null)).toBe('');
    });
  });

  describe('maskLicensePlate', () => {
    it('should mask license plate correctly', () => {
      expect(maskLicensePlate('京A12345')).toBe('京*2345');
    });

    it('should handle short plate', () => {
      expect(maskLicensePlate('京A')).toBe('**');
    });
  });

  describe('maskPassword', () => {
    it('should mask password with asterisks', () => {
      const result = maskPassword('mySecretPassword123');
      expect(result).toBe('************');
      expect(result.length).toBeLessThanOrEqual(12);
    });

    it('should handle empty password', () => {
      expect(maskPassword('')).toBe('');
    });
  });

  describe('maskSensitiveData', () => {
    it('should mask phone correctly based on type', () => {
      expect(maskSensitiveData('13812345678', 'phone')).toBe('138****5678');
    });

    it('should mask ID card correctly based on type', () => {
      expect(maskSensitiveData('110101199001011234', 'idCard')).toBe('110101********1234');
    });

    it('should mask bank card correctly based on type', () => {
      const result = maskSensitiveData('6222021234567890123', 'bankCard');
      expect(result.startsWith('622202')).toBe(true);
      expect(result.endsWith('0123')).toBe(true);
    });

    it('should mask custom type with asterisks', () => {
      expect(maskSensitiveData('sensitive-data', 'custom')).toBe('***');
    });
  });

  describe('maskVehicleInfo', () => {
    it('should mask vehicle owner information', () => {
      const vehicle = {
        vehicle_id: 1,
        license_plate: '京A12345',
        owner_name: '张三',
        owner_phone: '13812345678',
        owner_id_card: '110101199001011234',
      };

      const masked = maskVehicleInfo(vehicle);

      expect(masked.owner_name).toBe('张*');
      expect(masked.owner_phone).toBe('138****5678');
      expect(masked.owner_id_card).toBe('110101********1234');
      expect(masked.license_plate).toBe('京*2345');
    });

    it('should handle null values', () => {
      const vehicle = {
        vehicle_id: 1,
        license_plate: null,
        owner_name: null,
        owner_phone: null,
        owner_id_card: null,
      };

      const masked = maskVehicleInfo(vehicle);
      expect(masked.owner_name).toBeNull();
      expect(masked.owner_phone).toBeNull();
    });
  });

  describe('maskDriverInfo', () => {
    it('should mask driver sensitive information', () => {
      const driver = {
        driver_id: 1,
        name: '李四',
        phone: '13987654321',
        id_card: '110101199001011234',
        emergency_contact_phone: '13811112222',
        emergency_contact_name: '王五',
      };

      const masked = maskDriverInfo(driver);

      expect(masked.name).toBe('李*');
      expect(masked.phone).toBe('139****4321');
      expect(masked.id_card).toBe('110101********1234');
      expect(masked.emergency_contact_phone).toBe('138****2222');
      expect(masked.emergency_contact_name).toBe('王*');
    });
  });

  describe('maskUserInfo', () => {
    it('should mask user sensitive information', () => {
      const user = {
        user_id: 1,
        username: 'zhangsan',
        phone: '13812345678',
        email: 'zhangsan@example.com',
        id_card: '110101199001011234',
        real_name: '张三',
      };

      const masked = maskUserInfo(user);

      expect(masked.username).toBe('z***n');
      expect(masked.phone).toBe('138****5678');
      expect(masked.email).toBe('z****n@example.com');
      expect(masked.id_card).toBe('110101********1234');
      expect(masked.real_name).toBe('张*');
    });
  });

  describe('maskObjectFields', () => {
    it('should mask specified fields', () => {
      const obj = {
        name: '张三',
        phone: '13812345678',
        email: 'test@example.com',
      };

      const masked = maskObjectFields(obj, [
        { field: 'name', type: 'name' },
        { field: 'phone', type: 'phone' },
        { field: 'email', type: 'email' },
      ]);

      expect(masked.name).toBe('张*');
      expect(masked.phone).toBe('138****5678');
      expect(masked.email).toBe('t***t@example.com');
    });
  });
});
