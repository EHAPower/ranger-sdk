//! Bootstrap 消息：ConnectRequest 和 ConnectResponse。
//!
//! 这两个消息的 payload 格式固定在 wire protocol version 内，
//! 由本模块硬编码，不从 `sdk/schema/*.yaml` 生成。
//! 字段变更需要 wp_ver 递增。

use serde::{Deserialize, Serialize};

use super::envelope::{Envelope, MsgKind};
use super::schema_id::{CompatibilityResult, ConnectStatus, SchemaId};

/// 集合/字符串容量上限，防止恶意长度触发超量预分配。
pub const MAX_SCHEMA_ENTRIES: usize = 16;
pub const MAX_VERSIONS_PER_SCHEMA: usize = 8;
pub const MAX_SDK_VERSION_LEN: usize = 64;
pub const MAX_VERSION_STRING_LEN: usize = 64;
pub const MAX_HASH_STRING_LEN: usize = 128;

// ── Client → Server ──

/// 客户端连接请求，携带 schema manifest。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub sdk_version: String,
    pub schemas: Vec<ClientSchemaSupport>,
}

/// 客户端对某个 schema 平面支持的版本集合。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSchemaSupport {
    pub schema_id: SchemaIdRepr,
    pub supported_versions: Vec<SchemaVersionHash>,
}

/// Schema ID 的 wire 表示（u8 枚举）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaIdRepr {
    CommandRequest = 0,
    ReceiptEvent = 1,
    TelemetryDiagnostics = 2,
    ParameterConfig = 3,
}

impl From<SchemaId> for SchemaIdRepr {
    fn from(id: SchemaId) -> Self {
        match id {
            SchemaId::CommandRequest => Self::CommandRequest,
            SchemaId::ReceiptEvent => Self::ReceiptEvent,
            SchemaId::TelemetryDiagnostics => Self::TelemetryDiagnostics,
            SchemaId::ParameterConfig => Self::ParameterConfig,
        }
    }
}

impl TryFrom<SchemaIdRepr> for SchemaId {
    type Error = ();

    fn try_from(repr: SchemaIdRepr) -> Result<Self, Self::Error> {
        match repr {
            SchemaIdRepr::CommandRequest => Ok(SchemaId::CommandRequest),
            SchemaIdRepr::ReceiptEvent => Ok(SchemaId::ReceiptEvent),
            SchemaIdRepr::TelemetryDiagnostics => Ok(SchemaId::TelemetryDiagnostics),
            SchemaIdRepr::ParameterConfig => Ok(SchemaId::ParameterConfig),
        }
    }
}

/// 客户端声明的单个 schema version + hash。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionHash {
    pub version: String,
    pub hash: String,
}

// ── Server → Client ──

/// 连接响应，包含 schema 协商结果和允许的 session 类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub server_version: String,
    pub schemas: Vec<SchemaNegotiation>,
    pub allowed_session_types: Vec<SessionTypeRepr>,
    pub status: ConnectStatusRepr,
}

/// Schema 协商结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaNegotiation {
    pub schema_id: SchemaIdRepr,
    pub selected_version: Option<String>,
    pub selected_hash: Option<String>,
    pub compatibility: CompatibilityResultRepr,
}

/// Session 类型的 wire 表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionTypeRepr {
    Control = 0,
    ReadOnly = 1,
}

/// ConnectStatus 的 wire 表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectStatusRepr {
    Ok = 0,
    PeerNotAuthorized = 1,
    SchemaIncompatible = 2,
}

impl From<ConnectStatus> for ConnectStatusRepr {
    fn from(s: ConnectStatus) -> Self {
        match s {
            ConnectStatus::Ok => Self::Ok,
            ConnectStatus::PeerNotAuthorized => Self::PeerNotAuthorized,
            ConnectStatus::SchemaIncompatible => Self::SchemaIncompatible,
        }
    }
}

/// CompatibilityResult 的 wire 表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityResultRepr {
    Compatible = 0,
    CompatibleBackward = 1,
    MajorMismatch = 2,
    ClientNewer = 3,
    HashDrift = 4,
    NoCommonVersion = 5,
}

impl From<CompatibilityResult> for CompatibilityResultRepr {
    fn from(r: CompatibilityResult) -> Self {
        match r {
            CompatibilityResult::Compatible => Self::Compatible,
            CompatibilityResult::CompatibleBackward => Self::CompatibleBackward,
            CompatibilityResult::MajorMismatch => Self::MajorMismatch,
            CompatibilityResult::ClientNewer => Self::ClientNewer,
            CompatibilityResult::HashDrift => Self::HashDrift,
            CompatibilityResult::NoCommonVersion => Self::NoCommonVersion,
        }
    }
}

/// Bootstrap 消息编解码。
impl ConnectRequest {
    pub fn encode_envelope(&self) -> Result<Envelope, String> {
        validate_connect_request(self)?;
        Envelope::with_bincode_payload(MsgKind::ConnectRequest, self)
            .map_err(|e| format!("ConnectRequest 编码失败：{e}"))
    }

