//! Additional Protocol Support Module
//!
//! Provides support for additional vehicle and IoT communication protocols:
//! - CAN Bus (Controller Area Network)
//! - OBD-II (On-Board Diagnostics)
//! - UDS (Unified Diagnostic Services)
//! - NMEA 2000 (Marine electronics protocol)
//! - SAE J1939 (Heavy-duty vehicle protocol)
//! - ISO 15765 (Road vehicles diagnostic communication)

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Additional protocol error types
#[derive(Error, Debug)]
pub enum AdditionalProtocolError {
    #[error("CAN Bus error: {0}")]
    CanBus(String),

    #[error("OBD-II error: {0}")]
    Obd(String),

    #[error("UDS error: {0}")]
    Uds(String),

    #[error("NMEA 2000 error: {0}")]
    Nmea2000(String),

    #[error("SAE J1939 error: {0}")]
    J1939(String),

    #[error("ISO 15765 error: {0}")]
    Iso15765(String),

    #[error("Protocol conversion error: {0}")]
    Conversion(String),
}

/// Additional protocol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdditionalProtocolType {
    CanBus,
    Obd,
    Uds,
    Nmea2000,
    J1939,
    Iso15765,
}

impl std::fmt::Display for AdditionalProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdditionalProtocolType::CanBus => write!(f, "CAN Bus"),
            AdditionalProtocolType::Obd => write!(f, "OBD-II"),
            AdditionalProtocolType::Uds => write!(f, "UDS"),
            AdditionalProtocolType::Nmea2000 => write!(f, "NMEA 2000"),
            AdditionalProtocolType::J1939 => write!(f, "SAE J1939"),
            AdditionalProtocolType::Iso15765 => write!(f, "ISO 15765"),
        }
    }
}

/// CAN Bus frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrame {
    pub id: u32,
    pub data: Vec<u8>,
    pub dlc: u8,
    pub rtr: bool,
    pub extended: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// OBD-II PID (Parameter ID)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ObdPid {
    EngineRpm = 0x0C,
    VehicleSpeed = 0x0D,
    ThrottlePosition = 0x11,
    EngineCoolantTemp = 0x05,
    IntakeAirTemp = 0x0F,
    MafAirFlowRate = 0x10,
    FuelLevel = 0x2F,
    DistanceTraveled = 0x31,
    EngineRuntime = 0x1F,
    DistanceWithMil = 0x21,
}

/// OBD-II message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObdMessage {
    pub mode: u8,
    pub pid: u8,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// UDS service ID
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UdsService {
    DiagnosticSessionControl = 0x10,
    EcuReset = 0x11,
    SecurityAccess = 0x27,
    CommunicationControl = 0x28,
    TesterPresent = 0x3E,
    AccessTimingParameter = 0x83,
    SecuredDataTransmission = 0x84,
    ControlDtcSetting = 0x85,
    ResponseOnEvent = 0x86,
    LinkControl = 0x87,
    ReadDataByIdentifier = 0x22,
    ReadMemoryByAddress = 0x23,
    ReadScalingDataByIdentifier = 0x24,
    ReadDataByPeriodicIdentifier = 0x2A,
    DynamicallyDefineDataIdentifier = 0x2C,
    WriteDataByIdentifier = 0x2E,
    WriteMemoryByAddress = 0x3D,
    ClearDiagnosticInformation = 0x14,
    ReadDtcInformation = 0x19,
}

/// UDS message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdsMessage {
    pub service: UdsService,
    pub sub_function: Option<u8>,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// NMEA 2000 message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nmea2000Message {
    pub pgn: u32, // Parameter Group Number
    pub priority: u8,
    pub source_address: u8,
    pub destination_address: u8,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SAE J1939 parameter group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct J1939ParameterGroup {
    pub pgn: u32,
    pub priority: u8,
    pub source_address: u8,
    pub destination_address: u8,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// ISO 15765 frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iso15765Frame {
    pub frame_type: Iso15765FrameType,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// ISO 15765 frame type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Iso15765FrameType {
    SingleFrame(u8),
    FirstFrame(u16),
    ConsecutiveFrame(u8),
    FlowControlFrame(u8, u8),
}

