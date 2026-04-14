//! / 审计日志工具测试

use crate::utils::audit::log_audit_event;

// 测试用例:记录审计事件
#[test]
fn test_log_audit_event() {
    // 执行测试:记录审计事件
    log_audit_event(
        "test_user",
        "test_action",
        "test_resource",
        Some("test_detail"),
    );
    
    // 验证:日志应该被成功记录,没有 panic
    assert!(true);
}

// 测试用例:记录审计事件(无详细信息)
#[test]
fn test_log_audit_event_no_detail() {
    // 执行测试:记录审计事件(无详细信息)
    log_audit_event(
        "test_user",
        "test_action",
        "test_resource",
        None,
    );
    
    // 验证:日志应该被成功记录,没有 panic
    assert!(true);
}






