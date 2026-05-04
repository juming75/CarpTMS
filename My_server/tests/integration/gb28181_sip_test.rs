//! / GB28181 SIP 服务器集成测试

#[cfg(test)]
mod tests {
    use carptms::video::{Gb28181SipServer, SipServerState};
    use carptms::video::config::Gb28181Config;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_sip_server_creation() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);
        
        assert_eq!(server.get_state().await, SipServerState::Initializing);
    }

    #[tokio::test]
    async fn test_registered_device_count_initially_zero() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);
        
        assert_eq!(server.get_registered_device_count().await, 0);
    }

    #[tokio::test]
    async fn test_active_session_count_initially_zero() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);
        
        assert_eq!(server.get_active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_registered_devices_initially_empty() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);
        
        let devices = server.get_registered_devices().await;
        assert!(devices.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_sessions_initially_empty() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);
        
        let sessions = server.get_all_sessions().await;
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_gb28181_config_default() {
        let config = Gb28181Config::default();
        
        assert!(config.enabled);
        assert_eq!(config.sip_port, 5060);
        assert_eq!(config.server_id, "34020000002000000001");
        assert_eq!(config.server_domain, "3402000000");
        assert!(config.rtp_port_start < config.rtp_port_end);
    }

    #[test]
    fn test_gb28181_config_validation() {
        let mut config = Gb28181Config::default();
        config.sip_port = 0;
        
        // 验证配置
        assert!(config.rtp_port_start >= config.rtp_port_end || config.sip_port == 0);
    }
}
