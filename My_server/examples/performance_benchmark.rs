use std::sync::Arc;
/// 性能基准测试示例
///
/// 运行方式:
/// ```bash
/// cargo run --example performance_benchmark --release
/// ```
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==========================================");
    println!("CarpTMS 性能基准测试");
    println!("==========================================\n");

    // 1. 消息序列化性能测试
    test_message_serialization().await?;

    // 2. 数据转换性能测试
    test_data_transformation().await?;

    // 3. 批量查询性能测试
    test_batch_query().await?;

    // 4. 并发处理性能测试
    test_concurrent_processing().await?;

    println!("\n==========================================");
    println!("所有性能测试完成!");
    println!("==========================================");

    Ok(())
}

/// 消息序列化性能测试
async fn test_message_serialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("[1/4] 消息序列化性能测试");

    let iterations = 10000;
    let test_data: Vec<String> = (0..iterations)
        .map(|i| {
            format!(
                r#"{{"type":"login","username":"user_{}","password":"pass_{}","version":"1.0.0"}}"#,
                i, i
            )
        })
        .collect();

    let start = Instant::now();
    let mut serialized_count = 0;
    for json in &test_data {
        if let Ok(_) = serde_json::from_str::<serde_json::Value>(json) {
            serialized_count += 1;
        }
    }
    let duration = start.elapsed();

    println!("  序列化{}条数据耗时: {:?}", serialized_count, duration);
    println!(
        "  平均耗时: {:.2} μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  吞吐量: {:.2} ops/sec",
        iterations as f64 / duration.as_secs_f64()
    );

    let target_ms = 500.0;
    let actual_ms = duration.as_millis() as f64;
    if actual_ms < target_ms {
        println!(
            "  ✓ 性能达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    } else {
        println!(
            "  ✗ 性能未达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    }

    println!();
    Ok(())
}

/// 数据转换性能测试
async fn test_data_transformation() -> Result<(), Box<dyn std::error::Error>> {
    println!("[2/4] 数据转换性能测试");

    let iterations = 10000;

    #[derive(Clone)]
    struct Vehicle {
        vehicle_id: String,
        plate_no: String,
        status: i32,
    }

    let vehicles: Vec<Vehicle> = (0..iterations)
        .map(|i| Vehicle {
            vehicle_id: format!("v{}", i),
            plate_no: format!("粤A{:05}", i),
            status: i % 2,
        })
        .collect();

    let start = Instant::now();
    let _transformed: Vec<String> = vehicles
        .iter()
        .map(|v| format!("{}:{}:{}", v.vehicle_id, v.plate_no, v.status))
        .collect();
    let duration = start.elapsed();

    println!("  转换{}条数据耗时: {:?}", iterations, duration);
    println!(
        "  平均耗时: {:.2} μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  吞吐量: {:.2} ops/sec",
        iterations as f64 / duration.as_secs_f64()
    );

    let target_ms = 100.0;
    let actual_ms = duration.as_millis() as f64;
    if actual_ms < target_ms {
        println!(
            "  ✓ 性能达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    } else {
        println!(
            "  ✗ 性能未达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    }

    println!();
    Ok(())
}

/// 批量查询性能测试
async fn test_batch_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("[3/4] 批量查询性能测试");

    let iterations = 1000;
    let ids: Vec<String> = (0..iterations).map(|i| format!("id_{}", i)).collect();

    // 模拟批量查询
    let start = Instant::now();
    let mut results = Vec::new();
    for chunk in ids.chunks(100) {
        // 模拟查询延迟
        tokio::time::sleep(Duration::from_micros(10)).await;
        results.extend(chunk.iter().cloned());
    }
    let duration = start.elapsed();

    println!("  批量查询{}条数据耗时: {:?}", iterations, duration);
    println!(
        "  平均耗时: {:.2} μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  吞吐量: {:.2} ops/sec",
        iterations as f64 / duration.as_secs_f64()
    );

    let target_ms = 100.0;
    let actual_ms = duration.as_millis() as f64;
    if actual_ms < target_ms {
        println!(
            "  ✓ 性能达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    } else {
        println!(
            "  ✗ 性能未达标 (目标: < {}ms, 实际: {}ms)",
            target_ms, actual_ms
        );
    }

    println!();
    Ok(())
}

/// 并发处理性能测试
async fn test_concurrent_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("[4/4] 并发处理性能测试");

    let iterations = 1000;
    let concurrent_tasks = 10;
    let semaphore = Arc::new(Semaphore::new(concurrent_tasks));

    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..iterations {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await;
            // 模拟任务处理
            tokio::time::sleep(Duration::from_micros(100)).await;
            i
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    let duration = start.elapsed();

    println!("  并发处理{}个任务耗时: {:?}", iterations, duration);
    println!(
        "  平均耗时: {:.2} μs",
        duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  吞吐量: {:.2} ops/sec",
        iterations as f64 / duration.as_secs_f64()
    );

    let target_sec = 5.0;
    let actual_sec = duration.as_secs_f64();
    if actual_sec < target_sec {
        println!(
            "  ✓ 性能达标 (目标: < {}s, 实际: {:.2}s)",
            target_sec, actual_sec
        );
    } else {
        println!(
            "  ✗ 性能未达标 (目标: < {}s, 实际: {:.2}s)",
            target_sec, actual_sec
        );
    }

    println!();
    Ok(())
}
