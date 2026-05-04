//! Performance Optimization Module
//!
//! Provides comprehensive performance optimization capabilities including:
//! - Query monitoring and optimization
//! - Enhanced system monitoring
//! - Protocol parsing optimization
//! - Memory management optimization

pub mod enhanced_memory_monitor;
pub mod enhanced_monitor;
pub mod memory_monitor;
pub mod memory_optimization;
pub mod protocol_optimization;
pub mod query_monitor;
pub mod resource_alert;
// pub mod thread_pool_manager;

pub use query_monitor::{
    monitor_query, QueryMetrics, QueryMonitor, QueryTimer, GLOBAL_QUERY_MONITOR,
};

pub use enhanced_monitor::{
    Alert, AlertLevel, ApplicationMetrics, BusinessMetrics, CacheMetrics, DatabaseMetrics,
    EnhancedPerformanceMonitor, PerformanceMonitorConfig, PerformanceMonitorService,
    PerformanceReport, SystemMetrics,
};

pub use protocol_optimization::{
    AllocationStats, BufferPool, BufferPoolStats, FrameHeader, MemoryOptimizer,
    OptimizedJt808Parser, ParsedFrame, ProtocolFrameProcessor, ProtocolOptimizationError,
    SimdOperations, ZeroCopyParser,
};

pub use memory_optimization::{
    ArenaAllocation, ArenaAllocator, ArenaStats, MemoryError, MemoryManager, MemoryPool,
    MemoryStats, PoolStats, PooledObject, SharedData, TrackingAllocator, TrackingStats,
};

// pub use thread_pool_manager::{
//     HttpServerThreadPoolExt, ThreadPoolManager, ThreadPoolManagerConfig,
// };

pub use memory_monitor::{MemoryLimitService, MemoryMonitor, MemoryMonitorConfig};

pub use enhanced_memory_monitor::{
    EnhancedMemoryMonitor, EnhancedMemoryMonitorConfig, EnhancedMemoryMonitoringService,
    MemoryAlertLevel, MemoryUsageHistory,
};

pub use resource_alert::{ResourceAlertConfig, ResourceAlertService, ResourceUsage};
