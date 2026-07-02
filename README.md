# AIPMC - AIP智能体互联协议 MCU 实现

**中文** | [English](README.en.md)

基于 GB/Z 185 标准的智能体互联协作协议（ACPs）在微控制器（MCU）上的 Rust 实现，面向 ESP-32 / STM-32 等嵌入式平台。

## 项目简介

AIPMC（Agent Interconnection Protocol for Micro-Controllers）是将 [ACPs-community](https://atomgit.com/AIP-PUB/ACPs-community) 智能体互联协作协议适配到资源受限的微控制器场景的开源项目。它使 ESP-32、STM-32 等 MCU 设备能够作为轻量级智能体节点，参与基于 ACPs 协议的智能体网络，实现：

- **智能体发现与注册**：MCU 设备通过 AgentCard 描述自身能力与技能，支持动态服务发现
- **标准协议通信**：基于 ACPs 协议原语（Initialize / Message / TaskCancel / Authorize 等）进行智能体间通信
- **多传输通道**：支持 UART、SPI、I2C、WiFi、BLE 等嵌入式常用通信方式
- **低资源占用**：`no_std` 设计，核心协议栈内存占用可控，适配 RAM < 512KB 的 MCU

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
│  ┌────┬─────┬────┬────┬───────────┐        │
│  │UART│ SPI │I2C │WiFi│   BLE     │        │
│  └────┴─────┴────┴────┴───────────┘        │
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

协议核心库，`no_std` 兼容，无平台依赖。

- **AgentCard**：智能体卡片，描述智能体身份、能力、技能列表
- **AgentSkill**：技能描述，包含输入/输出模式定义
- **AcpProtocol**：协议常量与版本定义
- **AcpMessage**：协议消息，包含报头 + Session-ID + Payload
- **AcpCodec**：流式编解码器，支持从 `embedded_io::Read` 逐帧解码
- **AipError**：统一错误类型

#### transport（aipmc-transport）

传输层抽象，基于 `embedded-io` trait。

| 传输方式 | 适用场景 | 典型波特率/带宽 |
|----------|----------|-----------------|
| UART | MCU 间有线通信 | 9600~921600 bps |
| SPI | 高速板级通信 | 1~40 Mbps |
| I2C | 低速传感器总线 | 100~400 Kbps |
| WiFi | ESP-32 无线联网 | 1~54 Mbps |
| BLE | 低功耗无线 | 1~2 Mbps |

#### esp32（aipmc-esp32）

ESP-32 平台适配层，依赖 `esp-wifi`、`esp-hal` 等 crate。

#### stm32（aipmc-stm32）

STM-32 平台适配层，依赖 `stm32-hal` 等 crate。

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

#### ACPs 标准方法

| 方法码 | 名称 | 方向 | 说明 |
|--------|------|------|------|
| 0x01 | Initialize | C→S | 客户端发起会话初始化 |
| 0x02 | Initialized | S→C | 服务端确认初始化，返回 Session-ID |
| 0x03 | Message/Stream | 双向 | 消息流式传输（可多帧） |
| 0x04 | Task/Cancel | C→S | 取消当前任务 |
| 0x05 | ClearContext | C→S | 清除对话上下文 |
| 0x06 | Authorize | C→S | 发起授权 |
| 0x07 | Deauthorize | C→S | 取消授权 |
| 0x08 | Push | S→C | 服务端推送 |

#### MCU 扩展方法

| 方法码 | 名称 | 方向 | 说明 |
|--------|------|------|------|
| 0x10 | Discover | 广播 | 智能体发现请求/响应 |
| 0x11 | AgentCard | S→C | AgentCard 描述返回 |
| 0x20 | Heartbeat | 双向 | 心跳保活 |
| 0x21 | HeartbeatAck | 双向 | 心跳确认 |

### Payload 格式

#### Initialize (0x01)

**请求 Payload:**
```json
{
  "version": "1.0",
  "agent_id": "client-agent-01",
  "capabilities": ["sensor-read"]
}
```

**响应 Payload:**
```json
{
  "session_id": 12345,
  "status": 0,
  "server_agent_id": "esp32-sensor-01"
}
```

#### Message/Stream (0x03)

**请求 Payload:**
```json
{
  "skill_id": "temp-read",
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
    "capability": "sensor-read",
    "tag": "temperature"
  }
}
```

**响应 Payload (AgentCard 0x11):**
```json
{
  "agent_id": "esp32-sensor-01",
  "name": "ESP32 Temperature Sensor",
  "description": "DHT22 temperature and humidity sensor",
  "version": "0.1.0",
  "capabilities": ["sensor-read"],
  "skills": [
    {
      "id": "temp-read",
      "name": "ReadTemperature",
      "input_schema": {...},
      "output_schema": {...}
    }
  ],
  "endpoint": "aip://esp32-sensor-01.local"
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
     │   (AgentCard JSON)      │
     │                          │
```

---

## 项目结构

```
aipmc/
├── Cargo.toml                    # [package] aipmc + [workspace] 子 crates
├── src/
│   └── lib.rs                    # aipmc 统一导出层
├── crates/
│   ├── basic/                    # 核心协议库 (no_std)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs            
│   │       ├── agent.rs          # AgentCard / AgentCapability
│   │       ├── skill.rs          # AgentSkill 技能描述
│   │       ├── protocol.rs       # ACPs 协议定义 (方法码/状态码/帧头)
│   │       ├── message.rs        # AcpMessage 消息编解码
│   │       ├── codec.rs          # AcpCodec 编解码器
│   │       └── error.rs          # AipError 统一错误类型
│   ├── transport/                # 传输层抽象
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── uart.rs           # UART 传输
│   │       ├── wifi.rs           # WiFi 传输 (ESP-32)
│   │       └── ble.rs            # BLE 传输
│   ├── esp32/                    # ESP-32 平台适配
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       └── hal.rs
│   └── stm32/                    # STM-32 平台适配
│       ├── Cargo.toml
│       ├── README.md
│       └── src/
│           ├── lib.rs
│           └── hal.rs
├── examples/
│   ├── esp32/                    # ESP-32 智能体示例
│   └── stm32/                    # STM-32 智能体示例
├── tests/                        # 集成测试
└── README.md
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
use aipmc::basic::{AgentCard, AgentSkill, AcpMessage, AcpMethod, AgentCapability};

let agent = AgentCard::new("dev-01", "MyAgent", "A test agent")
    .unwrap()
    .with_capability(AgentCapability::SensorRead)
    .unwrap()
    .with_skill(AgentSkill::new("temp", "ReadTemp", "Read temperature").unwrap())
    .unwrap();

let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
let encoded = msg.encode().unwrap();
let decoded = AcpMessage::decode(&encoded).unwrap();
```

### 运行测试

```bash
cargo test
```

### 构建 ESP-32 示例

```bash
cargo build --target xtensa-esp32-none-elf --bin aipmc-esp32-example
espflash flash target/xtensa-esp32-none-elf/debug/aipmc-esp32-example
```

### 构建 STM-32 示例

```bash
cargo build --target thumbv7em-none-eabihf --bin aipmc-stm32-example
probe-rs run target/thumbv7em-none-eabihf/debug/aipmc-stm32-example
```

---

## 功能路线图

- [x] 核心协议数据结构（AgentCard / AgentSkill / AcpMessage）
- [x] 二进制协议编解码器
- [x] UART 传输层实现
- [x] 集成测试（17 个测试用例）
- [ ] WiFi 传输层（基于 esp-wifi）
- [ ] BLE 传输层
- [ ] 智能体发现协议实现
- [ ] 会话管理（initialize / clearContext）
- [ ] 授权认证流程
- [ ] 流式消息传输
- [ ] ESP-32 完整 HAL 适配
- [ ] STM-32 完整 HAL 适配
- [ ] CI/CD 自动化构建

## 参考资源

- [GB/Z 185《人工智能 智能体互联》国家标准](https://std.samr.gov.cn/gb/search/gbDetailed?id=52A05F8113453761E06397BE0A0AFEB5)
- [ACPs-community Python 实现](https://atomgit.com/AIP-PUB/ACPs-community)
- [ACPs 协议规范文档](https://atomgit.com/AIP-PUB/ACPs-community/tree/main/acps-specs)

## 许可证

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)