/// Additional protocol parser trait
#[async_trait]
pub trait AdditionalProtocolParser: Send + Sync {
    /// Parse message from bytes
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError>;

    /// Serialize message to bytes
    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError>;

    /// Get supported protocol type
    fn protocol_type(&self) -> AdditionalProtocolType;
}

/// CAN Bus protocol parser
pub struct CanBusParser {
    _bitrate: u32,
}

impl CanBusParser {
    pub fn new(bitrate: u32) -> Self {
        Self { _bitrate: bitrate }
    }
}

#[async_trait]
impl AdditionalProtocolParser for CanBusParser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.len() < 5 {
            return Err(AdditionalProtocolError::CanBus(
                "Frame too short".to_string(),
            ));
        }

        // Basic CAN frame parsing
        let _id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let dlc = data[4];

        if data.len() < 5 + dlc as usize {
            return Err(AdditionalProtocolError::CanBus("Invalid DLC".to_string()));
        }

        let payload = Bytes::copy_from_slice(&data[5..5 + dlc as usize]);
        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // Standard CAN ID (11-bit)
        result.extend_from_slice(&0x123u32.to_be_bytes());

        // DLC
        result.push(message.len() as u8);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::CanBus
    }
}

/// OBD-II protocol parser
#[allow(dead_code)]
pub struct ObdParser {
    ecu_address: u16,
}

impl ObdParser {
    pub fn new(ecu_address: u16) -> Self {
        Self { ecu_address }
    }

    /// Parse OBD-II PID
    pub fn parse_pid(&self, pid: u8) -> Option<String> {
        match pid {
            0x0C => Some("Engine RPM".to_string()),
            0x0D => Some("Vehicle Speed".to_string()),
            0x11 => Some("Throttle Position".to_string()),
            0x05 => Some("Engine Coolant Temperature".to_string()),
            0x0F => Some("Intake Air Temperature".to_string()),
            0x10 => Some("MAF Air Flow Rate".to_string()),
            0x2F => Some("Fuel Level".to_string()),
            0x31 => Some("Distance Traveled".to_string()),
            0x1F => Some("Engine Runtime".to_string()),
            0x21 => Some("Distance with MIL".to_string()),
            _ => None,
        }
    }
}

#[async_trait]
impl AdditionalProtocolParser for ObdParser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.len() < 3 {
            return Err(AdditionalProtocolError::Obd(
                "Message too short".to_string(),
            ));
        }

        let _mode = data[0];
        let pid = data[1];

        // Parse OBD-II data based on PID
        let payload = match pid {
            0x0C => {
                // Engine RPM: (A*256 + B) / 4
                if data.len() >= 4 {
                    let rpm = ((data[2] as u16) * 256 + data[3] as u16) / 4;
                    Bytes::copy_from_slice(&rpm.to_be_bytes())
                } else {
                    Bytes::new()
                }
            }
            0x0D => {
                // Vehicle Speed: A km/h
                if data.len() >= 3 {
                    Bytes::copy_from_slice(&data[2..3])
                } else {
                    Bytes::new()
                }
            }
            _ => Bytes::copy_from_slice(&data[2..]),
        };

        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // Mode 1: Request current powertrain diagnostic data
        result.push(0x01);

        // PID (Engine RPM)
        result.push(0x0C);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::Obd
    }
}

/// UDS protocol parser
#[allow(dead_code)]
pub struct UdsParser {
    tester_address: u16,
}

impl UdsParser {
    pub fn new(tester_address: u16) -> Self {
        Self { tester_address }
    }
}

