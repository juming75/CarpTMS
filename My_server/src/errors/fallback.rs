use crate::errors::AppError;

// 优雅降级策略 trait
pub trait FallbackStrategy<T> {
    // 当主操作失败时执行的降级逻辑
    fn fallback(&self, error: &AppError) -> Option<T>;
}

// 默认降级策略(返回 None)
pub struct DefaultFallback;

impl<T> FallbackStrategy<T> for DefaultFallback {
    fn fallback(&self, _error: &AppError) -> Option<T> {
        None
    }
}

// 常量值降级策略
pub struct ConstantFallback<T>(pub T);

impl<T: Clone> FallbackStrategy<T> for ConstantFallback<T> {
    fn fallback(&self, _error: &AppError) -> Option<T> {
        Some(self.0.clone())
    }
}

// 函数式降级策略
pub struct FunctionFallback<T, F>(pub F)
where
    F: Fn(&AppError) -> Option<T>;

impl<T, F> FallbackStrategy<T> for FunctionFallback<T, F>
where
    F: Fn(&AppError) -> Option<T>,
{
    fn fallback(&self, error: &AppError) -> Option<T> {
        (self.0)(error)
    }
}

// 带降级的操作执行函数
pub async fn with_fallback<T, F, Fut, FB>(operation: F, fallback: &FB) -> Result<T, AppError>
where
    F: FnOnce() -> Fut,
    Fut: futures::Future<Output = Result<T, AppError>>,
    FB: FallbackStrategy<T>,
{
    match operation().await {
        Ok(result) => Ok(result),
        Err(error) => match fallback.fallback(&error) {
            Some(fallback_result) => Ok(fallback_result),
            None => Err(error),
        },
    }
}

// 示例:缓存降级策略
pub struct CacheFallback<T>(pub Option<T>);

impl<T: Clone> FallbackStrategy<T> for CacheFallback<T> {
    fn fallback(&self, _error: &AppError) -> Option<T> {
        self.0.clone()
    }
}

// 示例:服务降级策略
pub struct ServiceFallback<T>(pub T);

impl<T: Clone> FallbackStrategy<T> for ServiceFallback<T> {
    fn fallback(&self, error: &AppError) -> Option<T> {
        // 只有当错误是外部服务错误时才降级
        if error.error_type == crate::errors::ErrorType::ExternalService {
            Some(self.0.clone())
        } else {
            None
        }
    }
}
