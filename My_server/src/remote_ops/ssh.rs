//! SSH 连接管理模块
//!
//! 使用 russh 库实现 SSH 连接、认证、命令执行、实时输出流

use async_trait::async_trait;
use chrono::Utc;
use russh::client::{self, Handler, Msg};
use russh::{Channel, ChannelMsg, Disconnect};
use russh_keys::key::PublicKey;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
use uuid::Uuid;

use crate::errors::AppError;
use crate::remote_ops::models::{
    CommandResult, CommandStatus, ExecuteCommandRequest, ServerMetrics,
};

// ═══════════ SSH 客户端 Handler ═══════════

#[derive(Clone)]
pub struct SshClientHandler {
    pub server_id: Uuid,
    pub session_id: Uuid,
    pub output_tx: Option<mpsc::UnboundedSender<String>>,
}

impl Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(self, _server_public_key: &PublicKey) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }

    async fn data(self, _channel: ChannelId, data: &[u8], _session: &mut client::Session) -> Result<(Self, Self::Error)> {
        if let Some(ref tx) = self.output_tx {
            let _ = tx.send(String::from_utf8_lossy(data).to_string());
        }
        Ok((self, russh::Error::Disconnect))
    }

    async fn extended_data(self, _channel: ChannelId, _ext: u32, data: &[u8], _session: &mut client::Session) -> Result<(Self, Self::Error)> {
        if let Some(ref tx) = self.output_tx {
            let _ = tx.send(String::from_utf8_lossy(data).to_string());
        }
        Ok((self, russh::Error::Disconnect))
    }
}

// ═══════════ SSH 配置 ═══════════

#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub timeout_secs: u64,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: "root".to_string(),
            password: None,
            private_key: None,
            private_key_passphrase: None,
            timeout_secs: 30,
        }
    }
}

// ═══════════ SSH 连接 ═══════════

pub struct SshConnection {
    pub session_id: Uuid,
    pub server_id: Uuid,
    pub config: SshConfig,
    pub created_at: chrono::DateTime<Utc>,
}

fn internal_err(msg: String) -> AppError {
    AppError::internal_error(&msg, None)
}

