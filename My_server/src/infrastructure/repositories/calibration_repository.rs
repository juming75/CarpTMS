//! 载重标定数据访问层

use chrono::{NaiveDateTime, TimeZone, Utc};

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::{info, warn};

use crate::domain::entities::calibration::{CalibrationHistory, SensorCalibration};
use crate::errors::AppError;

/// 标定 Repository trait
#[async_trait]
pub trait CalibrationRepository: Send + Sync {
    async fn get_calibrations(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<SensorCalibration>, i64), AppError>;

    async fn get_calibration(&self, id: i32) -> Result<Option<SensorCalibration>, AppError>;

    async fn create_calibration(&self, calibration: &SensorCalibration) -> Result<SensorCalibration, AppError>;

    async fn update_calibration(&self, id: i32, calibration: &SensorCalibration) -> Result<SensorCalibration, AppError>;

    async fn delete_calibration(&self, id: i32) -> Result<(), AppError>;

    async fn get_calibration_history(&self, page: i32, page_size: i32, sensor_no: Option<i32>, vehicle_id: Option<i32>, plate_no: Option<&str>) -> Result<(Vec<CalibrationHistory>, i64), AppError>;

    async fn create_calibration_history(&self, history: &CalibrationHistory) -> Result<CalibrationHistory, AppError>;

    async fn delete_calibration_history(&self, id: i32) -> Result<(), AppError>;
}

/// PostgreSQL 标定 Repository 实现
pub struct PgCalibrationRepository {
    pool: PgPool,
}

impl PgCalibrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_calibration(row: &sqlx::postgres::PgRow) -> SensorCalibration {
        SensorCalibration {
            id: row.get("id"),
            sensor_no: row.get("sensor_no"),
            vehicle_id: row.get("vehicle_id"),
            plate_no: row.get("plate_no"),
            sensor_side: row.get("sensor_side"),
            sensor_group: row.get("sensor_group"),
            self_weight: row.get("self_weight"),
            polynomial_json: row.get("polynomial_json"),
            linear_segments_json: row.get("linear_segments_json"),
            is_calibrated: row.get("is_calibrated"),
            create_time: Utc.from_utc_datetime(&row.get::<NaiveDateTime, _>("create_time")),
            update_time: row.get::<Option<NaiveDateTime>, _>("update_time").map(|t| Utc.from_utc_datetime(&t)),
            calibration_points: row.get("calibration_points"),
            pa_raw: row.get("pa_raw"),
            axle_number: row.get("axle_number"),
            is_left_wheel: row.get("is_left_wheel"),
            turn_point: row.get("turn_point"),
            polynomial_order: row.get("polynomial_order"),
            r2_score: row.get("r2_score"),
            rmse: row.get("rmse"),
            max_error: row.get("max_error"),
            point_count: row.get("point_count"),
            rated_total_weight: row.get("rated_total_weight"),
            tare_weight: row.get("tare_weight"),
        }
    }

    fn map_row_to_history(row: &sqlx::postgres::PgRow) -> CalibrationHistory {
        CalibrationHistory {
            id: row.get("id"),
            sensor_no: row.get("sensor_no"),
            vehicle_id: row.get("vehicle_id"),
            plate_no: row.get("plate_no"),
            polynomial_json: row.get("polynomial_json"),
            polynomial_order: row.get("polynomial_order"),
            r2_score: row.get("r2_score"),
            rmse: row.get("rmse"),
            max_error: row.get("max_error"),
            point_count: row.get("point_count"),
            operation_type: row.get("operation_type"),
            operation_type_name: row.get("operation_type_name"),
            operator: row.get("operator"),
            remark: row.get("remark"),
            is_valid: row.get("is_valid"),
            create_time: Utc.from_utc_datetime(&row.get::<NaiveDateTime, _>("create_time")),
            update_time: row.get::<Option<NaiveDateTime>, _>("update_time").map(|t| Utc.from_utc_datetime(&t)),
        }
    }
}