#[async_trait]
impl AdditionalProtocolParser for UdsParser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.is_empty() {
            return Err(AdditionalProtocolError::Uds("Empty message".to_string()));
        }

        let service_id = data[0];

        // Parse UDS service
        let _service = match service_id {
            0x10 => UdsService::DiagnosticSessionControl,
            0x11 => UdsService::EcuReset,
            0x27 => UdsService::SecurityAccess,
            0x22 => UdsService::ReadDataByIdentifier,
            0x2E => UdsService::WriteDataByIdentifier,
            0x19 => UdsService::ReadDtcInformation,
            _ => {
                return Err(AdditionalProtocolError::Uds(format!(
                    "Unknown service: 0x{:02X}",
                    service_id
                )))
            }
        };

        // Return payload (service-specific data)
        let payload = if data.len() > 1 {
            Bytes::copy_from_slice(&data[1..])
        } else {
            Bytes::new()
        };

        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // Service ID (Read Data By Identifier)
        result.push(0x22);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::Uds
    }
}

/// NMEA 2000 protocol parser
pub struct Nmea2000Parser {
    device_address: u8,
}

impl Nmea2000Parser {
    pub fn new(device_address: u8) -> Self {
        Self { device_address }
    }
}

#[async_trait]
impl AdditionalProtocolParser for Nmea2000Parser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.len() < 8 {
            return Err(AdditionalProtocolError::Nmea2000(
                "Message too short".to_string(),
            ));
        }

        // Parse NMEA 2000 message
        let _pgn = u32::from_be_bytes([0, data[0], data[1], data[2]]);
        let _priority = data[3];
        let _source_address = data[4];
        let _destination_address = data[5];

        // Return payload
        let payload = Bytes::copy_from_slice(&data[6..]);

        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // PGN (Parameter Group Number)
        result.extend_from_slice(&0x1F801u32.to_be_bytes()[1..]);

        // Priority
        result.push(0x02);

        // Source address
        result.push(self.device_address);

        // Destination address (global)
        result.push(0xFF);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::Nmea2000
    }
}

/// SAE J1939 protocol parser
pub struct J1939Parser {
    source_address: u8,
}

impl J1939Parser {
    pub fn new(source_address: u8) -> Self {
        Self { source_address }
    }
}

#[async_trait]
impl AdditionalProtocolParser for J1939Parser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.len() < 8 {
            return Err(AdditionalProtocolError::J1939(
                "Message too short".to_string(),
            ));
        }

        // Parse J1939 parameter group
        let _pgn = u32::from_be_bytes([0, data[0], data[1], data[2]]);
        let _priority = data[3];
        let _source_address = data[4];
        let _destination_address = data[5];

        // Return payload
        let payload = Bytes::copy_from_slice(&data[6..]);

        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // PGN (Parameter Group Number)
        result.extend_from_slice(&0x00F004u32.to_be_bytes()[1..]);

        // Priority
        result.push(0x03);

        // Source address
        result.push(self.source_address);

        // Destination address (global)
        result.push(0xFF);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::J1939
    }
}

/// ISO 15765 protocol parser
#[allow(dead_code)]
pub struct Iso15765Parser {
    block_size: u8,
    separation_time: u8,
}

impl Iso15765Parser {
    pub fn new(block_size: u8, separation_time: u8) -> Self {
        Self {
            block_size,
            separation_time,
        }
    }
}

#[async_trait]
impl AdditionalProtocolParser for Iso15765Parser {
    async fn parse(&self, data: &[u8]) -> Result<Bytes, AdditionalProtocolError> {
        if data.is_empty() {
            return Err(AdditionalProtocolError::Iso15765("Empty frame".to_string()));
        }

        let frame_type_byte = data[0];
        let _frame_type = match (frame_type_byte >> 4) & 0x0F {
            0x00 => Iso15765FrameType::SingleFrame(frame_type_byte & 0x0F),
            0x01 => {
                // First frame
                if data.len() < 2 {
                    return Err(AdditionalProtocolError::Iso15765(
                        "Invalid first frame".to_string(),
                    ));
                }
                let length = ((frame_type_byte & 0x0F) as u16) << 8 | data[1] as u16;
                Iso15765FrameType::FirstFrame(length)
            }
            0x02 => Iso15765FrameType::ConsecutiveFrame(frame_type_byte & 0x0F),
            0x03 => {
                // Flow control frame
                if data.len() < 3 {
                    return Err(AdditionalProtocolError::Iso15765(
                        "Invalid flow control frame".to_string(),
                    ));
                }
                let block_size = data[1];
                let separation_time = data[2];
                Iso15765FrameType::FlowControlFrame(block_size, separation_time)
            }
            _ => {
                return Err(AdditionalProtocolError::Iso15765(
                    "Unknown frame type".to_string(),
                ))
            }
        };

        // Return payload
        let payload = if data.len() > 1 {
            Bytes::copy_from_slice(&data[1..])
        } else {
            Bytes::new()
        };

        Ok(payload)
    }

