//! API端点性能基准测试
//! 使用 Criterion 对关键API端点进行压力测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// 注意：此基准测试需要先启动后端服务器
// 运行方式: cargo bench --bench api_benchmark

fn benchmark_health_check(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("health_check_request", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8082/api/health").send().await;
            black_box(resp)
        })
    });
}

fn benchmark_api_response_time(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("api_response_times");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("login_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let resp = client
                .post("http://localhost:8082/api/auth/login")
                .json(&serde_json::json!({
                    "username": "test_user",
                    "password": "test_pass"
                }))
                .send()
                .await;
            black_box(resp)
        })
    });

    group.finish();
}

fn benchmark_vehicle_query(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("vehicle_list_query", |b| {
        b.to_async(&rt).iter(|| async {
            let client = reqwest::Client::new();
            let resp = client
                .get("http://localhost:8082/api/vehicles?page=1&page_size=20")
                .header("Authorization", "Bearer test_token")
                .send()
                .await;
            black_box(resp)
        })
    });
}

criterion_group!(
    name = api_benchmarks;
    config = Criterion::default()
        .significance_level(0.05)
        .sample_size(100);
    targets = benchmark_health_check, benchmark_api_response_time, benchmark_vehicle_query
);
criterion_main!(api_benchmarks);
