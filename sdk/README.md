# Ranger Client SDK 内容

本目录保留从 `ranger-server` 提取的客户端接口候选快照：Rust package、schema 输入、文档入口和示例约束。它没有 Server implementation、profile checker 或硬件访问能力。

## 已实现的 Rust 范围

`rust/` 的 package `ranger-client` 通过同步 Linux Unix domain `SOCK_SEQPACKET` 连接 Ranger Server。它实现 control session handshake、motion、stop、heartbeat 和 EHA 域故障复位的 request/receipt transport。

当前公开 API 为 `ControlClient::connect`、`send_motion`、`heartbeat`、`stop` 和 `reset_eha_fault`：

```rust
use std::path::PathBuf;

use ranger_client::{ClientConfig, ControlClient, VehicleMotionCommand};

let mut client = ControlClient::connect(ClientConfig {
    socket_path: PathBuf::from("/run/ranger/server.sock"),
    max_frame_bytes: 65_536,
})?;
let receipt = client.send_motion(
    "yard-low-speed.v1",
    VehicleMotionCommand {
        linear_velocity_mps: 0.8,
        yaw_rate_radps: 0.15,
    },
)?;
println!("{}: {:?}", receipt.receipt_id, receipt.status);
# Ok::<(), ranger_client::ClientError>(())
```

`contracts` 含有 `Resource::VehicleSuspension`，但该 Client 还没有 `send_suspension` 方法，因此 suspension 不能列为当前 Rust SDK 的公开能力。

`Accepted` 只表示 Server gate 接受 request，不表示 backend executed、动作完成、stop confirmation 或车辆物理安全。调用方应读取 Server execution status 与 diagnostics；SDK 不自动重连、重发 mutating request 或恢复旧 motion。

当前 `ConnectRequest.schemas` 发送空集合。schema identity、YAML 输入和本地 DTO 都不能说明 runtime schema negotiation 已实现；Server 尚未提供该能力。完整 `ModeRequest`、统一 resource claim、整车 stop confirmation 也仍未实现。

## 边界

- SDK 只能使用 Ranger Server API，不打开 CAN、EtherCAT、S.Bus、MAVLink 或 backend。
- [`crates/contracts`](../crates/contracts/) 包含 DTO 与 RNG1 envelope。wire bytes 变更需要同时评估 Server、Ranger-Go 与发布组合。
- `schema/` 保留机器可读输入快照。迁移完成前，生产消费者仍以 `ranger-server` 中的同源输入为准，两个副本不得独立演进。
- `docs/` 和 `examples/` 只说明 SDK 使用边界，不能替代 Server 产品或安全规格。

开发、验证和迁移状态见仓库根目录的 `README.md` 与 `AGENTS.md`。
