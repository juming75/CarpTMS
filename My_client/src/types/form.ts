// 表单相关类型定义

// 表单验证规则类型
export interface FormRule {
  required?: boolean;
  message?: string;
  trigger?: string | string[];
  validator?: (rule: FormRule, value: unknown, callback: (error?: Error) => void) => void;
  min?: number;
  max?: number;
  pattern?: RegExp;
  [key: string]: unknown;
}

// 登录表单数据
export interface LoginForm {
  username: string;
  password: string;
  rememberPassword: boolean;
  autoLogin: boolean;
}

// 服务器配置
export interface ServerConfig {
  ip: string;
  port: string;
}

// 配置表单数据
export interface ConfigForm {
  ip: string;
  port: string;
}

// 登录规则
export interface LoginRules {
  username: FormRule[];
  password: FormRule[];
}

// 登录响应数据
export interface LoginResponse {
  code: number;
  message: string;
  data: {
    access_token: string;
    refresh_token?: string;
    user: {
      user_id: number;
      username: string;
      name: string;
      role: string;
      [key: string]: unknown;
    };
    [key: string]: unknown;
  };
}
