//! 工具函数单元测试

#[cfg(test)]
mod tests {
    // 测试日期时间工具
    mod datetime_tests {
        use chrono::{DateTime, Utc, TimeZone};

        #[test]
        fn test_datetime_utc_now() {
            let now = Utc::now();
            assert!(now.timestamp() > 0);
        }

        #[test]
        fn test_datetime_from_timestamp() {
            let timestamp: i64 = 1700000000;
            let dt = DateTime::from_timestamp(timestamp, 0);
            assert!(dt.is_some());
            let dt = dt.unwrap();
            assert_eq!(dt.timestamp(), timestamp);
        }

        #[test]
        fn test_datetime_format() {
            let dt = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
            let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            assert_eq!(formatted, "2024-01-15 10:30:00");
        }

        #[test]
        fn test_datetime_parse() {
            let s = "2024-01-15T10:30:00Z";
            let dt = DateTime::parse_from_rfc3339(s);
            assert!(dt.is_ok());
        }
    }

    // 测试字符串工具
    mod string_tests {
        #[test]
        fn test_string_trim() {
            let s = "  hello  ";
            assert_eq!(s.trim(), "hello");
        }

        #[test]
        fn test_string_is_empty() {
            assert!("".is_empty());
            assert!(!"hello".is_empty());
        }

        #[test]
        fn test_string_to_uppercase() {
            assert_eq!("hello".to_uppercase(), "HELLO");
        }

        #[test]
        fn test_string_to_lowercase() {
            assert_eq!("HELLO".to_lowercase(), "hello");
        }
    }

    // 测试选项工具
    mod option_tests {
        #[test]
        fn test_option_some() {
            let value = Some(42);
            assert!(value.is_some());
            assert_eq!(value.unwrap(), 42);
        }

        #[test]
        fn test_option_none() {
            let value: Option<i32> = None;
            assert!(value.is_none());
        }

        #[test]
        fn test_option_or_else() {
            let none: Option<i32> = None;
            assert_eq!(none.or_else(|| Some(42)).unwrap(), 42);

            let some = Some(10);
            assert_eq!(some.or_else(|| Some(42)).unwrap(), 10);
        }

        #[test]
        fn test_option_map() {
            let value = Some(5);
            let doubled = value.map(|x| x * 2);
            assert_eq!(doubled.unwrap(), 10);

            let none: Option<i32> = None;
            assert!(none.map(|x| x * 2).is_none());
        }
    }

    // 测试结果工具
    mod result_tests {
        #[test]
        fn test_result_ok() {
            let value: Result<i32, &str> = Ok(42);
            assert!(value.is_ok());
            assert_eq!(value.unwrap(), 42);
        }

        #[test]
        fn test_result_err() {
            let value: Result<i32, &str> = Err("error");
            assert!(value.is_err());
            assert_eq!(value.unwrap_err(), "error");
        }

        #[test]
        fn test_result_map() {
            let value: Result<i32, &str> = Ok(5);
            let doubled = value.map(|x| x * 2);
            assert_eq!(doubled.unwrap(), 10);
        }

        #[test]
        fn test_result_unwrap_or() {
            let err: Result<i32, &str> = Err("error");
            assert_eq!(err.unwrap_or(42), 42);

            let ok: Result<i32, &str> = Ok(10);
            assert_eq!(ok.unwrap_or(42), 10);
        }
    }

    // 测试向量工具
    mod vec_tests {
        #[test]
        fn test_vec_new() {
            let v: Vec<i32> = Vec::new();
            assert!(v.is_empty());
        }

        #[test]
        fn test_vec_from_iter() {
            let v: Vec<i32> = (1..=5).collect();
            assert_eq!(v.len(), 5);
            assert_eq!(v, vec![1, 2, 3, 4, 5]);
        }

        #[test]
        fn test_vec_push() {
            let mut v = Vec::new();
            v.push(1);
            v.push(2);
            assert_eq!(v.len(), 2);
        }

        #[test]
        fn test_vec_iter() {
            let v = vec![1, 2, 3];
            let sum: i32 = v.iter().sum();
            assert_eq!(sum, 6);
        }

        #[test]
        fn test_vec_map() {
            let v = vec![1, 2, 3];
            let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
            assert_eq!(doubled, vec![2, 4, 6]);
        }
    }

    // 测试哈希映射工具
    mod hashmap_tests {
        use std::collections::HashMap;

        #[test]
        fn test_hashmap_insert() {
            let mut map = HashMap::new();
            map.insert("key", 42);
            assert_eq!(map.get("key"), Some(&42));
        }

        #[test]
        fn test_hashmap_contains() {
            let mut map = HashMap::new();
            map.insert("key", 42);
            assert!(map.contains_key("key"));
            assert!(!map.contains_key("missing"));
        }

        #[test]
        fn test_hashmap_iter() {
            let mut map = HashMap::new();
            map.insert("a", 1);
            map.insert("b", 2);
            let keys: Vec<&str> = map.keys().cloned().collect();
            assert!(keys.contains(&"a"));
            assert!(keys.contains(&"b"));
        }
    }
}
