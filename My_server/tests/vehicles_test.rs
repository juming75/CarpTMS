use actix_web::{http::StatusCode, web, App};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Executor, PgPool};
use std::sync::LazyLock;
use tokio::sync::Mutex;

// 静态互斥锁,确保测试按顺序执行
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// 测试数据库初始化
async fn init_test_db() -> PgPool {
    // 从环境变量获取测试数据库连接字符串,默认使用新创建的tms_test数据库
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/tms_test".to_string());

    let pool = PgPool::connect(&db_url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", db_url))?;

    // 创建测试表 - 先删除所有表和序列,确保干净的测试环境
    let _ = pool
        .execute("DROP TABLE IF EXISTS weighing_data CASCADE")
        .await;
    let _ = pool.execute("DROP TABLE IF EXISTS vehicles CASCADE").await;
    let _ = pool
        .execute("DROP TABLE IF EXISTS vehicle_groups CASCADE")
        .await;

    // 删除可能存在的序列
    let _ = pool
        .execute("DROP SEQUENCE IF EXISTS vehicle_groups_group_id_seq CASCADE")
        .await;
    let _ = pool
        .execute("DROP SEQUENCE IF EXISTS vehicles_vehicle_id_seq CASCADE")
        .await;
    let _ = pool
        .execute("DROP SEQUENCE IF EXISTS weighing_data_id_seq CASCADE")
        .await;

    // 重新创建所有表,不使用IF NOT EXISTS,确保表结构正确
    // 1. 创建车组表
    pool.execute(
        r#"
        CREATE TABLE vehicle_groups (
            group_id SERIAL PRIMARY KEY,
            group_name VARCHAR(100) NOT NULL,
            parent_id INTEGER REFERENCES vehicle_groups(group_id),
            description TEXT,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP
        )
    "#,
    )
    .await
    .unwrap();

    // 插入默认车组
    pool.execute("INSERT INTO vehicle_groups (group_name) VALUES ('Default Group')")
        .await
        .unwrap();

    // 2. 创建车辆表
    pool.execute(
        r#"
        CREATE TABLE vehicles (
            vehicle_id SERIAL PRIMARY KEY,
            vehicle_name VARCHAR(100) NOT NULL,
            license_plate VARCHAR(20) NOT NULL,
            vehicle_type VARCHAR(50) NOT NULL,
            vehicle_color VARCHAR(20) NOT NULL,
            vehicle_brand VARCHAR(50) NOT NULL,
            vehicle_model VARCHAR(50) NOT NULL,
            engine_no VARCHAR(50) NOT NULL,
            frame_no VARCHAR(50) NOT NULL,
            register_date TIMESTAMP NOT NULL,
            inspection_date TIMESTAMP NOT NULL,
            insurance_date TIMESTAMP NOT NULL,
            seating_capacity INTEGER NOT NULL,
            load_capacity DOUBLE PRECISION NOT NULL,
            vehicle_length DOUBLE PRECISION NOT NULL,
            vehicle_width DOUBLE PRECISION NOT NULL,
            vehicle_height DOUBLE PRECISION NOT NULL,
            device_id VARCHAR(50),
            terminal_type VARCHAR(50),
            communication_type VARCHAR(50),
            sim_card_no VARCHAR(20),
            install_date TIMESTAMP,
            install_address TEXT,
            install_technician VARCHAR(50),
            own_no VARCHAR(50),
            own_name VARCHAR(100),
            own_phone VARCHAR(20),
            own_id_card VARCHAR(20),
            own_address TEXT,
            own_email VARCHAR(100),
            group_id INTEGER NOT NULL REFERENCES vehicle_groups(group_id),
            operation_status INTEGER NOT NULL DEFAULT 1,
            operation_route TEXT,
            operation_area TEXT,
            operation_company VARCHAR(100),
            driver_name VARCHAR(100),
            driver_phone VARCHAR(20),
            driver_license_no VARCHAR(20),
            purchase_price DOUBLE PRECISION,
            annual_fee DOUBLE PRECISION,
            insurance_fee DOUBLE PRECISION,
            remark TEXT,
            status INTEGER NOT NULL DEFAULT 1,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP,
            create_user_id INTEGER NOT NULL DEFAULT 1,
            update_user_id INTEGER,
            is_simulation BOOLEAN NOT NULL DEFAULT FALSE,
            simulation_source VARCHAR(50)
        )
    "#,
    )
    .await
    .unwrap();

    // 3. 创建称重数据表
    pool.execute(
        r#"
        CREATE TABLE weighing_data (
            id SERIAL PRIMARY KEY,
            vehicle_id INTEGER NOT NULL REFERENCES vehicles(vehicle_id) ON DELETE CASCADE,
            device_id VARCHAR(50) NOT NULL,
            weighing_time TIMESTAMP NOT NULL,
            gross_weight DOUBLE PRECISION NOT NULL,
            tare_weight DOUBLE PRECISION,
            net_weight DOUBLE PRECISION NOT NULL,
            axle_count INTEGER,
            speed DOUBLE PRECISION,
            lane_no INTEGER,
            site_id INTEGER,
            status INTEGER NOT NULL DEFAULT 1,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP
        )
    "#,
    )
    .await
    .unwrap();

    pool
}

// 定义测试用的车辆创建请求结构体
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct TestVehicleCreate {
    vehicle_name: String,
    license_plate: String,
    vehicle_type: String,
    vehicle_color: String,
    vehicle_brand: String,
    vehicle_model: String,
    engine_no: String,
    frame_no: String,
    register_date: DateTime<Utc>,
    inspection_date: DateTime<Utc>,
    insurance_date: DateTime<Utc>,
    seating_capacity: i32,
    load_capacity: f64,
    vehicle_length: f64,
    vehicle_width: f64,
    vehicle_height: f64,
    device_id: Option<String>,
    terminal_type: Option<String>,
    communication_type: Option<String>,
    sim_card_no: Option<String>,
    install_date: Option<DateTime<Utc>>,
    install_address: Option<String>,
    install_technician: Option<String>,
    own_no: Option<String>,
    own_name: Option<String>,
    own_phone: Option<String>,
    own_id_card: Option<String>,
    own_address: Option<String>,
    own_email: Option<String>,
    group_id: i32,
    operation_status: i16,
    operation_route: Option<String>,
    operation_area: Option<String>,
    operation_company: Option<String>,
    driver_name: Option<String>,
    driver_phone: Option<String>,
    driver_license_no: Option<String>,
    purchase_price: Option<f64>,
    annual_fee: Option<f64>,
    insurance_fee: Option<f64>,
    // 模拟数据信息
    is_simulation: bool,
    simulation_source: Option<String>,
    remark: Option<String>,
    create_user_id: i32,
}

// 定义测试用的车辆更新请求结构体
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct TestVehicleUpdate {
    vehicle_name: Option<String>,
    license_plate: Option<String>,
    vehicle_type: Option<String>,
    vehicle_color: Option<String>,
    vehicle_brand: Option<String>,
    vehicle_model: Option<String>,
    engine_no: Option<String>,
    frame_no: Option<String>,
    register_date: Option<NaiveDateTime>,
    inspection_date: Option<NaiveDateTime>,
    insurance_date: Option<NaiveDateTime>,
    seating_capacity: Option<i32>,
    load_capacity: Option<f64>,
    vehicle_length: Option<f64>,
    vehicle_width: Option<f64>,
    vehicle_height: Option<f64>,
    device_id: Option<String>,
    terminal_type: Option<String>,
    communication_type: Option<String>,
    sim_card_no: Option<String>,
    install_date: Option<NaiveDateTime>,
    install_address: Option<String>,
    install_technician: Option<String>,
    own_no: Option<String>,
    own_name: Option<String>,
    own_phone: Option<String>,
    own_id_card: Option<String>,
    own_address: Option<String>,
    own_email: Option<String>,
    group_id: Option<i32>,
    operation_status: Option<i32>,
    operation_route: Option<String>,
    operation_area: Option<String>,
    operation_company: Option<String>,
    driver_name: Option<String>,
    driver_phone: Option<String>,
    driver_license_no: Option<String>,
    purchase_price: Option<f64>,
    annual_fee: Option<f64>,
    insurance_fee: Option<f64>,
    remark: Option<String>,
    status: Option<i32>,
    update_user_id: Option<i32>,
}

// 测试车辆CRUD功能
#[actix_web::test]
async fn test_vehicle_crud() {
    // 获取互斥锁,确保测试按顺序执行
    let _lock = TEST_MUTEX.lock().await;

    // 初始化测试数据库
    let pool = init_test_db().await;

    // 创建测试应用
    let app = App::new().app_data(web::Data::new(pool.clone())).service(
        web::scope("/api")
            // 车辆管理路由
            .route(
                "/vehicles",
                web::get().to(tms_server::routes::vehicles::get_vehicles),
            )
            .route(
                "/vehicles/{id}",
                web::get().to(tms_server::routes::vehicles::get_vehicle),
            )
            .route(
                "/vehicles",
                web::post().to(tms_server::routes::vehicles::create_vehicle),
            )
            .route(
                "/vehicles/{id}",
                web::put().to(tms_server::routes::vehicles::update_vehicle),
            )
            .route(
                "/vehicles/{id}",
                web::delete().to(tms_server::routes::vehicles::delete_vehicle),
            ),
    );

    let test_server = actix_web::test::init_service(app).await;

    // 创建测试车辆数据
    let now = Utc::now();
    let test_vehicle = TestVehicleCreate {
        vehicle_name: "Test Vehicle".to_string(),
        license_plate: "TEST123".to_string(),
        vehicle_type: "Truck".to_string(),
        vehicle_color: "Red".to_string(),
        vehicle_brand: "Test Brand".to_string(),
        vehicle_model: "Test Model".to_string(),
        engine_no: "ENG123456".to_string(),
        frame_no: "FRM123456".to_string(),
        register_date: now,
        inspection_date: now,
        insurance_date: now,
        seating_capacity: 2,
        load_capacity: 5.5,
        vehicle_length: 5.0,
        vehicle_width: 2.0,
        vehicle_height: 2.5,
        device_id: Some("DEV123".to_string()),
        terminal_type: Some("GPS".to_string()),
        communication_type: Some("4G".to_string()),
        sim_card_no: Some("13800138000".to_string()),
        install_date: Some(now),
        install_address: Some("Test Address".to_string()),
        install_technician: Some("Test Technician".to_string()),
        own_no: Some("OWN123".to_string()),
        own_name: Some("Test Owner".to_string()),
        own_phone: Some("13900139000".to_string()),
        own_id_card: Some("110101199001011234".to_string()),
        own_address: Some("Test Owner Address".to_string()),
        own_email: Some("test@example.com".to_string()),
        group_id: 1,
        operation_status: 1i16,
        operation_route: Some("Test Route".to_string()),
        operation_area: Some("Test Area".to_string()),
        operation_company: Some("Test Company".to_string()),
        driver_name: Some("Test Driver".to_string()),
        driver_phone: Some("13700137000".to_string()),
        driver_license_no: Some("A123456789".to_string()),
        purchase_price: Some(500000.0),
        annual_fee: Some(10000.0),
        insurance_fee: Some(5000.0),
        is_simulation: false,
        simulation_source: None,
        remark: Some("Test Remark".to_string()),
        create_user_id: 1,
    };

    // 测试创建车辆
    let create_request = actix_web::test::TestRequest::post()
        .uri("/api/vehicles")
        .set_json(test_vehicle.clone())
        .to_request();
    let create_response = actix_web::test::call_service(&test_server, create_request).await;

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // 测试获取所有车辆
    let get_request = actix_web::test::TestRequest::get()
        .uri("/api/vehicles")
        .to_request();
    let get_response = actix_web::test::call_service(&test_server, get_request).await;

    assert_eq!(get_response.status(), StatusCode::OK);

    // 测试获取单个车辆
    let get_one_request = actix_web::test::TestRequest::get()
        .uri("/api/vehicles/1")
        .to_request();
    let get_one_response = actix_web::test::call_service(&test_server, get_one_request).await;

    assert_eq!(get_one_response.status(), StatusCode::OK);

    // 测试更新车辆
    let update_data = TestVehicleUpdate {
        vehicle_name: Some("Updated Vehicle".to_string()),
        ..Default::default()
    };

    let update_request = actix_web::test::TestRequest::put()
        .uri("/api/vehicles/1")
        .set_json(update_data)
        .to_request();
    let update_response = actix_web::test::call_service(&test_server, update_request).await;

    assert_eq!(update_response.status(), StatusCode::OK);

    // 测试删除车辆
    let delete_request = actix_web::test::TestRequest::delete()
        .uri("/api/vehicles/1")
        .to_request();
    let delete_response = actix_web::test::call_service(&test_server, delete_request).await;

    assert_eq!(delete_response.status(), StatusCode::OK);
}

// 测试车辆数据校验
#[actix_web::test]
async fn test_vehicle_validation() {
    // 获取互斥锁,确保测试按顺序执行
    let _lock = TEST_MUTEX.lock().await;

    // 初始化测试数据库
    let pool = init_test_db().await;

    // 创建测试应用
    let app = App::new().app_data(web::Data::new(pool.clone())).service(
        web::scope("/api")
            // 车辆管理路由
            .route(
                "/vehicles",
                web::post().to(tms_server::routes::vehicles::create_vehicle),
            ),
    );

    let test_server = actix_web::test::init_service(app).await;

    let now = Utc::now();

    // 测试创建车辆时缺少必填字段
    let invalid_vehicle = TestVehicleCreate {
        vehicle_name: "".to_string(), // 空车辆名称
        license_plate: "TEST123".to_string(),
        vehicle_type: "Truck".to_string(),
        vehicle_color: "Red".to_string(),
        vehicle_brand: "Test Brand".to_string(),
        vehicle_model: "Test Model".to_string(),
        engine_no: "ENG123456".to_string(),
        frame_no: "FRM123456".to_string(),
        register_date: now,
        inspection_date: now,
        insurance_date: now,
        seating_capacity: 2,
        load_capacity: 5.5,
        vehicle_length: 5.0,
        vehicle_width: 2.0,
        vehicle_height: 2.5,
        device_id: None,
        terminal_type: None,
        communication_type: None,
        sim_card_no: None,
        install_date: None,
        install_address: None,
        install_technician: None,
        own_no: None,
        own_name: None,
        own_phone: None,
        own_id_card: None,
        own_address: None,
        own_email: None,
        group_id: 1,
        operation_status: 1i16,
        operation_route: None,
        operation_area: None,
        operation_company: None,
        driver_name: None,
        driver_phone: None,
        driver_license_no: None,
        purchase_price: None,
        annual_fee: None,
        insurance_fee: None,
        is_simulation: false,
        simulation_source: None,
        remark: None,
        create_user_id: 1,
    };

    let invalid_request = actix_web::test::TestRequest::post()
        .uri("/api/vehicles")
        .set_json(invalid_vehicle)
        .to_request();
    let invalid_response = actix_web::test::call_service(&test_server, invalid_request).await;

    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);
}

