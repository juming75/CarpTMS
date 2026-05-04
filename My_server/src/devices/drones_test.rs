//! 无人机模块单元测试

#[cfg(test)]
mod tests {
    use crate::devices::drones::{
        parse_flight_status, parse_vendor, DroneCommand, DroneInfo, DroneTelemetry, DroneVendor,
        FlightStatus,
    };

    #[test]
    fn test_parse_vendor_dji() {
        assert!(matches!(parse_vendor("dji"), DroneVendor::Dji));
        assert!(matches!(parse_vendor("DJI"), DroneVendor::Dji));
        assert!(matches!(parse_vendor("Dji"), DroneVendor::Dji));
    }

    #[test]
    fn test_parse_vendor_autel() {
        assert!(matches!(parse_vendor("autel"), DroneVendor::Autel));
        assert!(matches!(parse_vendor("AUTEL"), DroneVendor::Autel));
    }

    #[test]
    fn test_parse_vendor_px4() {
        assert!(matches!(parse_vendor("px4"), DroneVendor::Px4));
        assert!(matches!(parse_vendor("PX4"), DroneVendor::Px4));
    }

    #[test]
    fn test_parse_vendor_other() {
        match parse_vendor("unknown") {
            DroneVendor::Other(s) => assert_eq!(s, "unknown"),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_parse_flight_status_ground() {
        assert!(matches!(
            parse_flight_status("ground"),
            FlightStatus::Ground
        ));
        assert!(matches!(
            parse_flight_status("GROUND"),
            FlightStatus::Ground
        ));
        assert!(matches!(
            parse_flight_status("unknown"),
            FlightStatus::Ground
        ));
    }

    #[test]
    fn test_parse_flight_status_all_variants() {
        assert!(matches!(
            parse_flight_status("takeoff"),
            FlightStatus::Takeoff
        ));
        assert!(matches!(
            parse_flight_status("landing"),
            FlightStatus::Landing
        ));
        assert!(matches!(parse_flight_status("hover"), FlightStatus::Hover));
        assert!(matches!(
            parse_flight_status("cruising"),
            FlightStatus::Cruising
        ));
        assert!(matches!(parse_flight_status("rth"), FlightStatus::Rth));
        assert!(matches!(
            parse_flight_status("return_to_home"),
            FlightStatus::Rth
        ));
        assert!(matches!(
            parse_flight_status("emergency"),
            FlightStatus::Emergency
        ));
        assert!(matches!(
            parse_flight_status("disconnected"),
            FlightStatus::Disconnected
        ));
    }

    #[test]
    fn test_drone_command_serialization() {
        let cmd = DroneCommand::Takeoff { altitude: 100.0 };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("takeoff"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_drone_command_deserialization() {
        let json = r#"{"takeoff":{"altitude":50.0}}"#;
        let cmd: DroneCommand = serde_json::from_str(json).unwrap();
        match cmd {
            DroneCommand::Takeoff { altitude } => assert_eq!(altitude, 50.0),
            _ => panic!("Expected Takeoff command"),
        }
    }

    #[test]
    fn test_drone_command_land() {
        let cmd = DroneCommand::Land;
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: DroneCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, DroneCommand::Land));
    }

    #[test]
    fn test_drone_command_goto_waypoint() {
        let cmd = DroneCommand::GotoWaypoint {
            latitude: 39.9042,
            longitude: 116.4074,
            altitude: 120.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: DroneCommand = serde_json::from_str(&json).unwrap();
        match deserialized {
            DroneCommand::GotoWaypoint {
                latitude,
                longitude,
                altitude,
            } => {
                assert!((latitude - 39.9042).abs() < 0.0001);
                assert!((longitude - 116.4074).abs() < 0.0001);
                assert!((altitude - 120.0).abs() < 0.0001);
            }
            _ => panic!("Expected GotoWaypoint command"),
        }
    }

    #[test]
    fn test_drone_info_serialization() {
        let info = DroneInfo {
            id: 1,
            name: "Test Drone".to_string(),
            serial_number: "SN001".to_string(),
            vendor: DroneVendor::Dji,
            model: "M300".to_string(),
            firmware_version: "v1.0.0".to_string(),
            registration_code: "REG001".to_string(),
            status: "online".to_string(),
            telemetry: None,
            created_at: chrono::Utc::now(),
            last_telemetry: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Test Drone"));
        assert!(json.contains("SN001"));
        assert!(json.contains("dji"));
    }

    #[test]
    fn test_drone_telemetry_serialization() {
        let telemetry = DroneTelemetry {
            latitude: 39.9042,
            longitude: 116.4074,
            altitude: 100.0,
            altitude_agl: 50.0,
            speed: 10.5,
            heading: 45.0,
            battery_percent: 85.0,
            battery_voltage: 22.2,
            battery_current: 5.0,
            satellite_count: 12,
            signal_strength: 95.0,
            flight_status: FlightStatus::Cruising,
            flight_mode: "GPS".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&telemetry).unwrap();
        assert!(json.contains("39.9042"));
        assert!(json.contains("cruising"));
        assert!(json.contains("85"));
    }

    #[test]
    fn test_drone_vendor_serialization() {
        let dji = DroneVendor::Dji;
        let json = serde_json::to_string(&dji).unwrap();
        assert_eq!(json, r#""dji""#);

        let autel = DroneVendor::Autel;
        let json = serde_json::to_string(&autel).unwrap();
        assert_eq!(json, r#""autel""#);

        let px4 = DroneVendor::Px4;
        let json = serde_json::to_string(&px4).unwrap();
        assert_eq!(json, r#""px4""#);
    }

    #[test]
    fn test_drone_vendor_serialization_roundtrip() {
        let dji: DroneVendor = serde_json::from_str(r#""dji""#).unwrap();
        assert!(matches!(dji, DroneVendor::Dji));

        let autel: DroneVendor = serde_json::from_str(r#""autel""#).unwrap();
        assert!(matches!(autel, DroneVendor::Autel));

        let px4: DroneVendor = serde_json::from_str(r#""px4""#).unwrap();
        assert!(matches!(px4, DroneVendor::Px4));
    }

    #[test]
    fn test_flight_status_serialization() {
        let ground = FlightStatus::Ground;
        let json = serde_json::to_string(&ground).unwrap();
        assert_eq!(json, r#""ground""#);

        let cruising = FlightStatus::Cruising;
        let json = serde_json::to_string(&cruising).unwrap();
        assert_eq!(json, r#""cruising""#);

        let rth = FlightStatus::Rth;
        let json = serde_json::to_string(&rth).unwrap();
        assert_eq!(json, r#""rth""#);
    }

    #[test]
    fn test_drone_command_gimbal_control() {
        let cmd = DroneCommand::GimbalControl {
            pitch: -90.0,
            yaw: 45.0,
            roll: 0.0,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: DroneCommand = serde_json::from_str(&json).unwrap();
        match deserialized {
            DroneCommand::GimbalControl { pitch, yaw, roll } => {
                assert!((pitch + 90.0).abs() < 0.0001);
                assert!((yaw - 45.0).abs() < 0.0001);
                assert!((roll - 0.0).abs() < 0.0001);
            }
            _ => panic!("Expected GimbalControl command"),
        }
    }

    #[test]
    fn test_drone_command_camera_operations() {
        let capture = DroneCommand::CameraCapture;
        let json = serde_json::to_string(&capture).unwrap();
        assert!(json.contains("camera_capture"));

        let start_record = DroneCommand::CameraStartRecord;
        let json = serde_json::to_string(&start_record).unwrap();
        assert!(json.contains("camera_start_record"));

        let stop_record = DroneCommand::CameraStopRecord;
        let json = serde_json::to_string(&stop_record).unwrap();
        assert!(json.contains("camera_stop_record"));
    }

    #[test]
    fn test_drone_command_mission_operations() {
        let start = DroneCommand::StartMission { mission_id: 42 };
        let json = serde_json::to_string(&start).unwrap();
        assert!(json.contains("42"));

        let pause = DroneCommand::PauseMission;
        let _ = serde_json::to_string(&pause).unwrap();

        let resume = DroneCommand::ResumeMission;
        let _ = serde_json::to_string(&resume).unwrap();
    }

    #[test]
    fn test_drone_command_set_speed_and_altitude() {
        let speed = DroneCommand::SetSpeed { speed: 15.5 };
        let json = serde_json::to_string(&speed).unwrap();
        assert!(json.contains("15.5"));

        let altitude = DroneCommand::SetAltitude { altitude: 200.0 };
        let json = serde_json::to_string(&altitude).unwrap();
        assert!(json.contains("200"));
    }

    #[test]
    fn test_drone_command_rth_hover() {
        let rth = DroneCommand::Rth;
        let json = serde_json::to_string(&rth).unwrap();
        assert!(json.contains("rth"));

        let hover = DroneCommand::Hover;
        let json = serde_json::to_string(&hover).unwrap();
        assert!(json.contains("hover"));
    }
}