    async fn serialize(&self, message: &Bytes) -> Result<Vec<u8>, AdditionalProtocolError> {
        let mut result = Vec::new();

        // Single frame
        result.push(message.len() as u8 & 0x0F);

        // Data
        result.extend_from_slice(message);

        Ok(result)
    }

    fn protocol_type(&self) -> AdditionalProtocolType {
        AdditionalProtocolType::Iso15765
    }
}

/// Additional protocol manager that coordinates all additional protocol parsers
pub struct AdditionalProtocolManager {
    parsers: Arc<RwLock<HashMap<AdditionalProtocolType, Arc<dyn AdditionalProtocolParser>>>>,
    can_parser: Arc<CanBusParser>,
    obd_parser: Arc<ObdParser>,
    uds_parser: Arc<UdsParser>,
    nmea_parser: Arc<Nmea2000Parser>,
    j1939_parser: Arc<J1939Parser>,
    iso15765_parser: Arc<Iso15765Parser>,
}

impl AdditionalProtocolManager {
    /// Create a new additional protocol manager
    pub async fn new() -> Result<Self, AdditionalProtocolError> {
        let can_parser = Arc::new(CanBusParser::new(500_000)); // 500 kbps
        let obd_parser = Arc::new(ObdParser::new(0x7E8)); // ECU address - use u16
        let uds_parser = Arc::new(UdsParser::new(0x7E0)); // Tester address - use u16
        let nmea_parser = Arc::new(Nmea2000Parser::new(0x01)); // Device address
        let j1939_parser = Arc::new(J1939Parser::new(0x81)); // Source address
        let iso15765_parser = Arc::new(Iso15765Parser::new(8, 0)); // Block size 8, no separation time

        let mut parsers: HashMap<AdditionalProtocolType, Arc<dyn AdditionalProtocolParser>> =
            HashMap::new();

        parsers.insert(AdditionalProtocolType::CanBus, can_parser.clone());
        parsers.insert(AdditionalProtocolType::Obd, obd_parser.clone());
        parsers.insert(AdditionalProtocolType::Uds, uds_parser.clone());
        parsers.insert(AdditionalProtocolType::Nmea2000, nmea_parser.clone());
        parsers.insert(AdditionalProtocolType::J1939, j1939_parser.clone());
        parsers.insert(AdditionalProtocolType::Iso15765, iso15765_parser.clone());

        Ok(Self {
            parsers: Arc::new(RwLock::new(parsers)),
            can_parser,
            obd_parser,
            uds_parser,
            nmea_parser,
            j1939_parser,
            iso15765_parser,
        })
    }