#[async_trait::async_trait]
impl CalibrationRepository for PgCalibrationRepository {
    async fn get_calibrations(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<SensorCalibration>, i64), AppError> {
        let offset = (page - 1) * page_size;

        let query = r#"
            SELECT id, sensor_no, vehicle_id, plate_no, sensor_side, sensor_group,
                   self_weight, polynomial_json, linear_segments_json, is_calibrated,
                   create_time, update_time,
                   calibration_points, pa_raw, axle_number, is_left_wheel,
                   turn_point, polynomial_order, r2_score, rmse, max_error,
                   point_count, rated_total_weight, tare_weight
            FROM sensor_calibration
            WHERE ($1::int4 IS NULL OR sensor_no = $1)
              AND ($2::int4 IS NULL OR vehicle_id = $2)
              AND ($3::text IS NULL OR plate_no LIKE '%' || $3 || '%')
            ORDER BY create_time DESC
            LIMIT $4 OFFSET $5
        "#;

        let rows = sqlx::query(query)
            .bind(sensor_no)
            .bind(vehicle_id)
            .bind(plate_no)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let calibrations: Vec<SensorCalibration> = rows.iter().map(Self::map_row_to_calibration).collect();
        let total = calibrations.len() as i64;

        Ok((calibrations, total))
    }

