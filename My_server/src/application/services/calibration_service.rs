//! 载重标定应用服务
//!
//! 职责：协调 domain entities 与 infrastructure repositories
//! 数学计算委托给 `WeightCalibrationService`（domain service）

use serde_json::json;
use tracing::{info, warn};

use crate::domain::entities::calibration::{CalibrationHistory, SensorCalibration};
use crate::errors::AppError;
use crate::infrastructure::repositories::calibration_repository::{
    CalibrationRepository, PgCalibrationRepository,
};

/// 标定服务接口
#[allow(async_fn_in_trait)]
pub trait CalibrationService {
    /// 获取传感器标定列表
    async fn get_calibrations(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<SensorCalibration>, i64), AppError>;

    /// 获取传感器标定详情
    async fn get_calibration(&self, id: i32) -> Result<Option<SensorCalibration>, AppError>;

    /// 创建传感器标定
    async fn create_calibration(
        &self,
        calibration: &SensorCalibration,
    ) -> Result<SensorCalibration, AppError>;

    /// 更新传感器标定
    async fn update_calibration(
        &self,
        id: i32,
        calibration: &SensorCalibration,
    ) -> Result<SensorCalibration, AppError>;

    /// 删除传感器标定
    async fn delete_calibration(&self, id: i32) -> Result<(), AppError>;

    /// 获取标定历史记录
    async fn get_calibration_history(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<CalibrationHistory>, i64), AppError>;

    /// 创建标定历史记录
    async fn create_calibration_history(
        &self,
        history: &CalibrationHistory,
    ) -> Result<CalibrationHistory, AppError>;

    /// 标记传感器标定为已标定
    async fn mark_calibration_completed(&self, id: i32) -> Result<SensorCalibration, AppError>;

    /// 验证标定数据的有效性
    fn validate_calibration_data(&self, calibration: &SensorCalibration) -> Result<(), AppError>;
}

/// 标定服务实现
pub struct CalibrationServiceImpl {
    calibration_repository: PgCalibrationRepository,
}

impl CalibrationServiceImpl {
    /// 创建新的标定服务
    pub fn new(calibration_repository: PgCalibrationRepository) -> Self {
        Self {
            calibration_repository,
        }
    }

    /// 调用 WeightCalibrationService 多项式拟合（固有方法，非 trait）
    fn fit_polynomial(
        &self,
        points_json: &str,
        plate_no: &str,
        axle_number: u8,
        is_left_wheel: bool,
        turn_point: f64,
    ) -> Option<(String, f64, f64, f64, i32)> {
        let points: Vec<serde_json::Value> = serde_json::from_str(points_json).ok()?;
        let mut math_svc =
            crate::services::weight_calibration_service::WeightCalibrationService::new();

        for pt in &points {
            let pa_raw = pt.get("pa_raw")?.as_i64()? as u32;
            let actual_weight = pt.get("actual_weight")?.as_f64()?;
            let temperature = pt.get("temperature")?.as_f64().unwrap_or(25.0);
            let load_percentage = pt.get("load_percentage")?.as_i64().unwrap_or(0) as u8;

            let params = crate::services::weight_calibration_service::CalibrationPointParams {
                plate_number: plate_no.to_string(),
                axle_number,
                is_left_wheel,
                pa_value: (pa_raw as f64 / 100.0).round() as u32,
                actual_weight,
                temperature,
                load_percentage,
            };
            let _ = math_svc.add_calibration_point(params);
        }

        let coeffs = math_svc
            .calculate_sensor_coefficients(plate_no, axle_number, is_left_wheel, turn_point)
            .ok()?;

        let pts_ref = math_svc
            .get_calibration_sheet(plate_no)
            .and_then(|s| s.axle_records.get(&axle_number))
            .map(|a| {
                if is_left_wheel {
                    a.left_wheel.calibration_points.clone()
                } else {
                    a.right_wheel.calibration_points.clone()
                }
            })
            .unwrap_or_default();

        let n = pts_ref.len() as f64;
        let y_mean = pts_ref.iter().map(|p| p.actual_weight).sum::<f64>() / n.max(1.0);
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;
        let mut max_err = 0.0f64;

        for p in &pts_ref {
            let y_pred =
                crate::services::weight_calibration_service::PolynomialCoefficients::calculate(
                    &coeffs.after_turning,
                    p.pa_value as f64,
                );
            let err = (y_pred - p.actual_weight).abs();
            ss_res += err * err;
            ss_tot += (p.actual_weight - y_mean).powi(2);
            max_err = max_err.max(err);
        }

        let r2 = if ss_tot > 0.0 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };
        let rmse = (ss_res / n.max(1.0)).sqrt();

