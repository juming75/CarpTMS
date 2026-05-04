/**
 * 数据脱敏工具
 * 用于保护敏感个人信息
 */

/**
 * 手机号脱敏
 * 保留前3位和后4位，中间4位用*掩码
 * 例如: 13812345678 -> 138****5678
 */
export function maskPhoneNumber(phone: string | null | undefined): string {
  if (!phone) return '';
  const cleaned = phone.replace(/\D/g, '');
  if (cleaned.length !== 11) return phone;
  return `${cleaned.slice(0, 3)}****${cleaned.slice(-4)}`;
}

/**
 * 身份证号脱敏
 * 保留前6位和后4位，中间8位用*掩码
 * 例如: 110101199001011234 -> 110101********1234
 */
export function maskIdCard(idCard: string | null | undefined): string {
  if (!idCard) return '';
  const cleaned = idCard.replace(/\D/g, '');
  if (cleaned.length !== 18) return idCard;
  return `${cleaned.slice(0, 6)}********${cleaned.slice(-4)}`;
}

/**
 * 银行卡号脱敏
 * 保留前6位和后4位，中间用*掩码
 * 例如: 6222021234567890123 -> 622202**********0123
 */
export function maskBankCard(bankCard: string | null | undefined): string {
  if (!bankCard) return '';
  const cleaned = bankCard.replace(/\D/g, '');
  if (cleaned.length < 12) return bankCard;
  const maskedLength = cleaned.length - 10;
  const masked = '*'.repeat(maskedLength);
  return `${cleaned.slice(0, 6)}${masked}${cleaned.slice(-4)}`;
}

/**
 * 邮箱脱敏
 * 保留@前的第1位和最后一位，中间用*掩码
 * 例如: zhangsan@example.com -> z****n@example.com
 */
export function maskEmail(email: string | null | undefined): string {
  if (!email) return '';
  const parts = email.split('@');
  if (parts.length !== 2) return email;

  const localPart = parts[0];
  const domain = parts[1];

  if (localPart.length <= 2) {
    return `${'*'.repeat(localPart.length)}@${domain}`;
  }

  return `${localPart[0]}${'*'.repeat(localPart.length - 2)}${localPart.slice(-1)}@${domain}`;
}

/**
 * 姓名脱敏
 * 保留姓氏，最后一个字符用*掩码（如果长度>2）
 * 例如: 张三 -> 张*；欧阳锋 -> 欧*锋
 */
export function maskName(name: string | null | undefined): string {
  if (!name) return '';
  if (name.length === 1) return '*';
  if (name.length === 2) return `${name[0]}*`;

  // 复姓处理
  const compoundSurnames = ['欧阳', '司马', '上官', '诸葛', '慕容', '令狐', '公孙', '西门', '南宫', '东方', '夏侯', '皇甫', '尉迟', '呼延', '赫连', '澹台', '长孙', '宇文', '司徒', '司空'];
  for (const surname of compoundSurnames) {
    if (name.startsWith(surname)) {
      if (name.length === surname.length + 1) {
        return `${surname[0]}*`;
      }
      return `${surname[0]}${'*'.repeat(surname.length - 1)}${name.slice(-1)}`;
    }
  }

  return `${name[0]}${'*'.repeat(name.length - 2)}${name.slice(-1)}`;
}

/**
 * 地址脱敏
 * 保留省市区详细信息，隐藏具体门牌号
 * 例如: 北京市朝阳区建国路88号1号楼201室 -> 北京市朝阳区建国路*号
 */