    async fn get_calibration(&self, id: i32) -> Result<Option<SensorCalibration>, AppError> {
        let query = r#"
            SELECT id, sensor_no, vehicle_id, plate_no, sensor_side, sensor_group,
                   self_weight, polynomial_json, linear_segments_json, is_calibrated,
                   create_time, update_time,
                   calibration_points, pa_raw, axle_number, is_left_wheel,
                   turn_point, polynomial_order, r2_score, rmse, max_error,
                   point_count, rated_total_weight, tare_weight
            FROM sensor_calibration WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row_to_calibration(&r)))
    }

    async fn create_calibration(&self, calibration: &SensorCalibration) -> Result<SensorCalibration, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO sensor_calibration
                (sensor_no, vehicle_id, plate_no, sensor_side, sensor_group,
                 self_weight, polynomial_json, linear_segments_json, is_calibrated,
                 calibration_points, pa_raw, axle_number, is_left_wheel,
                 turn_point, polynomial_order, r2_score, rmse, max_error, point_count,
                 rated_total_weight, tare_weight, create_time, update_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $22)
            RETURNING *
            "#,
        )
        .bind(calibration.sensor_no)
        .bind(calibration.vehicle_id)
        .bind(&calibration.plate_no)
        .bind(&calibration.sensor_side)
        .bind(calibration.sensor_group)
        .bind(calibration.self_weight)
        .bind(&calibration.polynomial_json)
        .bind(&calibration.linear_segments_json)
        .bind(calibration.is_calibrated)
        .bind(&calibration.calibration_points)
        .bind(calibration.pa_raw)
        .bind(calibration.axle_number)
        .bind(calibration.is_left_wheel)
        .bind(calibration.turn_point)
        .bind(calibration.polynomial_order)
        .bind(calibration.r2_score)
        .bind(calibration.rmse)
        .bind(calibration.max_error)
        .bind(calibration.point_count)
        .bind(calibration.rated_total_weight)
        .bind(calibration.tare_weight)
        .bind(chrono::Local::now().naive_local())
        .fetch_one(&self.pool)
        .await?;

        info!("Created calibration for sensor: {}", calibration.sensor_no);
        Ok(Self::map_row_to_calibration(&row))
    }

    async fn update_calibration(&self, id: i32, calibration: &SensorCalibration) -> Result<SensorCalibration, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE sensor_calibration SET
                sensor_no           = $1,
                vehicle_id          = $2,
                plate_no            = $3,
                sensor_side         = $4,
                sensor_group        = $5,
                self_weight         = $6,
                polynomial_json     = $7,
                linear_segments_json = $8,
                is_calibrated      = $9,
                update_time         = CURRENT_TIMESTAMP
            WHERE id = $10
            RETURNING *
            "#,
        )
        .bind(calibration.sensor_no)
        .bind(calibration.vehicle_id)
        .bind(&calibration.plate_no)
        .bind(&calibration.sensor_side)
        .bind(calibration.sensor_group)
        .bind(calibration.self_weight)
        .bind(&calibration.polynomial_json)
        .bind(&calibration.linear_segments_json)
        .bind(calibration.is_calibrated)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::business_error(&e.to_string(), None))?;

        info!("Updated calibration id: {}", id);
        Ok(Self::map_row_to_calibration(&row))
    }

    async fn delete_calibration(&self, id: i32) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM sensor_calibration WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            warn!("Calibration not found for deletion: {}", id);
            return Err(AppError::not_found_error("Calibration not found".to_string()));
        }

        info!("Deleted calibration id: {}", id);
        Ok(())
    }

    async fn get_calibration_history(&self, page: i32, page_size: i32, sensor_no: Option<i32>, vehicle_id: Option<i32>, plate_no: Option<&str>) -> Result<(Vec<CalibrationHistory>, i64), AppError> {
        let offset = (page - 1) * page_size;

        let mut query = r#"
            SELECT id, sensor_no, vehicle_id, plate_no, polynomial_json,
                   polynomial_order, r2_score, rmse, max_error, point_count,
                   operation_type, operation_type_name, operator, remark, is_valid,
                   create_time, update_time
            FROM calibration_history
            WHERE 1=1
        "#.to_string();

        let mut param_count = 0;
        if sensor_no.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sensor_no = ${}", param_count));
        }
        if vehicle_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND vehicle_id = ${}", param_count));
        }
        if plate_no.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND plate_no = ${}", param_count));
        }

        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        query.push_str(&format!(" ORDER BY create_time DESC LIMIT ${} OFFSET ${}", limit_param, offset_param));

        let mut sqlx_query = sqlx::query(&query);

        if let Some(sensor_no_val) = sensor_no {
            sqlx_query = sqlx_query.bind(sensor_no_val);
        }
        if let Some(vehicle_id_val) = vehicle_id {
            sqlx_query = sqlx_query.bind(vehicle_id_val);
        }
        if let Some(plate_no_val) = plate_no {
            sqlx_query = sqlx_query.bind(plate_no_val);
        }

        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let rows = sqlx_query.fetch_all(&self.pool).await?;

        let history: Vec<CalibrationHistory> = rows.iter().map(Self::map_row_to_history).collect();

        // 获取总数
        let mut count_query = "SELECT COUNT(*) FROM calibration_history WHERE 1=1".to_string();

        let mut count_param_count = 0;
        if sensor_no.is_some() {
            count_param_count += 1;
            count_query.push_str(&format!(" AND sensor_no = ${}", count_param_count));
        }
        if vehicle_id.is_some() {
            count_param_count += 1;
            count_query.push_str(&format!(" AND vehicle_id = ${}", count_param_count));
        }
        if plate_no.is_some() {
            count_param_count += 1;
            count_query.push_str(&format!(" AND plate_no = ${}", count_param_count));
        }

        let mut sqlx_count_query = sqlx::query_scalar(&count_query);

        if let Some(sensor_no_val) = sensor_no {
            sqlx_count_query = sqlx_count_query.bind(sensor_no_val);
        }
        if let Some(vehicle_id_val) = vehicle_id {
            sqlx_count_query = sqlx_count_query.bind(vehicle_id_val);
        }
        if let Some(plate_no_val) = plate_no {
            sqlx_count_query = sqlx_count_query.bind(plate_no_val);
        }

        let total: i64 = sqlx_count_query.fetch_one(&self.pool).await?;

        Ok((history, total))
    }

    async fn create_calibration_history(&self, history: &CalibrationHistory) -> Result<CalibrationHistory, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO calibration_history
                (sensor_no, vehicle_id, plate_no, polynomial_json, polynomial_order,
                 r2_score, rmse, max_error, point_count,
                 operation_type, operation_type_name, operator, remark, is_valid,
                 create_time, update_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $15)
            RETURNING *
            "#,
        )
        .bind(history.sensor_no)
        .bind(history.vehicle_id)
        .bind(&history.plate_no)
        .bind(&history.polynomial_json)
        .bind(history.polynomial_order)
        .bind(history.r2_score)
        .bind(history.rmse)
        .bind(history.max_error)
        .bind(history.point_count)
        .bind(&history.operation_type)
        .bind(&history.operation_type_name)
        .bind(&history.operator)
        .bind(&history.remark)
        .bind(history.is_valid)
        .bind(chrono::Local::now().naive_local())
        .fetch_one(&self.pool)
        .await?;

        info!("Created calibration history for sensor: {}", history.sensor_no);
        Ok(Self::map_row_to_history(&row))
    }

    async fn delete_calibration_history(&self, id: i32) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM calibration_history WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            warn!("Calibration history not found for deletion: {}", id);
            return Err(AppError::not_found_error("Calibration history not found".to_string()));
        }

        info!("Deleted calibration history id: {}", id);
        Ok(())
    }
}
