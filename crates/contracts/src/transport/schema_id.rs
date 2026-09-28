//! Schema 平面标识和版本常量。

/// 四个 schema 平面。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaId {
    CommandRequest,
    ReceiptEvent,
    TelemetryDiagnostics,
    ParameterConfig,
}

impl SchemaId {
    pub fn all() -> [SchemaId; 4] {
        [
            SchemaId::CommandRequest,
            SchemaId::ReceiptEvent,
            SchemaId::TelemetryDiagnostics,
            SchemaId::ParameterConfig,
        ]
    }

    /// schema_version 字符串前缀，例如 "ranger.command_request"。
    pub fn version_prefix(&self) -> &'static str {
        match self {
            Self::CommandRequest => "ranger.command_request",
            Self::ReceiptEvent => "ranger.receipt_event",
            Self::TelemetryDiagnostics => "ranger.telemetry_diagnostics",
            Self::ParameterConfig => "ranger.parameter_config",
        }
    }
}

impl std::fmt::Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.version_prefix())
    }
}

/// schema 兼容判定结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityResult {
    /// 双方存在共同 version 且 hash 一致。
    Compatible,
    /// Client 版本 < Server 版本，但 Server 声明该 minor 向后兼容。
    CompatibleBackward,
    /// 严重版本不匹配。
    MajorMismatch,
    /// Client 只有更高版本。
    ClientNewer,
    /// 同 version 但 hash 不同。
    HashDrift,
    /// 没有任何共同 version。
    NoCommonVersion,
}

/// session 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Control,
    ReadOnly,
}

/// 连接状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectStatus {
    Ok,
    PeerNotAuthorized,
    SchemaIncompatible,
}

/// Server 错误原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerErrorCode {
    FrameTooLarge,
    FrameLengthMismatch,
    FrameTruncated,
    DecodeFailed,
    Unauthorized,
    InternalError,
}