impl SshConnection {
    /// 建立 SSH 连接并执行命令
    pub async fn execute_command(
        &self,
        command: &str,
        timeout_secs: u64,
    ) -> Result<(String, String, Option<i32>), AppError> {
        let config = Arc::new(client::Config::default());
        let shost = self.config.host.clone();
        let sport = self.config.port;

        let session_id = Uuid::new_v4();
        let handler = SshClientHandler {
            server_id: self.server_id,
            session_id,
            output_tx: None,
        };

        let mut session = client::connect(config, (shost.as_str(), sport), handler)
            .await
            .map_err(|e| internal_err(format!("SSH连接失败: {}", e)))?;

        if let Some(password) = &self.config.password {
            session
                .authenticate_password(self.config.username.as_str(), password.as_str())
                .await
                .map_err(|e| internal_err(format!("SSH密码认证失败: {}", e)))?;
        } else if let Some(key) = &self.config.private_key {
            let key_pair = russh_keys::load_secret_key(key.as_bytes(), self.config.private_key_passphrase.as_deref())
                .map_err(|e| internal_err(format!("SSH密钥加载失败: {}", e)))?;
            session
                .authenticate_publickey(self.config.username.as_str(), Arc::new(key_pair))
                .await
                .map_err(|e| internal_err(format!("SSH密钥认证失败: {}", e)))?;
        } else {
            return Err(internal_err("SSH认证信息不完整".to_string()));
        }

        let mut channel = session
            .channel_open_session()
            .await
            .map_err(|e| internal_err(format!("SSH channel打开失败: {}", e)))?;

        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| internal_err(format!("命令执行失败: {}", e)))?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut exit_code: Option<i32> = None;

        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            stdout.push_str(&String::from_utf8_lossy(data));
                        }
                        Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                            stderr.push_str(&String::from_utf8_lossy(data));
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            exit_code = Some(exit_status);
                            break;
                        }
                        Some(ChannelMsg::Eof) => break,
                        None => break,
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => break,
            }
        }

        let _ = channel.eof().await;
        let _ = channel.close().await;
        let _ = session.disconnect(Disconnect::Bye, "done", "en").await;

        Ok((stdout, stderr, exit_code))
    }

    /// 带实时输出的命令执行
    pub async fn execute_command_with_output(
        &self,
        command: &str,
        timeout_secs: u64,
        output_tx: mpsc::UnboundedSender<String>,
    ) -> Result<i32, AppError> {
        let config = Arc::new(client::Config::default());
        let shost = self.config.host.clone();
        let sport = self.config.port;
        let session_id = Uuid::new_v4();

        let handler = SshClientHandler {
            server_id: self.server_id,
            session_id,
            output_tx: Some(output_tx.clone()),
        };

        let mut session = client::connect(config, (shost.as_str(), sport), handler)
            .await
            .map_err(|e| internal_err(format!("SSH连接失败: {}", e)))?;

        if let Some(password) = &self.config.password {
            session
                .authenticate_password(self.config.username.as_str(), password.as_str())
                .await
                .map_err(|e| internal_err(format!("SSH认证失败: {}", e)))?;
        } else if let Some(key) = &self.config.private_key {
            let key_pair = russh_keys::load_secret_key(key.as_bytes(), self.config.private_key_passphrase.as_deref())
                .map_err(|e| internal_err(format!("SSH密钥加载失败: {}", e)))?;
            session
                .authenticate_publickey(self.config.username.as_str(), Arc::new(key_pair))
                .await
                .map_err(|e| internal_err(format!("SSH密钥认证失败: {}", e)))?;
        } else {
            return Err(internal_err("SSH认证信息不完整".to_string()));
        }

        let mut channel = session.channel_open_session().await
            .map_err(|e| internal_err(format!("SSH channel打开失败: {}", e)))?;

        channel.exec(true, command.as_bytes()).await
            .map_err(|e| internal_err(format!("命令执行失败: {}", e)))?;

        let mut exit_code = -1;
        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            let _ = output_tx.send(String::from_utf8_lossy(data).to_string());
                        }
                        Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                            let _ = output_tx.send(String::from_utf8_lossy(data).to_string());
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            exit_code = exit_status;
                            break;
                        }
                        _ => break,
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => break,
            }
        }

        let _ = session.disconnect(Disconnect::Bye, "done", "en").await;
        Ok(exit_code)
    }
}

// ═══════════ SSH 连接管理器 ═══════════

#[derive(Clone)]
pub struct SshConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, SshConnection>>>,
}

impl SshConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, server_id: Uuid, config: SshConfig) -> Result<SshConnection, AppError> {
        let session_id = Uuid::new_v4();
        let connection = SshConnection {
            session_id,
            server_id,
            config: config.clone(),
            created_at: Utc::now(),
        };

        let (stdout, stderr, exit_code) = connection.execute_command("echo 'connected'", 10).await?;
        if exit_code != Some(0) {
            return Err(internal_err(format!("SSH连接测试失败: {}", stderr)));
        }

        let mut conns = self.connections.write().await;
        conns.insert(server_id, connection);
        let conn = conns.get(&server_id).unwrap();
        Ok(SshConnection {
            session_id: conn.session_id,
            server_id: conn.server_id,
            config: conn.config.clone(),
            created_at: conn.created_at,
        })
    }

    pub async fn get_or_connect(&self, server_id: Uuid, config: SshConfig) -> Result<SshConnection, AppError> {
        let conns = self.connections.read().await;
        if let Some(conn) = conns.get(&server_id) {
            return Ok(SshConnection {
                session_id: conn.session_id,
                server_id: conn.server_id,
                config: conn.config.clone(),
                created_at: conn.created_at,
            });
        }
        drop(conns);
        self.connect(server_id, config).await
    }

    pub async fn disconnect(&self, server_id: Uuid) {
        let mut conns = self.connections.write().await;
        conns.remove(&server_id);
    }

    pub async fn execute(
        &self,
        server_id: Uuid,
        config: SshConfig,
        command: &str,
        timeout_secs: u64,
    ) -> Result<(String, String, Option<i32>), AppError> {
        let conn = self.get_or_connect(server_id, config).await?;
        conn.execute_command(command, timeout_secs).await
    }
}

impl Default for SshConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
