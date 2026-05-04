//! / 腾讯地图地理编码
// 将经纬度转换为详细地址

use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 腾讯地图地理编码器
pub struct TencentGeocoder {
    client: Client,
    api_key: String,
    base_url: String,
}

impl TencentGeocoder {
    /// 创建新的地理编码器
    pub fn new(api_key: String) -> Self {
        info!("Creating Tencent geocoder");
        
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            api_key,
            base_url: "https://apis.map.qq.com/ws/geocoder/v1".to_string(),
        }
    }

    /// 逆地理编码:将经纬度转换为地址
    pub async fn reverse_geocode(&self, lat: f64, lng: f64) -> Result<GeocodeResult, GeocodeError> {
        let url = format!(
            "{}?location={},{}&key={}&get_poi=1",
            self.base_url, lat, lng, self.api_key
        );

        debug!("Calling geocoding API for lat={}, lng={}", lat, lng);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<TencentGeocodeResponse>().await {
                        Ok(geocode_response) => {
                            if geocode_response.status == 0 {
                                Ok(geocode_response.result.into())
                            } else {
                                error!("Geocoding API error: {}", geocode_response.message);
                                Err(GeocodeError::ApiError(geocode_response.message))
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse geocoding response: {}", e);
                            Err(GeocodeError::ParseError(e.to_string()))
                        }
                    }
                } else {
                    error!("Geocoding API returned status: {}", response.status());
                    Err(GeocodeError::HttpError(response.status().as_u16()))
                }
            }
            Err(e) => {
                error!("Geocoding request failed: {}", e);
                Err(GeocodeError::RequestError(e.to_string()))
            }
        }
    }

    /// 批量逆地理编码
    pub async fn batch_reverse_geocode(
        &self,
        locations: Vec<(f64, f64)>,
    ) -> Vec<Result<GeocodeResult, GeocodeError>> {
        let mut results = Vec::new();

        for (lat, lng) in locations {
            let result = self.reverse_geocode(lat, lng).await;
            results.push(result);

            // 避免超过 API 速率限制
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        results
    }
}

/// 地理编码结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeResult {
    pub location: Location,
    pub address: String,
    pub formatted_addresses: Vec<String>,
    pub address_component: AddressComponent,
    pub ad_info: AdInfo,
    pub pois: Option<Vec<Poi>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressComponent {
    pub nation: String,
    pub province: String,
    pub city: String,
    pub district: String,
    pub street: String,
    pub street_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdInfo {
    pub nation_code: String,
    pub adcode: String,
    pub province: String,
    pub city: String,
    pub district: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poi {
    pub id: String,
    pub title: String,
    pub address: String,
    pub category: String,
    pub location: Location,
    pub distance: f64,
}

/// 地理编码错误
#[derive(Debug, Clone, Serialize)]
pub enum GeocodeError {
    RequestError(String),
    HttpError(u16),
    ParseError(String),
    ApiError(String),
}

/// 腾讯地图 API 响应
#[derive(Debug, Deserialize)]
struct TencentGeocodeResponse {
    pub status: i32,
    pub message: String,
    pub result: TencentGeocodeResult,
}

#[derive(Debug, Deserialize)]
struct TencentGeocodeResult {
    pub location: Location,
    pub address: String,
    pub formatted_addresses: Option<Vec<String>>,
    pub address_component: AddressComponent,
    pub ad_info: AdInfo,
    pub pois: Option<Vec<Poi>>,
}

impl From<TencentGeocodeResult> for GeocodeResult {
    fn from(result: TencentGeocodeResult) -> Self {
        Self {
            location: result.location,
            address: result.address,
            formatted_addresses: result.formatted_addresses.unwrap_or_default(),
            address_component: result.address_component,
            ad_info: result.ad_info,
            pois: result.pois,
        }
    }
}