    pub fn decode_payload(payload: &[u8], max_bytes: u64) -> Result<Self, String> {
        let request = super::codec::deserialize(payload, max_bytes)
            .map_err(|e| format!("ConnectRequest 解码失败：{e}"))?;
        validate_connect_request(&request)?;
        Ok(request)
    }
}

impl ConnectResponse {
    pub fn encode_envelope(&self) -> Result<Envelope, String> {
        Envelope::with_bincode_payload(MsgKind::ConnectResponse, self)
            .map_err(|e| format!("ConnectResponse 编码失败：{e}"))
    }
}

/// 校验 ConnectRequest 的容量限制。
pub fn validate_connect_request(req: &ConnectRequest) -> Result<(), String> {
    if req.sdk_version.len() > MAX_SDK_VERSION_LEN {
        return Err(format!(
            "sdk_version 过长：{} > {}",
            req.sdk_version.len(),
            MAX_SDK_VERSION_LEN
        ));
    }
    if req.schemas.len() > MAX_SCHEMA_ENTRIES {
        return Err(format!(
            "schemas 过多：{} > {}",
            req.schemas.len(),
            MAX_SCHEMA_ENTRIES
        ));
    }
    for entry in &req.schemas {
        if entry.supported_versions.len() > MAX_VERSIONS_PER_SCHEMA {
            return Err(format!(
                "schema {:?} 的版本数过多：{} > {}",
                entry.schema_id,
                entry.supported_versions.len(),
                MAX_VERSIONS_PER_SCHEMA
            ));
        }
        for v in &entry.supported_versions {
            if v.version.len() > MAX_VERSION_STRING_LEN {
                return Err(format!("version 字符串过长：{}", v.version.len()));
            }
            if v.hash.len() > MAX_HASH_STRING_LEN {
                return Err(format!("hash 字符串过长：{}", v.hash.len()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_connect_request() -> ConnectRequest {
        ConnectRequest {
            sdk_version: "0.1.0".to_string(),
            schemas: vec![ClientSchemaSupport {
                schema_id: SchemaIdRepr::CommandRequest,
                supported_versions: vec![SchemaVersionHash {
                    version: "ranger.command_request.v1".to_string(),
                    hash: "sha256:abc123".to_string(),
                }],
            }],
        }
    }

    #[test]
    fn connect_request_roundtrip() {
        let req = sample_connect_request();
        let env = req.encode_envelope().unwrap();
        assert_eq!(env.msg_kind, MsgKind::ConnectRequest);

        let decoded = ConnectRequest::decode_payload(&env.payload, 65_536).unwrap();
        assert_eq!(decoded.sdk_version, "0.1.0");
        assert_eq!(decoded.schemas.len(), 1);
    }

    #[test]
    fn reject_sdk_version_too_long() {
        let mut req = sample_connect_request();
        req.sdk_version = "x".repeat(MAX_SDK_VERSION_LEN + 1);
        assert!(validate_connect_request(&req).is_err());
    }

    #[test]
    fn reject_too_many_schemas() {
        let mut req = sample_connect_request();
        req.schemas = (0..=MAX_SCHEMA_ENTRIES)
            .map(|_| ClientSchemaSupport {
                schema_id: SchemaIdRepr::CommandRequest,
                supported_versions: vec![],
            })
            .collect();
        assert!(validate_connect_request(&req).is_err());
    }

    #[test]
    fn connect_response_roundtrip() {
        let resp = ConnectResponse {
            server_version: "0.1.0".to_string(),
            schemas: vec![SchemaNegotiation {
                schema_id: SchemaIdRepr::CommandRequest,
                selected_version: Some("ranger.command_request.v1".to_string()),
                selected_hash: Some("sha256:abc123".to_string()),
                compatibility: CompatibilityResultRepr::Compatible,
            }],
            allowed_session_types: vec![SessionTypeRepr::Control],
            status: ConnectStatusRepr::Ok,
        };
        let env = resp.encode_envelope().unwrap();
        assert_eq!(env.msg_kind, MsgKind::ConnectResponse);

        let decoded: ConnectResponse =
            crate::transport::codec::deserialize(&env.payload, 65_536).unwrap();
        assert_eq!(decoded.server_version, "0.1.0");
    }

    #[test]
    fn full_envelope_roundtrip() {
        let req = sample_connect_request();
        let env = req.encode_envelope().unwrap();
        let wire = env.encode();

        let decoded_env = Envelope::decode(&wire, 65536).unwrap();
        assert_eq!(decoded_env.msg_kind, MsgKind::ConnectRequest);

        let decoded_req = ConnectRequest::decode_payload(&decoded_env.payload, 65_536).unwrap();
        assert_eq!(decoded_req.sdk_version, req.sdk_version);
    }
}