        let coefs_json = json!({
            "coef_1":   coeffs.after_turning.coef_1,
            "constant": coeffs.after_turning.constant,
            "before_coef_1":   coeffs.before_turning.coef_1,
            "before_constant": coeffs.before_turning.constant,
            "turning_point":   coeffs.turning_point,
        })
        .to_string();

        Some((coefs_json, r2, rmse, max_err, pts_ref.len() as i32))
    }
}

impl CalibrationService for CalibrationServiceImpl {
    async fn get_calibrations(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<SensorCalibration>, i64), AppError> {
        let (calibrations, total): (Vec<SensorCalibration>, i64) = self
            .calibration_repository
            .get_calibrations(page, page_size, sensor_no, vehicle_id, plate_no)
            .await?;

        info!(
            "Fetched {} calibrations out of total {}",
            calibrations.len(),
            total
        );
        Ok((calibrations, total))
    }

    async fn get_calibration(&self, id: i32) -> Result<Option<SensorCalibration>, AppError> {
        let calibration = self.calibration_repository.get_calibration(id).await?;

        match &calibration {
            Some(calib) => info!("Fetched calibration for sensor: {}", calib.sensor_no),
            None => warn!("Calibration not found: {}", id),
        }

        Ok(calibration)
    }

    async fn create_calibration(
        &self,
        calibration: &SensorCalibration,
    ) -> Result<SensorCalibration, AppError> {
        // 验证标定数据
        self.validate_calibration_data(calibration)?;

        // 检查传感器是否已存在标定数据
        let (existing_calibrations, _) = self
            .calibration_repository
            .get_calibrations(1, 10, Some(calibration.sensor_no), None, None)
            .await?;

        for existing in &existing_calibrations {
            if existing.sensor_no == calibration.sensor_no {
                return Err(AppError::business_error(
                    "Calibration already exists for this sensor",
                    None,
                ));
            }
        }

        // 如果传入了标定点（calibration_points JSON），用数学引擎计算多项式系数
        let (polynomial_json, r2_score, rmse, max_error, point_count) =
            if let Some(pts_json) = &calibration.calibration_points {
                self.fit_polynomial(pts_json, &calibration.plate_no, 1, true, 50000.0)
                    .unwrap_or_else(|| {
                        (
                            calibration.polynomial_json.clone().unwrap_or_default(),
                            0.0,
                            0.0,
                            0.0,
                            0,
                        )
                    })
            } else {
                (
                    calibration.polynomial_json.clone().unwrap_or_default(),
                    0.0,
                    0.0,
                    0.0,
                    0,
                )
            };

        // 用计算出的系数创建实体
        let mut calib = calibration.clone();
        calib.polynomial_json = Some(polynomial_json.clone());
        calib.is_calibrated = true;

        let created_calibration = self
            .calibration_repository
            .create_calibration(&calib)
            .await?;

        // 写历史记录
        let history = CalibrationHistory::new(
            calibration.sensor_no,
            calibration.vehicle_id,
            calibration.plate_no.clone(),
            polynomial_json,
            2,
            r2_score,
            rmse,
            max_error,
            point_count,
            "auto".to_string(),
            Some("多项式拟合自动标定".to_string()),
            Some("system".to_string()),
            Some(format!("R²={:.4} RMSE={:.2}", r2_score, rmse)),
        );
        let _ = self
            .calibration_repository
            .create_calibration_history(&history)
            .await;

        info!(
            "Created calibration for sensor: {} (R²={:.4})",
            calibration.sensor_no, r2_score
        );
        Ok(created_calibration)
    }

    async fn update_calibration(
        &self,
        id: i32,
        calibration: &SensorCalibration,
    ) -> Result<SensorCalibration, AppError> {
        // 验证标定数据
        self.validate_calibration_data(calibration)?;

        // 检查标定数据是否存在
        let existing_calibration: Option<SensorCalibration> =
            self.calibration_repository.get_calibration(id).await?;
        if existing_calibration.is_none() {
            return Err(AppError::not_found_error(
                "Calibration not found".to_string(),
            ));
        }

        // 更新标定数据
        let updated_calibration = self
            .calibration_repository
            .update_calibration(id, calibration)
            .await?;

        // 如果标定完成，创建历史记录
        if calibration.is_calibrated {
            let history = CalibrationHistory::new(
                calibration.sensor_no,
                calibration.vehicle_id,
                calibration.plate_no.clone(),
                calibration.polynomial_json.clone().unwrap_or_default(),
                calibration.polynomial_order.unwrap_or(2),
                calibration.r2_score.unwrap_or(0.0),
                calibration.rmse.unwrap_or(0.0),
                calibration.max_error.unwrap_or(0.0),
                calibration.point_count.unwrap_or(0),
                "manual".to_string(),
                Some("手动标定".to_string()),
                Some("admin".to_string()),
                Some("标定完成".to_string()),
            );

            self.calibration_repository
                .create_calibration_history(&history)
                .await?;
        }

        info!("Updated calibration: {}", id);
        Ok(updated_calibration)
    }

