# AIPMC - AIP智能体互联协议 MCU 实现

**中文** | [English](README.en.md)

基于 GB/Z 185 标准的智能体互联协作协议（ACPs）在微控制器（MCU）上的 Rust 实现，面向 ESP-32 / STM-32 等嵌入式平台。

## 项目简介

AIPMC（Agent Interconnection Protocol for Micro-Controllers）是将 [ACPs-community](https://atomgit.com/AIP-PUB/ACPs-community) 智能体互联协作协议适配到资源受限的微控制器场景的开源项目。它使 ESP-32、STM-32 等 MCU 设备能够作为轻量级智能体节点，参与基于 ACPs 协议的智能体网络，实现：

- **智能体身份与描述对齐规范**：身份使用 AIC（10 级点分 OID，带 CRC 校验），能力描述使用 ACS 的 `AgentCapabilitySpec`（camelCase JSON）
- **标准协议通信**：基于自有二进制帧原语（Initialize / Stream / TaskCancel / Authorize 等）进行智能体间通信
- **多传输通道**：UART、SPI 已实现（任何 `embedded_io::Read + Write` 字节流都可复用同一套分帧逻辑）；WiFi / BLE 为占位实现；I2C 待实现

> ⚠️ **协议一致性提示**
>
> 数据模型（身份 AIC / 能力描述 ACS / camelCase 字段）已对齐 ACPs 规范；
> 但**传输与帧格式仍是自有的**（Magic `0xA1A2` + 单字节方法码），规范使用
> JSON-RPC 2.0 over HTTP/SSE（方法为 `rpc` / `stream` / `notification/*` / `group`）。
> 跨网关进入 ACPs 网络需要一个适配层（尚未实现）。

---

## 设计目标与约束

AIPMC 的核心设计目标是将基于 GB/Z 185 标准的 ACPs 智能体互联协作协议适配到微控制器场景，使得资源受限的 MCU 设备能够作为智能体节点参与智能体网络。

### MCU 约束条件

| 约束 | 说明 |
|------|------|
| 内存 | MCU RAM 通常 32KB~512KB，协议栈需严格控制内存占用 |
| 算力 | MCU 主频 48MHz~240MHz，避免复杂计算 |
| 网络 | 嵌入式网络受限（UART/SPI/BLE/WiFi），非 TCP/IP 标准环境 |
| 功耗 | 电池供电场景需低功耗设计 |
| 无 OS | 多数场景无操作系统，需 `no_std` 支持 |

### 设计原则

1. **零拷贝优先**：协议编解码尽量原地操作，减少内存分配
2. **静态分配**：使用 `heapless` 固定大小容器，避免动态内存
3. **分层解耦**：核心协议 / 传输层 / HAL 严格分离
4. **渐进适配**：核心库可独立测试，平台层逐步实现

---

## 分层架构

```
┌─────────────────────────────────────────┐
│              Application Layer               │  用户智能体逻辑
├─────────────────────────────────────────┤
│                 aipmc (统一导出)              │
├──────────────────────────────────────────┤
│            basic (协议核心)                    │
│  ┌─────────┬──────────┬───────────┐        │
│  │ Agent   │ Skill    │ Protocol  │        │
│  │ Card    │ Registry │ Engine    │        │
│  └─────────┴──────────┴───────────┘        │
│  ┌─────────┬──────────┬───────────┐        │
│  │ Message │ Codec    │ Error     │        │
│  │ Builder │ Stream   │ Handler   │        │
│  └─────────┴──────────┴───────────┘        │
├──────────────────────────────────────────┤
│           transport (传输层)                  │
│  ┌────────┬────────┬────────┬────────┐      │
│  │ UART   │  SPI   │ WiFi   │  BLE   │      │
│  │ 已实现 │ 已实现 │  占位  │  占位  │      │
│  └────────┴────────┴────────┴────────┘      │
│         FrameReader（read_exact 分帧）        │
├───────────┬───────────────────────────────┤
│  esp32    │          stm32                   │  平台适配
│ (ESP32    │        (STM32 HAL                │
│  HAL)     │         Adapter)                 │
├───────────┴───────────────────────────────┤
│              Hardware (MCU)                  │
│    ESP-32 / ESP32-S3 / STM32F4 / STM32H7    │
└─────────────────────────────────────────┘
```

### 核心模块说明

#### basic（aipmc-basic）

协议核心库，`no_std` 兼容，无平台依赖，**不依赖任何 IO trait**。

- **Aic**：智能体身份码，10 级点分 OID，构造即校验（格式 + CRC-16/CCITT-FALSE）
- **AgentCapabilitySpec**：ACS 能力描述（身份、协议能力、端点、默认 MIME、技能），camelCase JSON
- **AgentSkill / MimeMode**：技能描述（`id`/`name`/`description`/`version`/`tags`/`examples`/`inputModes`/`outputModes`）
- **AgentCapability**：MCU 硬件能力（传感器/执行器/…），序列化到 `x-mcu.capabilities`
- **AcpProtocol / AcpMethod / AcpStatusCode / AcpsBinding**：自有二进制帧定义，含方法码 → 规范的绑定查询
- **AcpMessage**：协议消息，包含报头 + Session-ID + Payload（编解码契约：编码自洽、解码绝不 panic）
- **AcpHeader / flags**：帧头字段与 FIN/COMP/ENC 标志位访问器
- **AcpCodec**：无状态编解码器（`encode` / `encode_to` / `decode` / `decode_frame`）
- **AipError**：统一错误类型（实现 `core::error::Error`，可用于 `no_std`）

#### transport（aipmc-transport）

传输层抽象，基于 `embedded-io` trait，`no_std` 且不做动态分配。

- **FrameReader**：从任意 `embedded_io::Read` 用 `read_exact` 逐帧读取、校验
- **StreamTransport**：整帧发送 + 按帧接收；`UartTransport` / `SpiTransport` 是其场景别名
- **WifiTransport / BleTransport**：占位实现，返回 `AipError::NotImplemented`（**不会 panic**）

| 传输方式 | 适用场景 | 典型波特率/带宽 | 状态 |
|----------|----------|-----------------|------|
| UART | MCU 间有线通信 | 9600~921600 bps | 已实现 |
| SPI | 高速板级通信 | 1~40 Mbps | 已实现（需板级代码把事务封装为字节流） |
| WiFi | ESP-32 无线联网 | 1~54 Mbps | 占位，返回 `NotImplemented` |
| BLE | 低功耗无线 | 1~2 Mbps | 占位，返回 `NotImplemented` |
| I2C | 低速传感器总线 | 100~400 Kbps | 未实现（不是有序字节流，无法复用 `StreamTransport`） |

#### esp32（aipmc-esp32）

ESP-32 平台适配层，`no_std`。已实现芯片识别（`esp32` / `esp32s2` / `esp32s3` / `esp32c3` / `esp32c6`
feature，互斥，冲突时 `compile_error!`）与 ACS 能力描述生成 `capability_spec(aic)`；`init_wifi`（需 `wifi` feature）与
`init_ble`（需 `ble` feature）返回 `AipError::NotImplemented`。尚未对接 `esp-hal` / `esp-wifi`。

#### stm32（aipmc-stm32）

STM-32 平台适配层，`no_std`。已实现系列识别（`stm32f4` / `stm32h7` / `stm32l4` / `stm32g4`
feature，互斥）与 ACS 能力描述生成 `capability_spec(aic)`；`init_uart` 返回 `AipError::NotImplemented`。
尚未对接 `stm32-hal` / `embassy-stm32`。

---

## 协议适配策略

云端 ACPs 协议基于 JSON-RPC over HTTP/SSE，MCU 场景做以下适配：

| 维度 | 云端 ACPs | MCU ACPs (本项目) |
|------|-----------|-------------------|
| 编码格式 | JSON | 二进制帧（可选 JSON payload） |
| 传输协议 | HTTP/SSE | UART/SPI/BLE 原始帧 |
| 会话管理 | HTTP Header (session-id) | 帧内 Session-ID 字段 |
| 流式传输 | SSE (text/event-stream) | 分帧流式传输 |
| 认证 | OAuth / AK-SK | 共享密钥 / 轻量认证 |
| 发现 | HTTP URL | 组播/广播发现帧 |

> **注意**：上表描述的是**设计意图**，不是与 ACPs 规范的实际字段级对应关系。
> 就当前 ACPs 社区实现（`acps-specs` v02.01）而言：协议方法是 `rpc` / `stream` /
> `notification/*` / `group`（JSON-RPC 2.0），智能体描述对象是 `AgentCapabilitySpec`（ACS），
> 身份字段是 `aic`，线上字段为 camelCase；本项目并没有这些方法，也没有 `session` 之外的会话握手。

---

## ACPs-MCU 协议规范

> 基于 GB/Z 185 标准的智能体互联协作协议 MCU 适配版

### 帧格式

#### 帧结构

```
+--------+--------+--------+--------+--------+--------+--------+--------+
| Magic0 | Magic1 | Ver    | Method | Flags  | LenHi  | LenLo  | Chksum |
+--------+--------+--------+--------+--------+--------+--------+--------+
| SID0   | SID1   | SID2   | SID3   | Payload ...                          |
+--------+--------+--------+--------+-------------------------------------+
```

#### 字段定义

| 字段 | 偏移 | 长度 | 说明 |
|------|------|------|------|
| Magic | 0 | 2B | 协议标识 `0xA1A2` |
| Version | 2 | 1B | 协议版本，当前 `0x01` |
| Method | 3 | 1B | 方法码，见下方方法码定义 |
| Flags | 4 | 1B | 标志位，见下方标志位定义 |
| Payload Length | 5 | 2B | 负载长度（含 Session-ID 4B），大端序 |
| Checksum | 7 | 1B | 全帧算术校验和（Checksum 字段本身填 0 后计算） |
| Session-ID | 8 | 4B | 会话标识，大端序 |
| Payload | 12 | 变长 | 业务数据 |

#### 标志位定义

| Bit | 名称 | 说明 |
|-----|------|------|
| 0 | FIN | 1=最后一帧，0=后续还有分片 |
| 1 | COMP | 1=payload 已压缩（预留） |
| 2 | ENC | 1=payload 已加密（预留） |
| 3-7 | RSV | 保留 |

### 方法码定义

方法码是本协议自有的（规范用 JSON-RPC 方法名字符串）。下表给出**真实**的对应关系，
代码中可通过 `AcpMethod::acps_binding()` 查询，并有测试锁定：

| 方法码 | 名称 | 方向 | ACPs 对应 |
|--------|------|------|-----------|
| 0x01 | Initialize | C→S | 无（MCU 扩展；规范没有握手流程） |
| 0x02 | Initialized | S→C | 无（MCU 扩展），返回 Session-ID |
| 0x03 | Stream | 双向 | JSON-RPC 方法 `stream`（可多帧） |
| 0x04 | TaskCancel | C→S | TaskCommand `cancel` |
| 0x05 | ClearContext | C→S | 无（MCU 扩展；规范没有该概念） |
| 0x06 | Authorize | C→S | 无（规范用 `securitySchemes` + mTLS 表达） |
| 0x07 | Deauthorize | C→S | 无（MCU 扩展） |
| 0x08 | Push | S→C | `notification/*`（Partner→Leader HTTP POST） |
| 0x10 | Discover | 广播 | ADP 发现路径 `/discover` |
| 0x11 | CapabilitySpec | S→C | ACS 文档对象 `AgentCapabilitySpec`（Discover 的响应） |
| 0x20 | Heartbeat | 双向 | 无（MCU 扩展），心跳保活 |
| 0x21 | HeartbeatAck | 双向 | 无（MCU 扩展），心跳确认 |

> 响应方法码映射由 `AcpMethod::response_method()` 统一给出：
> `Initialize → Initialized`、`Discover → CapabilitySpec`、`Heartbeat → HeartbeatAck`，其余方法保持不变。

### Payload 格式

> 以下 Payload 结构为**约定示例**（当前实现只提供编解码与数据模型，尚未实现这些流程）。
> 身份字段统一使用 `aic`（AIC 字符串），能力描述使用 ACS 的 camelCase 字段。

#### Initialize (0x01)

**请求 Payload:**
```json
{
  "protocolVersion": "02.01",
  "aic": "1.2.156.3088.1.0001.00001.ABC123.000001.176F",
  "capabilities": ["streaming"]
}
```

**响应 Payload:**
```json
{
  "session_id": 12345,
  "status": 0,
  "aic": "1.2.156.3088.1.0001.00002.ABC124.000003.XXXX"
}
```

#### Stream (0x03)

**请求 Payload:**
```json
{
  "skill_id": "temp.read",
  "input": {
    "unit": "C"
  }
}
```

**响应 Payload (多帧):**
```json
{
  "status": 1,
  "progress": 50,
  "partial": null
}
```

最终帧 (`FIN=1`):
```json
{
  "status": 0,
  "progress": 100,
  "output": {
    "value": 25.3,
    "unit": "C"
  }
}
```

#### Discover (0x10)

**请求 Payload（广播）:**
```json
{
  "filter": {
    "tag": "sensor"
  }
}
```

**响应 Payload (CapabilitySpec 0x11，ACS 子集 + MCU 扩展):**
```json
{
  "aic": "1.2.156.3088.1.0001.00001.ABC123.000001.176F",
  "active": true,
  "lastModifiedTime": "2026-07-03T09:26:02+08:00",
  "protocolVersion": "02.01",
  "name": "ESP32 Temperature Sensor",
  "description": "DHT22 temperature and humidity sensor",
  "version": "0.1.0",
  "provider": { "countryCode": "CN", "organization": "ACME" },
  "endPoints": [
    { "url": "aip://gateway.local/agents/esp32-01", "transport": "HTTP_JSON" }
  ],
  "capabilities": { "streaming": true, "notification": false, "messageQueue": [] },
  "defaultInputModes": ["application/json"],
  "defaultOutputModes": ["application/json"],
  "skills": [
    {
      "id": "temp.read",
      "name": "ReadTemperature",
      "description": "Read temperature",
      "version": "0.1.0",
      "tags": ["sensor"],
      "x-mcu": { "inputSchema": { "...": "..." }, "outputSchema": { "...": "..." } }
    }
  ],
  "x-mcu": { "capabilities": ["sensorRead"] }
}
```

#### Heartbeat (0x20)

**Payload: 空 或 4 字节时间戳**

### 状态码

| Code | 名称 | 说明 |
|------|------|------|
| 0x00 | OK | 成功 |
| 0x01 | Accepted | 已接受，异步处理中 |
| 0x02 | PartialContent | 部分内容（流式传输中间帧） |
| 0x40 | BadRequest | 请求格式错误 |
| 0x41 | Unauthorized | 未授权 |
| 0x44 | NotFound | 智能体/技能未找到 |
| 0x45 | MethodNotSupported | 方法不支持 |
| 0x4E | SessionExpired | 会话过期 |
| 0x50 | InternalError | 内部错误 |
| 0x53 | ServiceUnavailable | 服务不可用 |

### 校验和计算

对帧所有字节（Checksum 字段填 0）做模 256 算术求和：

```
checksum = (sum of all bytes with checksum_field=0) mod 256
```

### 分片传输

当 payload 大于 MTU 时，使用分片传输：

1. 首帧 `FIN=0`，携带部分 payload
2. 中间帧 `FIN=0`，同一 Session-ID
3. 末帧 `FIN=1`，标识传输完成

所有分片的 Method、Session-ID 必须相同。

### 与云端 ACPs 的映射关系

| 云端 ACPs | MCU ACPs | 说明 |
|-----------|----------|------|
| HTTP POST + JSON-RPC | 二进制帧 | 编码格式适配 |
| SSE 流式响应 | 分帧流式（FIN 标志） | 流式传输适配 |
| HTTP Header session-id | 帧内 Session-ID 字段 | 会话管理适配 |
| HTTPS TLS | Flags.ENC + 共享密钥加密 | 安全传输适配 |
| DNS/URL 端点发现 | Discover 广播帧 | 发现机制适配 |

---

## 典型交互流程

### 智能体初始化

```
Client (Gateway)              Server (MCU Agent)
     │                              │
     │── Initialize ──────────────>│
     │   (method=0x01, session=0)  │
     │                              │
     │<── Initialized ─────────────│
     │   (method=0x02, session=N)  │
     │                              │
```

### 技能调用

```
Client                        Server
     │                          │
     │── Message/Stream ──────>│
     │   (skill_id, input)     │
     │                          │
     │<── Message/Stream ──────│
     │   (partial result)      │
     │                          │
     │<── Message/Stream ──────│
     │   (final result)        │
     │                          │
```

### 智能体发现

```
Client                        Server
     │                          │
     │── Discover ────────────>│
     │   (broadcast)           │
     │                          │
     │<── Discover Response ───│
     │   (AgentCapabilitySpec JSON)      │
     │                          │
```

---

## 项目结构

```
aipmc/
├── Cargo.toml                    # [package] aipmc + [workspace] 子 crates + [workspace.lints]
├── src/
│   └── lib.rs                    # aipmc 统一导出层（no_std）
├── crates/
│   ├── basic/                    # 核心协议库 (no_std, 无 IO 依赖)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── aic.rs            # Aic：身份码解析/校验（CRC-16 + Base36）
│   │       ├── acs.rs            # AgentCapabilitySpec / AgentProvider / AgentEndPoint / Transport
│   │       ├── skill.rs          # AgentSkill / MimeMode
│   │       ├── protocol.rs       # 自有帧定义（方法码/状态码/帧头/flags/AcpsBinding）
│   │       ├── message.rs        # AcpMessage 消息编解码
│   │       ├── codec.rs          # AcpCodec 无状态编解码器
│   │       ├── util.rs           # crate 内部工具
│   │       └── error.rs          # AipError 统一错误类型
│   ├── transport/                # 传输层抽象 (no_std)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── frame.rs          # FrameReader：read_exact 分帧 + 校验
│   │       ├── stream.rs         # StreamTransport：整帧收发
│   │       ├── uart.rs           # UART 别名
│   │       ├── spi.rs            # SPI 别名
│   │       ├── wifi.rs           # WiFi 占位（返回 NotImplemented）
│   │       └── ble.rs            # BLE 占位（返回 NotImplemented）
│   ├── esp32/                    # ESP-32 平台适配（芯片 feature + AgentCapabilitySpec）
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/{lib.rs, hal.rs}
│   └── stm32/                    # STM-32 平台适配（系列 feature + AgentCapabilitySpec）
│       ├── Cargo.toml
│       ├── README.md
│       └── src/{lib.rs, hal.rs}
├── examples/
│   ├── README.md                 # 示例构建说明与限制
│   ├── esp32/                    # ESP-32 智能体固件骨架（独立包）
│   └── stm32/                    # STM-32 智能体固件骨架（独立包）
├── tests/
│   ├── integration.rs            # 集成测试入口
│   ├── basic/                    # 协议核心测试（protocol / message / agent）
│   └── transport/                # 传输层测试（frame / stream）
└── README.md / README.en.md
```

---

## 快速开始

### 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ESP-32 开发环境
rustup target add xtensa-esp32-none-elf
cargo install espflash

# STM-32 开发环境
rustup target add thumbv7em-none-eabihf
cargo install probe-rs
```

### 使用示例

```rust
use aipmc::basic::acs::{AgentCapability, AgentCapabilitySpec, AgentEndPoint, Transport};
use aipmc::basic::aic::Aic;
use aipmc::basic::skill::AgentSkill;
use aipmc::basic::{AcpMessage, AcpMethod};

// 身份码由注册中心签发；设备侧解析即校验（格式 + 段数 + Base36 + 前缀）
let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
assert!(aic.is_entity());
// 有 SALT 时可进一步校验第 10 级校验码
assert!(aic.verify_checksum(aipmc::basic::aic::REGISTRY_DEFAULT_SALT));

// ACS 能力描述：字段名与 JSON 大小写遵循规范
// 注意：约 11 KB，固件里应放在 static/.bss；设置接口是 &mut self
let mut spec =
    AgentCapabilitySpec::new(aic, "ESP32-Sensor", "Temperature sensor agent").unwrap();
let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
skill.add_tag("sensor").unwrap();
spec.add_skill(skill).unwrap();
spec.add_hardware_capability(AgentCapability::SensorRead) // MCU 扩展：x-mcu
    .unwrap();
spec.add_end_point(
    AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
)
.unwrap();
assert!(spec.validate().is_ok());
assert!(spec.find_skill("temp.read").is_some());

// 自有二进制帧：编码自洽、解码不 panic
let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
let encoded = msg.encode().unwrap();
let decoded = AcpMessage::decode(&encoded).unwrap();
```

### 运行测试

```bash
# 全工作区（单位置测试 + 集成测试 + doctest）
cargo test --workspace
```

### MCU 场景（关闭 std 与分配器）

```toml
[dependencies]
aipmc = { version = "0.0.1", default-features = false }
```

```bash
# 交叉编译检查（以 STM-32 目标为例）
cargo build --no-default-features --target thumbv7em-none-eabihf
```

### 构建 ESP-32 示例

```bash
cd examples/esp32
cargo build --release --target xtensa-esp32-none-elf
espflash flash target/xtensa-esp32-none-elf/release/aipmc-esp32-example
```

### 构建 STM-32 示例

```bash
cd examples/stm32
cargo build --release --target thumbv7em-none-eabihf
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/aipmc-stm32-example
```

## 功能路线图

### 已完成

- [x] 与规范对齐的数据模型：AIC 身份码（10 级 OID + CRC-16/CCITT-FALSE + Base36 校验）
- [x] ACS 能力描述 `AgentCapabilitySpec`（camelCase、规范的 `endPoints` / `capabilities` / `defaultInputModes` 等字段）
- [x] 技能 `AgentSkill`（规范的 `id`/`name`/`description`/`version`/`tags`/`examples`/`inputModes`/`outputModes`）
- [x] MCU 扩展显式命名空间化（根级 / 技能级 `x-mcu`），为空时不出现
- [x] 自有二进制协议编解码（`AcpHeader` / `AcpMessage` / `AcpCodec`），编码自洽、解码绝不 panic
- [x] 帧头字段校验（Magic / 版本 / 方法码 / `payload_len` 区间）与校验和校验
- [x] FIN / COMP / ENC 标志位与分片语义辅助方法
- [x] 方法码 → 规范概念的绑定查询（`AcpMethod::acps_binding()`）与响应映射，有测试锁定
- [x] 字节流分帧（`FrameReader`：`read_exact` + 长度校验）与 `StreamTransport`
- [x] UART / SPI 传输通道
- [x] ESP-32 / STM-32 芯片 feature 与 ACS 能力描述生成（`capability_spec(aic)`）
- [x] `no_std` + 无分配器构建；关闭默认特性后不依赖 `alloc`
- [x] 集成测试（99 个用例）+ doctest（7 个），覆盖全部已知缺陷的回归与 AIC 上游黄金向量
- [x] CI：`fmt` / `clippy -D warnings` / `test` / `no_std` 交叉构建 / `rustdoc`

### 待完成

- [ ] **网关适配层**：帧 ↔ JSON-RPC、方法码 ↔ 规范方法、`AcpStatusCode` ↔ `TaskState`/错误码
- [ ] 智能体发现流程（Discover 广播 / 应答 / 结果聚合）
- [ ] 会话管理（Initialize / 会话表 / 过期清理 / ClearContext）
- [ ] 授权认证流程（Authorize / Deauthorize / `securitySchemes`）
- [ ] 流式消息分片与重组（FIN 语义已定义，未实现重组状态机）
- [ ] 任务状态机（accepted / working / awaiting-input / awaiting-completion / completed / canceled / failed / rejected）
- [ ] 接收侧重同步（Magic 扫描），提升 UART 噪声环境下的恢复能力
- [ ] **ACS 改为常驻 flash**：以 JSON 形式放在 `.rodata`、构建期校验，
      把运行期 RAM 从约 11 KB 降到约 1 KB（面向 32 KB RAM 级芯片）
- [ ] WiFi 传输（对接 `esp-wifi`）
- [ ] BLE 传输（GATT 特征值收发）
- [ ] I2C 传输（需基于 I2C HAL 单独实现）
- [ ] ESP-32 / STM-32 完整 HAL 适配与真实固件示例（启动代码、链接脚本）
- [ ] 嵌入式目标上的硬件在环（HIL）测试
- [ ] （可选，选项 C）在 MCU 上实现 ACPs 传输子集，使设备可直接加入 ACPs 网络

## 已知限制

| 限制 | 说明 |
|------|------|
| 传输层非规范实现 | 数据模型已对齐规范，但帧格式与传输仍是自有的，进入 ACPs 网络需要网关适配层 |
| `securitySchemes` 未输出 | ACS 的必填字段之一，MCU 无法独立承担 TLS/mTLS，注册时由网关补齐 |
| AIC 不能生成 | 生成属注册中心职责；设备侧只做校验，AIC 需预置 |
| WiFi / BLE 未实现 | 相关 API 存在但返回 `AipError::NotImplemented`（不再 panic） |
| 平台层未对接 HAL | 只提供芯片识别与 ACS 骨架，无真实外设驱动 |
| 无接收侧重同步 | 帧头被破坏后无法自动重新对齐，需上层复位链路 |
| 32 KB RAM 需严控技能数 | `AgentCapabilitySpec` 常驻 `.bss` 约 11 KB（1 技能）/ 18.5 KB（满载 8 技能） |
| 帧校验和强度有限 | 8 位算术校验和只能发现偶发错误，不抵抗恶意篡改（身份码的 CRC-16 是另一套机制） |
| 示例不可直接烧录 | 只做类型检查，缺平台启动代码 |
| `--all-features` 不可用 | 平台 crate 的芯片 feature 互斥，同时启用会 `compile_error!` |

## 参考资源

- [GB/Z 185《人工智能 智能体互联》国家标准](https://std.samr.gov.cn/gb/search/gbDetailed?id=52A05F8113453761E06397BE0A0AFEB5)
- [ACPs-community Python 实现](https://atomgit.com/AIP-PUB/ACPs-community)
- [ACPs 协议规范文档](https://atomgit.com/AIP-PUB/ACPs-community/tree/main/acps-specs)

## 许可证

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)