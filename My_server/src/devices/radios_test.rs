//! 对讲机模块单元测试

use super::radios::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_command_serialization() {
        let commands = vec![
            RadioCommand::SetChannel {
                channel: "CH1".to_string(),
            },
            RadioCommand::SetFrequency { frequency: 145.5 },
            RadioCommand::SetVolume { level: 8 },
            RadioCommand::Transmit {
                message: Some("Test message".to_string()),
            },
            RadioCommand::EmergencyCall,
            RadioCommand::GroupCall {
                group_id: "GROUP1".to_string(),
            },
            RadioCommand::PrivateCall {
                target_id: "RADIO001".to_string(),
            },
        ];

        for cmd in commands {
            let json = serde_json::to_string(&cmd).unwrap();
            let deserialized: RadioCommand = serde_json::from_str(&json).unwrap();
            assert_eq!(cmd, deserialized);
        }
    }

    #[test]
    fn test_radio_info_serialization() {
        let radio = RadioInfo {
            id: 1,
            name: "Test Radio".to_string(),
            serial_number: "SN123456".to_string(),
            vendor: RadioVendor::Motorola,
            model: "DP4800".to_string(),
            firmware_version: "1.0.0".to_string(),
            radio_id: "RADIO001".to_string(),
            mode: RadioMode::Digital,
            status: "online".to_string(),
            telemetry: None,
            created_at: chrono::Utc::now(),
            last_telemetry: None,
        };

        let json = serde_json::to_string(&radio).unwrap();
        let deserialized: RadioInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(radio.id, deserialized.id);
        assert_eq!(radio.name, deserialized.name);
        assert_eq!(radio.vendor, deserialized.vendor);
    }

    #[test]
    fn test_radio_telemetry_serialization() {
        let telemetry = RadioTelemetry {
            battery_percent: 85.5,
            battery_voltage: 7.4,
            signal_strength: -75.0,
            channel: "CH1".to_string(),
            frequency: 145.5,
            squelch_level: 3,
            volume_level: 8,
            radio_status: RadioStatus::Idle,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&telemetry).unwrap();
        let deserialized: RadioTelemetry = serde_json::from_str(&json).unwrap();
        assert_eq!(telemetry.battery_percent, deserialized.battery_percent);
        assert_eq!(telemetry.channel, deserialized.channel);
        assert_eq!(telemetry.radio_status, deserialized.radio_status);
    }

    #[test]
    fn test_radio_vendor_enum() {
        // Test enum serialization
        let vendor = RadioVendor::Motorola;
        let json = serde_json::to_string(&vendor).unwrap();
        assert_eq!(json, "\"motorola\"");

        let deserialized: RadioVendor = serde_json::from_str(&json).unwrap();
        assert_eq!(vendor, deserialized);
    }

    #[test]
    fn test_radio_mode_enum() {
        let mode = RadioMode::Digital;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"digital\"");

        let deserialized: RadioMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_radio_status_enum() {
        let status = RadioStatus::Transmitting;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"transmitting\"");

        let deserialized: RadioStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