    async fn delete_calibration(&self, id: i32) -> Result<(), AppError> {
        // 检查标定数据是否存在
        let existing_calibration: Option<SensorCalibration> =
            self.calibration_repository.get_calibration(id).await?;
        if existing_calibration.is_none() {
            return Err(AppError::not_found_error(
                "Calibration not found".to_string(),
            ));
        }

        // 删除标定数据
        self.calibration_repository.delete_calibration(id).await?;

        info!("Deleted calibration: {}", id);
        Ok(())
    }

    async fn get_calibration_history(
        &self,
        page: i32,
        page_size: i32,
        sensor_no: Option<i32>,
        vehicle_id: Option<i32>,
        plate_no: Option<&str>,
    ) -> Result<(Vec<CalibrationHistory>, i64), AppError> {
        let (history, total): (Vec<CalibrationHistory>, i64) = self
            .calibration_repository
            .get_calibration_history(page, page_size, sensor_no, vehicle_id, plate_no)
            .await?;

        info!(
            "Fetched {} calibration history records out of total {}",
            history.len(),
            total
        );
        Ok((history, total))
    }

    async fn create_calibration_history(
        &self,
        history: &CalibrationHistory,
    ) -> Result<CalibrationHistory, AppError> {
        let created_history = self
            .calibration_repository
            .create_calibration_history(history)
            .await?;

        info!(
            "Created calibration history for sensor: {}",
            history.sensor_no
        );
        Ok(created_history)
    }

    async fn mark_calibration_completed(&self, id: i32) -> Result<SensorCalibration, AppError> {
        // 获取标定数据
        let existing_calibration: Option<SensorCalibration> =
            self.calibration_repository.get_calibration(id).await?;
        let mut calibration = existing_calibration
            .ok_or_else(|| AppError::not_found_error("Calibration not found".to_string()))?;

        // 标记为已标定
        calibration.mark_as_calibrated();

        // 更新标定数据
        let updated_calibration = self
            .calibration_repository
            .update_calibration(id, &calibration)
            .await?;

        // 创建标定历史记录
        let history = CalibrationHistory::new(
            calibration.sensor_no,
            calibration.vehicle_id,
            calibration.plate_no.clone(),
            calibration.polynomial_json.clone().unwrap_or_default(),
            calibration.polynomial_order.unwrap_or(2),
            calibration.r2_score.unwrap_or(0.0),
            calibration.rmse.unwrap_or(0.0),
            calibration.max_error.unwrap_or(0.0),
            calibration.point_count.unwrap_or(0),
            "auto".to_string(),
            Some("自动标定".to_string()),
            Some("system".to_string()),
            Some("标定完成".to_string()),
        );

        self.calibration_repository
            .create_calibration_history(&history)
            .await?;

        info!("Marked calibration as completed: {}", id);
        Ok(updated_calibration)
    }

    fn validate_calibration_data(&self, calibration: &SensorCalibration) -> Result<(), AppError> {
        // 验证传感器编号
        if calibration.sensor_no <= 0 {
            return Err(AppError::business_error(
                "Sensor number must be positive",
                None,
            ));
        }

        // 验证车辆ID
        if calibration.vehicle_id <= 0 {
            return Err(AppError::business_error(
                "Vehicle ID must be positive",
                None,
            ));
        }

        // 验证车牌号
        if calibration.plate_no.is_empty() {
            return Err(AppError::business_error(
                "Plate number cannot be empty",
                None,
            ));
        }

        // 验证传感器位置
        if calibration.sensor_side.is_empty() {
            return Err(AppError::business_error(
                "Sensor side cannot be empty",
                None,
            ));
        }

        // 如果已标定，验证多项式系数
        if calibration.is_calibrated
            && (calibration.polynomial_json.is_none()
                || calibration
                    .polynomial_json
                    .as_deref()
                    .is_none_or(|s| s.is_empty()))
        {
            return Err(AppError::business_error(
                "Polynomial coefficients are required for calibrated sensors",
                None,
            ));
        }

        Ok(())
    }
}