// 测试删除带有称重数据的车辆
#[actix_web::test]
async fn test_delete_vehicle_with_weighing_data() {
    // 获取互斥锁,确保测试按顺序执行
    let _lock = TEST_MUTEX.lock().await;

    // 初始化测试数据库
    let pool = init_test_db().await;

    // 创建测试应用
    let app = App::new().app_data(web::Data::new(pool.clone())).service(
        web::scope("/api")
            // 车辆管理路由
            .route(
                "/vehicles",
                web::get().to(tms_server::routes::vehicles::get_vehicles),
            )
            .route(
                "/vehicles",
                web::post().to(tms_server::routes::vehicles::create_vehicle),
            )
            .route(
                "/vehicles/{id}",
                web::delete().to(tms_server::routes::vehicles::delete_vehicle),
            ),
    );

    let test_server = actix_web::test::init_service(app).await;

    // 创建测试车辆数据
    let now = Utc::now();
    let test_vehicle = TestVehicleCreate {
        vehicle_name: "Test Vehicle with Weighing Data".to_string(),
        license_plate: "TEST456".to_string(),
        vehicle_type: "Truck".to_string(),
        vehicle_color: "Blue".to_string(),
        vehicle_brand: "Test Brand".to_string(),
        vehicle_model: "Test Model".to_string(),
        engine_no: "ENG789012".to_string(),
        frame_no: "FRM789012".to_string(),
        register_date: now,
        inspection_date: now,
        insurance_date: now,
        seating_capacity: 2,
        load_capacity: 10.0,
        vehicle_length: 6.0,
        vehicle_width: 2.5,
        vehicle_height: 3.0,
        device_id: Some("DEV456".to_string()),
        terminal_type: Some("GPS".to_string()),
        communication_type: Some("4G".to_string()),
        sim_card_no: Some("13800138001".to_string()),
        install_date: Some(now),
        install_address: Some("Test Address".to_string()),
        install_technician: Some("Test Technician".to_string()),
        own_no: Some("OWN456".to_string()),
        own_name: Some("Test Owner".to_string()),
        own_phone: Some("13900139001".to_string()),
        own_id_card: Some("110101199001011235".to_string()),
        own_address: Some("Test Owner Address".to_string()),
        own_email: Some("test2@example.com".to_string()),
        group_id: 1,
        operation_status: 1i16,
        operation_route: Some("Test Route".to_string()),
        operation_area: Some("Test Area".to_string()),
        operation_company: Some("Test Company".to_string()),
        driver_name: Some("Test Driver".to_string()),
        driver_phone: Some("13700137001".to_string()),
        driver_license_no: Some("A123456789".to_string()),
        purchase_price: Some(600000.0),
        annual_fee: Some(12000.0),
        insurance_fee: Some(6000.0),
        is_simulation: false,
        simulation_source: None,
        remark: Some("Test Vehicle with Weighing Data".to_string()),
        create_user_id: 1,
    };

    // 1. 创建测试车辆
    let create_request = actix_web::test::TestRequest::post()
        .uri("/api/vehicles")
        .set_json(test_vehicle.clone())
        .to_request();
    let create_response = actix_web::test::call_service(&test_server, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // 2. 直接向数据库添加称重数据
    let now = Utc::now().naive_utc();

    // 添加多条称重数据
    sqlx::query(
        r#"
        INSERT INTO weighing_data (vehicle_id, device_id, weighing_time, gross_weight, net_weight)
        VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind(1)
    .bind("DEV456")
    .bind(now)
    .bind(15.5)
    .bind(10.5)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO weighing_data (vehicle_id, device_id, weighing_time, gross_weight, net_weight)
        VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind(1)
    .bind("DEV456")
    .bind(now - chrono::Duration::hours(1))
    .bind(16.0)
    .bind(11.0)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO weighing_data (vehicle_id, device_id, weighing_time, gross_weight, net_weight)
        VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind(1)
    .bind("DEV456")
    .bind(now - chrono::Duration::hours(2))
    .bind(15.0)
    .bind(10.0)
    .execute(&pool)
    .await
    .unwrap();

    // 3. 验证称重数据已添加
    let weighing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM weighing_data WHERE vehicle_id = $1")
            .bind(1)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(weighing_count, 3);

    // 4. 删除车辆,应该同时删除相关称重数据
    let delete_request = actix_web::test::TestRequest::delete()
        .uri("/api/vehicles/1")
        .to_request();
    let delete_response = actix_web::test::call_service(&test_server, delete_request).await;

    // 5. 验证车辆删除成功
    assert_eq!(delete_response.status(), StatusCode::OK);

    // 6. 验证车辆已从数据库中删除
    let vehicle_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE vehicle_id = $1")
            .bind(1)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(vehicle_count, 0);

    // 7. 验证相关称重数据已被删除
    let weighing_count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM weighing_data WHERE vehicle_id = $1")
            .bind(1)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        weighing_count_after, 0,
        "Weighing data should be deleted when vehicle is deleted"
    );
}
