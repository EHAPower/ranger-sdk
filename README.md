# Ranger SDK

`ranger-sdk` 是 Ranger 的客户端接口仓库。当前内容从 `ranger-server` 的 `a691ce1a999314eae5634876786de182915509cb` 提取，包含 Rust Client、共享 `contracts` 和机器可读 schema 输入。它不包含 Ranger Server、硬件 backend、控制算法或部署配置。

这是一次接口提取的发布候选快照，尚不是协议 owner 已迁移完成的证明。迁移期间，`ranger-server` 中的 `crates/contracts` 仍是当前生产消费者使用的来源；在 Ranger-Go、Server 和发布流程切换到此仓固定版本前，不得在两个位置独立演进 DTO、schema 或 wire bytes。任何协议改动必须先确定唯一 owner、迁移顺序、兼容策略和验证组合。

## 当前可用接口

- `sdk/rust` 的 `ranger-client`：同步 Linux Unix domain `SOCK_SEQPACKET` Client。
- `crates/contracts`：RNG1 transport envelope、DTO 和 schema 身份类型。
- `sdk/schema`：schema 源输入快照，包括九个对外 schema 和 checker registry metadata。

当前 `ranger-client` 的公开方法是 `ControlClient::connect`、`send_motion`、
`heartbeat`、`stop` 和 `reset_eha_fault`。`contracts` 已定义
`Resource::VehicleSuspension`，但 Rust Client 还没有 `send_suspension` 方法；
不得把 DTO 枚举当成已交付的 suspension SDK API。

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

Client 只通过 Ranger Server API 请求 control session、motion、stop 和 EHA 域故障复位；它不会打开 CAN、EtherCAT、S.Bus、MAVLink 或任何 backend。`contracts` 保留 diagnostics DTO，但当前 Rust Client 尚未提供 ReadOnly/diagnostics client。当前 `ConnectRequest.schemas` 发送空集合，Server 尚未实现 schema negotiation；schema 文件与 `SchemaId` 的存在不能说明运行时已协商兼容。

`GateReceipt::Accepted` 仅表示 Server gate 接受请求。它不表示 backend 已执行、停止已被 backend 确认，或车辆已处于物理安全状态。调用方必须依据 Server 返回的执行状态和 diagnostics 判断结果；SDK 不自动重连、重发 mutating request 或恢复运动。

当前接口尚未实现完整车辆级 `ModeRequest`、统一 resource claim、整车 stop confirmation 或 schema negotiation。这些能力不能由 SDK 文档或 schema 文件替代。

## 目录

```text
crates/contracts/  共享 DTO、RNG1 envelope 和 schema 身份类型
sdk/rust/          ranger-client Rust package
sdk/schema/        机器可读 schema 输入
sdk/docs/          接口边界与迁移说明
sdk/examples/      SDK 使用示例约束
```

## 本地验证

在具备 Rust Linux 构建环境中执行：

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release
```

`ranger-client` 依赖 Linux Unix domain `SOCK_SEQPACKET`；构建通过不证明目标 Server、profile 或硬件已经验证。

## 迁移边界

此仓添加到 `ranger` 集成仓只固定一次接口候选版本，不会自动让现有 path dependency 切换。下一步应先让 Ranger-Go 改为固定的 `ranger-sdk` 依赖并验证 wire 兼容，再让 `ranger-server` 消费同一版本，最后删除其重复来源。迁移完成前，Server 和 Go 的实际发布组合仍须以各自仓库的锁定 revision 为准。
