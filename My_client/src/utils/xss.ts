import DOMPurify from 'dompurify';

/**
 * XSS防护工具类
 * 使用DOMPurify库来净化用户输入，防止XSS攻击
 */
export class XssProtection {
  /**
   * 净化HTML内容
   * @param html 原始HTML内容
   * @returns 净化后的HTML内容
   */
  static sanitizeHtml(html: string): string {
    return DOMPurify.sanitize(html);
  }

  /**
   * 净化文本内容
   * @param text 原始文本内容
   * @returns 净化后的文本内容
   */
  static sanitizeText(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  /**
   * 净化URL
   * @param url 原始URL
   * @returns 净化后的URL
   */
  static sanitizeUrl(url: string): string {
    try {
      const parsedUrl = new URL(url);
      return parsedUrl.toString();
    } catch {
      return '';
    }
  }

  /**
   * 净化表单输入
   * @param input 表单输入对象
   * @returns 净化后的表单输入对象
   */
  static sanitizeFormInput<T extends Record<string, any>>(input: T): T {
    const sanitized: Record<string, any> = {};
    
    for (const [key, value] of Object.entries(input)) {
      if (typeof value === 'string') {
        sanitized[key] = this.sanitizeText(value);
      } else if (typeof value === 'object' && value !== null) {
        sanitized[key] = this.sanitizeFormInput(value);
      } else {
        sanitized[key] = value;
      }
    }
    
    return sanitized as T;
  }

  /**
   * 验证输入是否包含XSS攻击向量
   * @param input 输入内容
   * @returns 是否包含XSS攻击向量
   */
  static containsXss(input: string): boolean {
    const sanitized = DOMPurify.sanitize(input, { ALLOWED_TAGS: [] });
    return sanitized !== input;
  }
}