    /// Parse CAN Bus frame
    pub async fn parse_can_frame(&self, data: &[u8]) -> Result<CanFrame, AdditionalProtocolError> {
        let payload = self.can_parser.parse(data).await?;

        Ok(CanFrame {
            id: 0x123, // Default ID
            data: payload.to_vec(),
            dlc: payload.len() as u8,
            rtr: false,
            extended: false,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse OBD-II message
    pub async fn parse_obd_message(
        &self,
        data: &[u8],
    ) -> Result<ObdMessage, AdditionalProtocolError> {
        let payload = self.obd_parser.parse(data).await?;

        Ok(ObdMessage {
            mode: data[0],
            pid: data[1],
            data: payload.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse UDS message
    pub async fn parse_uds_message(
        &self,
        data: &[u8],
    ) -> Result<UdsMessage, AdditionalProtocolError> {
        let payload = self.uds_parser.parse(data).await?;

        let service = match data[0] {
            0x10 => UdsService::DiagnosticSessionControl,
            0x11 => UdsService::EcuReset,
            0x27 => UdsService::SecurityAccess,
            0x22 => UdsService::ReadDataByIdentifier,
            0x2E => UdsService::WriteDataByIdentifier,
            0x19 => UdsService::ReadDtcInformation,
            _ => UdsService::DiagnosticSessionControl, // Default
        };

        Ok(UdsMessage {
            service,
            sub_function: if data.len() > 1 { Some(data[1]) } else { None },
            data: payload.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse NMEA 2000 message
    pub async fn parse_nmea2000_message(
        &self,
        data: &[u8],
    ) -> Result<Nmea2000Message, AdditionalProtocolError> {
        let payload = self.nmea_parser.parse(data).await?;

        Ok(Nmea2000Message {
            pgn: 0x1F801, // Default PGN
            priority: 2,
            source_address: 0x01,
            destination_address: 0xFF, // Global
            data: payload.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse SAE J1939 message
    pub async fn parse_j1939_message(
        &self,
        data: &[u8],
    ) -> Result<J1939ParameterGroup, AdditionalProtocolError> {
        let payload = self.j1939_parser.parse(data).await?;

        Ok(J1939ParameterGroup {
            pgn: 0x00F004, // Default PGN
            priority: 3,
            source_address: 0x81,
            destination_address: 0xFF, // Global
            data: payload.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse ISO 15765 frame
    pub async fn parse_iso15765_frame(
        &self,
        data: &[u8],
    ) -> Result<Iso15765Frame, AdditionalProtocolError> {
        let payload = self.iso15765_parser.parse(data).await?;

        let frame_type = if data.is_empty() {
            Iso15765FrameType::SingleFrame(0)
        } else {
            match (data[0] >> 4) & 0x0F {
                0x00 => Iso15765FrameType::SingleFrame(data[0] & 0x0F),
                0x01 => Iso15765FrameType::FirstFrame(((data[0] & 0x0F) as u16) << 8),
                0x02 => Iso15765FrameType::ConsecutiveFrame(data[0] & 0x0F),
                0x03 => Iso15765FrameType::FlowControlFrame(0, 0),
                _ => Iso15765FrameType::SingleFrame(0),
            }
        };

        Ok(Iso15765Frame {
            frame_type,
            data: payload.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Add a new protocol parser
    pub async fn add_parser(
        &self,
        parser: Arc<dyn AdditionalProtocolParser>,
    ) -> Result<(), AdditionalProtocolError> {
        let mut parsers = self.parsers.write().await;
        parsers.insert(parser.protocol_type(), parser);
        Ok(())
    }

    /// Remove a protocol parser
    pub async fn remove_parser(
        &self,
        protocol_type: &AdditionalProtocolType,
    ) -> Result<(), AdditionalProtocolError> {
        let mut parsers = self.parsers.write().await;
        parsers.remove(protocol_type);
        Ok(())
    }

    /// Get list of supported protocols
    pub async fn get_supported_protocols(&self) -> Vec<AdditionalProtocolType> {
        let parsers = self.parsers.read().await;
        parsers.keys().cloned().collect()
    }

    /// Convert between protocol formats
    pub async fn convert_protocol(
        &self,
        from_type: AdditionalProtocolType,
        to_type: AdditionalProtocolType,
        data: &[u8],
    ) -> Result<Vec<u8>, AdditionalProtocolError> {
        match (from_type, to_type) {
            (AdditionalProtocolType::Obd, AdditionalProtocolType::Uds) => {
                // Convert OBD-II to UDS
                let obd_msg = self.parse_obd_message(data).await?;
                let uds_data = Bytes::from(obd_msg.data);
                self.uds_parser.serialize(&uds_data).await
            }
            (AdditionalProtocolType::CanBus, AdditionalProtocolType::J1939) => {
                // Convert CAN Bus to J1939
                let can_frame = self.parse_can_frame(data).await?;
                let j1939_data = Bytes::from(can_frame.data);
                self.j1939_parser.serialize(&j1939_data).await
            }
            _ => Err(AdditionalProtocolError::Conversion(
                "Unsupported protocol conversion".to_string(),
            )),
        }
    }
}
