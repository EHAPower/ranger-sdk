//! v1 DiagnosticsQuery 和 DiagnosticsSnapshot DTO。

use serde::{Deserialize, Serialize};

/// 诊断查询请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsQuery {
    pub request_id: u64,
    pub paths: Vec<String>,
}

/// 诊断查询响应快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    pub request_id: u64,
    pub entries: Vec<DiagnosticsEntry>,
}

/// 单条诊断结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsEntry {
    pub path: String,
    pub value: DiagnosticsValue,
    pub timestamp_ns: u64,
    pub quality: DataQuality,
}

/// 诊断值类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticsValue {
    Bool(bool),
    Uint64(u64),
    Float64(f64),
    String(String),
    Object(Vec<(String, DiagnosticsValue)>),
}

/// 数据质量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQuality {
    Valid = 0,
    Stale = 1,
    Unavailable = 2,
    Error = 3,
}

/// Server 错误响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    pub code: ServerErrorCodeRepr,
    pub message: String,
}

/// Server 错误 code wire 表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerErrorCodeRepr {
    FrameTooLarge = 0,
    FrameLengthMismatch = 1,
    FrameTruncated = 2,
    DecodeFailed = 3,
    Unauthorized = 4,
    InternalError = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_query_roundtrip() {
        let query = DiagnosticsQuery {
            request_id: 1,
            paths: vec!["/vehicle/readiness".to_string(), "/safety".to_string()],
        };
        let bytes = bincode::serialize(&query).unwrap();
        let decoded: DiagnosticsQuery = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.paths.len(), 2);
    }

    #[test]
    fn diagnostics_snapshot_roundtrip() {
        let snap = DiagnosticsSnapshot {
            request_id: 1,
            entries: vec![DiagnosticsEntry {
                path: "/vehicle/readiness".to_string(),
                value: DiagnosticsValue::Bool(true),
                timestamp_ns: 1_000_000_000,
                quality: DataQuality::Valid,
            }],
        };
        let bytes = bincode::serialize(&snap).unwrap();
        let decoded: DiagnosticsSnapshot = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.entries.len(), 1);
    }
}
