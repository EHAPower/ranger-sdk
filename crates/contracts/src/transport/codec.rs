//! RFC 0002 wire payload 的唯一 Bincode 配置。

use bincode::Options;
use serde::de::DeserializeOwned;

/// 编码 payload。固定整数编码、小端序并拒绝 trailing bytes。
pub fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    options(u64::MAX).serialize(value)
}

/// 在给定字节上限内解码 payload。
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8], max_bytes: u64) -> Result<T, bincode::Error> {
    options(max_bytes).deserialize(bytes)
}

fn options(limit: u64) -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_little_endian()
        .reject_trailing_bytes()
        .with_limit(limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::session::Heartbeat;

    #[test]
    fn heartbeat_has_stable_golden_bytes() {
        let bytes = serialize(&Heartbeat {
            timestamp_ns: 0x0102_0304_0506_0708,
        })
        .unwrap();
        assert_eq!(bytes, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
        let decoded: Heartbeat = deserialize(&bytes, bytes.len() as u64).unwrap();
        assert_eq!(decoded.timestamp_ns, 0x0102_0304_0506_0708);
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = serialize(&Heartbeat { timestamp_ns: 1 }).unwrap();
        bytes.push(0);
        assert!(deserialize::<Heartbeat>(&bytes, bytes.len() as u64).is_err());
    }
}