export function maskAddress(address: string | null | undefined): string {
  if (!address) return '';

  // 匹配数字+号牌或栋/楼/单元等
  const pattern = /\d+(号|栋|号楼|单元|室|弄|巷|条|楼|#|-)/gi;
  return address.replace(pattern, (match) => {
    const numPart = match.match(/\d+/)?.[0] || '';
    return numPart ? '*' : '';
  });
}

/**
 * 车牌号脱敏
 * 保留第1位和最后一位
 * 例如: 京A12345 -> 京*2345
 */
export function maskLicensePlate(plate: string | null | undefined): string {
  if (!plate) return '';
  if (plate.length <= 2) return '*'.repeat(plate.length);
  return `${plate[0]}${'*'.repeat(plate.length - 2)}${plate.slice(-1)}`;
}

/**
 * 密码脱敏
 * 完全掩码
 */
export function maskPassword(password: string | null | undefined): string {
  if (!password) return '';
  return '*'.repeat(Math.min(password.length, 12));
}

/**
 * 通用脱敏函数
 * 根据字段类型自动选择脱敏方式
 */
export function maskSensitiveData(value: string | null | undefined, fieldType: 'phone' | 'idCard' | 'bankCard' | 'email' | 'name' | 'address' | 'licensePlate' | 'password' | 'custom'): string {
  switch (fieldType) {
    case 'phone':
      return maskPhoneNumber(value);
    case 'idCard':
      return maskIdCard(value);
    case 'bankCard':
      return maskBankCard(value);
    case 'email':
      return maskEmail(value);
    case 'name':
      return maskName(value);
    case 'address':
      return maskAddress(value);
    case 'licensePlate':
      return maskLicensePlate(value);
    case 'password':
      return maskPassword(value);
    case 'custom':
    default:
      return value ? '***' : '';
  }
}

/**
 * Vue 过滤器：用于模板中快速脱敏
 */
export const dataMaskFilters = {
  phone: maskPhoneNumber,
  idCard: maskIdCard,
  bankCard: maskBankCard,
  email: maskEmail,
  name: "maskName",
  address: maskAddress,
  licensePlate: maskLicensePlate,
  password: maskPassword,
};

/**
 * 批量脱敏对象中的敏感字段
 */
export function maskObjectFields<T extends Record<string, any>>(
  obj: T,
  fields: Array<{ field: keyof T; type: 'phone' | 'idCard' | 'bankCard' | 'email' | 'name' | 'address' | 'licensePlate' | 'password' | 'custom' }>
): T {
  const result = { ...obj };
  for (const { field, type } of fields) {
    if (result[field] !== null && result[field] !== undefined) {
      (result as any)[field] = maskSensitiveData(String(result[field]), type);
    }
  }
  return result;
}

/**
 * 车辆信息脱敏
 */
export function maskVehicleInfo(vehicle: any): any {
  return {
    ...vehicle,
    owner_name: vehicle.owner_name ? maskName(vehicle.owner_name) : null,
    owner_phone: vehicle.owner_phone ? maskPhoneNumber(vehicle.owner_phone) : null,
    owner_id_card: vehicle.owner_id_card ? maskIdCard(vehicle.owner_id_card) : null,
    license_plate: vehicle.license_plate ? maskLicensePlate(vehicle.license_plate) : null,
  };
}

/**
 * 司机信息脱敏
 */
export function maskDriverInfo(driver: any): any {
  return {
    ...driver,
    name: driver.name ? maskName(driver.name) : null,
    phone: driver.phone ? maskPhoneNumber(driver.phone) : null,
    id_card: driver.id_card ? maskIdCard(driver.id_card) : null,
    emergency_contact_phone: driver.emergency_contact_phone ? maskPhoneNumber(driver.emergency_contact_phone) : null,
    emergency_contact_name: driver.emergency_contact_name ? maskName(driver.emergency_contact_name) : null,
  };
}

/**
 * 用户信息脱敏
 */
export function maskUserInfo(user: any): any {
  return {
    ...user,
    username: user.username ? user.username[0] + '***' + user.username.slice(-1) : null,
    phone: user.phone ? maskPhoneNumber(user.phone) : null,
    email: user.email ? maskEmail(user.email) : null,
    id_card: user.id_card ? maskIdCard(user.id_card) : null,
    real_name: user.real_name ? maskName(user.real_name) : null,
  };
}
