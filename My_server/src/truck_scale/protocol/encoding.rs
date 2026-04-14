//! / 字符编码模块(GB2312)
use encoding_rs::Encoding;

/// 将 GB2312 编码的字符串转换为 UTF-8
pub fn gb2312_to_utf8(data: &[u8]) -> String {
    let encoding = Encoding::for_label(b"GB2312").unwrap();
    let (cow, _, _) = encoding.decode(data);
    cow.to_string()
}

/// 将 UTF-8 字符串转换为 GB2312 编码
pub fn utf8_to_gb2312(s: &str) -> Vec<u8> {
    let encoding = Encoding::for_label(b"GB2312").unwrap();
    let (cow, _, _) = encoding.encode(s);
    cow.to_vec()
}

/// 检测并转换编码
pub fn auto_convert_to_utf8(data: &[u8]) -> String {
    // 尝试 GB2312 解码
    let encoding = Encoding::for_label(b"GB2312").unwrap();
    let (cow, _, _) = encoding.decode(data);
    let result = cow.to_string();

    // 如果结果包含替换字符,可能是 UTF-8
    if result.contains('\u{FFFD}') {
        String::from_utf8_lossy(data).to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gb2312_to_utf8() {
        // GB2312 编码的 "测试"
        let gb2312_bytes = [0xB2, 0xE2, 0xCA, 0xD4];
        let utf8_str = gb2312_to_utf8(&gb2312_bytes);
        assert_eq!(utf8_str, "测试");
    }

    #[test]
    fn test_utf8_to_gb2312() {
        let utf8_str = "测试";
        let gb2312_bytes = utf8_to_gb2312(utf8_str);
        let back_to_utf8 = gb2312_to_utf8(&gb2312_bytes);
        assert_eq!(utf8_str, back_to_utf8);
    }
}
