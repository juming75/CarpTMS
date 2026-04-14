//! 性能基准测试
//!
//! 使用criterion库进行性能基准测试,建立性能基线

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// 导入序列化工具
use tms_server::utils::serialization::{
    binary_deserialize, binary_serialize, json_deserialize, json_serialize,
};

// 测试数据结构
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestVehicle {
    id: i32,
    plate_number: String,
    model: String,
    brand: String,
    status: i32,
    longitude: f64,
    latitude: f64,
    speed: f64,
    direction: f64,
    last_updated: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestUser {
    id: i32,
    username: String,
    email: Option<String>,
    phone: Option<String>,
    group_id: i32,
    created_at: String,
    last_login: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestWeighingData {
    id: i32,
    vehicle_id: i32,
    plate_number: String,
    weighing_time: String,
    gross_weight: f64,
    tare_weight: f64,
    net_weight: f64,
    weighing_station: String,
    operator: String,
    status: i32,
}

// 生成测试数据
fn generate_test_vehicle(id: i32) -> TestVehicle {
    TestVehicle {
        id,
        plate_number: format!("粤A{:05}", id),
        model: "解放J6P".to_string(),
        brand: "解放".to_string(),
        status: 1,
        longitude: 113.264434,
        latitude: 23.129111,
        speed: 60.5,
        direction: 45.0,
        last_updated: "2024-01-01T12:00:00".to_string(),
    }
}

fn generate_test_user(id: i32) -> TestUser {
    TestUser {
        id,
        username: format!("user_{}", id),
        email: Some(format!("user{}@example.com", id)),
        phone: Some(format!("1380013800{}", id % 10)),
        group_id: 1,
        created_at: "2024-01-01T00:00:00".to_string(),
        last_login: Some("2024-01-01T12:00:00".to_string()),
    }
}

fn generate_test_weighing_data(id: i32) -> TestWeighingData {
    TestWeighingData {
        id,
        vehicle_id: id,
        plate_number: format!("粤A{:05}", id),
        weighing_time: "2024-01-01T10:00:00".to_string(),
        gross_weight: 45.5,
        tare_weight: 15.5,
        net_weight: 30.0,
        weighing_station: "广州称重站".to_string(),
        operator: "操作员1".to_string(),
        status: 1,
    }
}

// 序列化性能测试
fn serialization_benchmark(c: &mut Criterion) {
    let vehicle = generate_test_vehicle(1);
    let user = generate_test_user(1);
    let weighing_data = generate_test_weighing_data(1);

    let mut group = c.benchmark_group("serialization");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(5));

    // JSON序列化测试
    group.bench_function("json_serialize_vehicle", |b| {
        b.iter(|| black_box(json_serialize(black_box(&vehicle)).unwrap()))
    });

    group.bench_function("json_serialize_user", |b| {
        b.iter(|| black_box(json_serialize(black_box(&user)).unwrap()))
    });

    group.bench_function("json_serialize_weighing_data", |b| {
        b.iter(|| black_box(json_serialize(black_box(&weighing_data)).unwrap()))
    });

    // 二进制序列化测试
    group.bench_function("binary_serialize_vehicle", |b| {
        b.iter(|| black_box(binary_serialize(black_box(&vehicle)).unwrap()))
    });

    group.bench_function("binary_serialize_user", |b| {
        b.iter(|| black_box(binary_serialize(black_box(&user)).unwrap()))
    });

    group.bench_function("binary_serialize_weighing_data", |b| {
        b.iter(|| black_box(binary_serialize(black_box(&weighing_data)).unwrap()))
    });

    group.finish();
}

// 反序列化性能测试
fn deserialization_benchmark(c: &mut Criterion) {
    let vehicle = generate_test_vehicle(1);
    let user = generate_test_user(1);
    let weighing_data = generate_test_weighing_data(1);

    // 预先序列化数据
    let vehicle_json = json_serialize(&vehicle).unwrap();
    let user_json = json_serialize(&user).unwrap();
    let weighing_data_json = json_serialize(&weighing_data).unwrap();

    let vehicle_binary = binary_serialize(&vehicle).unwrap();
    let user_binary = binary_serialize(&user).unwrap();
    let weighing_data_binary = binary_serialize(&weighing_data).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(5));

    // JSON反序列化测试
    group.bench_function("json_deserialize_vehicle", |b| {
        b.iter(|| black_box(json_deserialize::<TestVehicle>(black_box(&vehicle_json)).unwrap()))
    });

    group.bench_function("json_deserialize_user", |b| {
        b.iter(|| black_box(json_deserialize::<TestUser>(black_box(&user_json)).unwrap()))
    });

    group.bench_function("json_deserialize_weighing_data", |b| {
        b.iter(|| {
            black_box(json_deserialize::<TestWeighingData>(black_box(&weighing_data_json)).unwrap())
        })
    });

    // 二进制反序列化测试
    group.bench_function("binary_deserialize_vehicle", |b| {
        b.iter(|| black_box(binary_deserialize::<TestVehicle>(black_box(&vehicle_binary)).unwrap()))
    });

    group.bench_function("binary_deserialize_user", |b| {
        b.iter(|| black_box(binary_deserialize::<TestUser>(black_box(&user_binary)).unwrap()))
    });

    group.bench_function("binary_deserialize_weighing_data", |b| {
        b.iter(|| {
            black_box(
                binary_deserialize::<TestWeighingData>(black_box(&weighing_data_binary)).unwrap(),
            )
        })
    });

    group.finish();
}

// 批量数据处理性能测试
fn batch_processing_benchmark(c: &mut Criterion) {
    let vehicles: Vec<TestVehicle> = (1..100).map(generate_test_vehicle).collect();
    let users: Vec<TestUser> = (1..100).map(generate_test_user).collect();

    let mut group = c.benchmark_group("batch_processing");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(5));

    // 批量JSON序列化测试
    group.bench_function("batch_json_serialize_100_vehicles", |b| {
        b.iter(|| {
            black_box(
                vehicles
                    .iter()
                    .map(|v| json_serialize(v).unwrap())
                    .collect::<Vec<_>>(),
            )
        })
    });

    group.bench_function("batch_json_serialize_100_users", |b| {
        b.iter(|| {
            black_box(
                users
                    .iter()
                    .map(|u| json_serialize(u).unwrap())
                    .collect::<Vec<_>>(),
            )
        })
    });

    group.finish();
}

// 基准测试分组
criterion_group!(
    benches,
    serialization_benchmark,
    deserialization_benchmark,
    batch_processing_benchmark
);
criterion_main!(benches);
