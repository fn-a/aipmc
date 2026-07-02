# AIPMC - AIP Agent Interconnection Protocol for MCU

[中文](README.md) | **English**

A Rust implementation of the Agent Collaboration Protocol Suite (ACPs) based on the GB/Z 185 standard, targeting microcontroller (MCU) platforms such as ESP-32 and STM-32.

## Overview

AIPMC (Agent Interconnection Protocol for Micro-Controllers) is an open-source project that adapts the [ACPs-community](https://atomgit.com/AIP-PUB/ACPs-community) agent interconnection protocol for resource-constrained microcontroller scenarios. It enables ESP-32, STM-32, and other MCU devices to act as lightweight agent nodes participating in ACPs-based agent networks, providing:

- **Agent Discovery & Registration**: MCU devices describe their capabilities and skills via AgentCard, supporting dynamic service discovery
- **Standard Protocol Communication**: Inter-agent communication based on ACPs protocol primitives (Initialize / Message / TaskCancel / Authorize, etc.)
- **Multiple Transport Channels**: Support for UART, SPI, I2C, WiFi, BLE, and other common embedded communication methods
- **Low Resource Footprint**: `no_std` design with controllable core protocol stack memory usage, adaptable to MCUs with < 512KB RAM

---

## Design Goals & Constraints

The core design goal of AIPMC is to adapt the ACPs agent interconnection protocol based on the GB/Z 185 standard to microcontroller scenarios, enabling resource-constrained MCU devices to participate in agent networks as agent nodes.

### MCU Constraints

| Constraint | Description |
|------------|-------------|
| Memory | MCU RAM typically 32KB~512KB; protocol stack must strictly control memory usage |
| Compute | MCU clock rates 48MHz~240MHz; avoid complex computations |
| Network | Embedded networks are constrained (UART/SPI/BLE/WiFi), not standard TCP/IP environments |
| Power | Battery-powered scenarios require low-power design |
| No OS | Most scenarios have no operating system; `no_std` support required |

### Design Principles

1. **Zero-copy Priority**: Protocol encoding/decoding operates in-place whenever possible, minimizing memory allocation
2. **Static Allocation**: Uses `heapless` fixed-size containers, avoiding dynamic memory
3. **Layered Decoupling**: Strict separation between core protocol / transport layer / HAL
4. **Incremental Adaptation**: Core library can be tested independently; platform layers implemented incrementally

---

## Layered Architecture

```
┌────────────────────────────────────────┐
│              Application Layer               │  User agent logic
├────────────────────────────────────────┤
│                 aipmc (unified export)       │
├─────────────────────────────────────────┤
│            basic (protocol core)             │
│  ┌─────────┬──────────┬───────────┐        │
│  │ Agent   │ Skill    │ Protocol  │        │
│  │ Card    │ Registry │ Engine    │        │
│  └─────────┴──────────┴───────────┘        │
│  ┌─────────┬──────────┬───────────┐        │
│  │ Message │ Codec    │ Error     │        │
│  │ Builder │ Stream   │ Handler   │        │
│  └─────────┴──────────┴───────────┘        │
├────────────────────────────────────────┤
│           transport (transport layer)        │
│  ┌────┬─────┬────┬────┬───────────┐        │
│  │UART│ SPI │I2C │WiFi│   BLE     │        │
│  └────┴─────┴────┴────┴───────────┘        │
├───────────┬──────────────────────────────┤
│  esp32    │          stm32                   │  Platform adaptation
│ (ESP32    │        (STM32 HAL                │
│  HAL)     │         Adapter)                 │
├───────────┴──────────────────────────────┤
│              Hardware (MCU)                  │
│    ESP-32 / ESP32-S3 / STM32F4 / STM32H7    │
└────────────────────────────────────────┘
```

### Core Module Descriptions

#### basic (aipmc-basic)

Protocol core library, `no_std` compatible, no platform dependencies.

- **AgentCard**: Agent card describing agent identity, capabilities, and skill list
- **AgentSkill**: Skill description including input/output schema definitions
- **AcpProtocol**: Protocol constants and version definitions
- **AcpMessage**: Protocol message containing header + Session-ID + Payload
- **AcpCodec**: Streaming codec supporting frame-by-frame decoding from `embedded_io::Read`
- **AipError**: Unified error type

#### transport (aipmc-transport)

Transport layer abstraction based on `embedded-io` traits.

| Transport | Use Case | Typical Bandwidth |
|-----------|----------|-------------------|
| UART | Wired inter-MCU communication | 9600~921600 bps |
| SPI | High-speed board-level communication | 1~40 Mbps |
| I2C | Low-speed sensor bus | 100~400 Kbps |
| WiFi | ESP-32 wireless networking | 1~54 Mbps |
| BLE | Low-power wireless | 1~2 Mbps |

#### esp32 (aipmc-esp32)

ESP-32 platform adaptation layer, depending on `esp-wifi`, `esp-hal`, and other crates.

#### stm32 (aipmc-stm32)

STM-32 platform adaptation layer, depending on `stm32-hal` and other crates.

---

## Protocol Adaptation Strategy

The cloud ACPs protocol is based on JSON-RPC over HTTP/SSE. The MCU scenario makes the following adaptations:

| Dimension | Cloud ACPs | MCU ACPs (this project) |
|-----------|------------|-------------------------|
| Encoding format | JSON | Binary frames (optional JSON payload) |
| Transport protocol | HTTP/SSE | UART/SPI/BLE raw frames |
| Session management | HTTP Header (session-id) | In-frame Session-ID field |
| Streaming | SSE (text/event-stream) | Frame-based streaming |
| Authentication | OAuth / AK-SK | Shared secret / lightweight auth |
| Discovery | HTTP URL | Multicast/broadcast discovery frames |

---

## ACPs-MCU Protocol Specification

> MCU-adapted version of the Agent Collaboration Protocol Suite based on the GB/Z 185 standard

### Frame Format

#### Frame Structure

```
+--------+--------+--------+--------+--------+--------+--------+--------+
| Magic0 | Magic1 | Ver    | Method | Flags  | LenHi  | LenLo  | Chksum |
+--------+--------+--------+--------+--------+--------+--------+--------+
| SID0   | SID1   | SID2   | SID3   | Payload ...                          |
+--------+--------+--------+--------+-------------------------------------+
```

#### Field Definitions

| Field | Offset | Size | Description |
|-------|--------|------|-------------|
| Magic | 0 | 2B | Protocol identifier `0xA1A2` |
| Version | 2 | 1B | Protocol version, currently `0x01` |
| Method | 3 | 1B | Method code, see Method Code Definitions below |
| Flags | 4 | 1B | Flag bits, see Flag Bit Definitions below |
| Payload Length | 5 | 2B | Payload length (including Session-ID 4B), big-endian |
| Checksum | 7 | 1B | Full-frame arithmetic checksum (computed with Checksum field set to 0) |
| Session-ID | 8 | 4B | Session identifier, big-endian |
| Payload | 12 | Variable | Business data |

#### Flag Bit Definitions

| Bit | Name | Description |
|-----|------|-------------|
| 0 | FIN | 1=last frame, 0=more fragments follow |
| 1 | COMP | 1=payload is compressed (reserved) |
| 2 | ENC | 1=payload is encrypted (reserved) |
| 3-7 | RSV | Reserved |

### Method Code Definitions

#### ACPs Standard Methods

| Code | Name | Direction | Description |
|------|------|-----------|-------------|
| 0x01 | Initialize | C→S | Client initiates session |
| 0x02 | Initialized | S→C | Server confirms initialization, returns Session-ID |
| 0x03 | Message/Stream | Bidirectional | Message streaming (may span multiple frames) |
| 0x04 | Task/Cancel | C→S | Cancel current task |
| 0x05 | ClearContext | C→S | Clear conversation context |
| 0x06 | Authorize | C→S | Initiate authorization |
| 0x07 | Deauthorize | C→S | Cancel authorization |
| 0x08 | Push | S→C | Server push notification |

#### MCU Extension Methods

| Code | Name | Direction | Description |
|------|------|-----------|-------------|
| 0x10 | Discover | Broadcast | Agent discovery request/response |
| 0x11 | AgentCard | S→C | AgentCard description response |
| 0x20 | Heartbeat | Bidirectional | Keep-alive heartbeat |
| 0x21 | HeartbeatAck | Bidirectional | Heartbeat acknowledgment |

### Payload Formats

#### Initialize (0x01)

**Request Payload:**
```json
{
  "version": "1.0",
  "agent_id": "client-agent-01",
  "capabilities": ["sensor-read"]
}
```

**Response Payload:**
```json
{
  "session_id": 12345,
  "status": 0,
  "server_agent_id": "esp32-sensor-01"
}
```

#### Message/Stream (0x03)

**Request Payload:**
```json
{
  "skill_id": "temp-read",
  "input": {
    "unit": "C"
  }
}
```

**Response Payload (multi-frame):**
```json
{
  "status": 1,
  "progress": 50,
  "partial": null
}
```

Final frame (`FIN=1`):
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

**Request Payload (broadcast):**
```json
{
  "filter": {
    "capability": "sensor-read",
    "tag": "temperature"
  }
}
```

**Response Payload (AgentCard 0x11):**
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

**Payload: Empty or 4-byte timestamp**

### Status Codes

| Code | Name | Description |
|------|------|-------------|
| 0x00 | OK | Success |
| 0x01 | Accepted | Accepted, processing asynchronously |
| 0x02 | PartialContent | Partial content (streaming intermediate frame) |
| 0x40 | BadRequest | Malformed request |
| 0x41 | Unauthorized | Not authorized |
| 0x44 | NotFound | Agent/skill not found |
| 0x45 | MethodNotSupported | Method not supported |
| 0x4E | SessionExpired | Session expired |
| 0x50 | InternalError | Internal error |
| 0x53 | ServiceUnavailable | Service unavailable |

### Checksum Calculation

Arithmetic sum of all frame bytes (with Checksum field set to 0) modulo 256:

```
checksum = (sum of all bytes with checksum_field=0) mod 256
```

### Fragmented Transmission

When payload exceeds MTU, fragmented transmission is used:

1. First frame: `FIN=0`, carries partial payload
2. Intermediate frames: `FIN=0`, same Session-ID
3. Final frame: `FIN=1`, marks transmission complete

All fragments must share the same Method and Session-ID.

### Cloud-to-MCU ACPs Mapping

| Cloud ACPs | MCU ACPs | Description |
|------------|----------|-------------|
| HTTP POST + JSON-RPC | Binary frames | Encoding format adaptation |
| SSE streaming response | Frame-based streaming (FIN flag) | Streaming adaptation |
| HTTP Header session-id | In-frame Session-ID field | Session management adaptation |
| HTTPS TLS | Flags.ENC + shared key encryption | Secure transport adaptation |
| DNS/URL endpoint discovery | Discover broadcast frames | Discovery mechanism adaptation |

---

## Typical Interaction Flows

### Agent Initialization

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

### Skill Invocation

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

### Agent Discovery

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

## Project Structure

```
aipmc/
├── Cargo.toml                    # [package] aipmc + [workspace] sub-crates
├── src/
│   └── lib.rs                    # aipmc unified export layer
├── crates/
│   ├── basic/                    # Core protocol library (no_std)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent.rs          # AgentCard / AgentCapability
│   │       ├── skill.rs          # AgentSkill
│   │       ├── protocol.rs       # ACPs protocol definitions
│   │       ├── message.rs        # AcpMessage
│   │       ├── codec.rs          # AcpCodec
│   │       └── error.rs          # AipError
│   ├── transport/                # Transport layer abstraction
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── uart.rs           # UART transport
│   │       ├── wifi.rs           # WiFi transport (ESP-32)
│   │       └── ble.rs            # BLE transport
│   ├── esp32/                    # ESP-32 platform adaptation
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       └── hal.rs
│   └── stm32/                    # STM-32 platform adaptation
│       ├── Cargo.toml
│       ├── README.md
│       └── src/
│           ├── lib.rs
│           └── hal.rs
├── examples/
│   ├── esp32/                    # ESP-32 agent example
│   └── stm32/                    # STM-32 agent example
├── tests/                        # Integration tests
└── README.md
```

---

## Quick Start

### Environment Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ESP-32 development environment
rustup target add xtensa-esp32-none-elf
cargo install espflash

# STM-32 development environment
rustup target add thumbv7em-none-eabihf
cargo install probe-rs
```

### Usage Example

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

### Running Tests

```bash
cargo test
```

### Building ESP-32 Example

```bash
cargo build --target xtensa-esp32-none-elf --bin aipmc-esp32-example
espflash flash target/xtensa-esp32-none-elf/debug/aipmc-esp32-example
```

### Building STM-32 Example

```bash
cargo build --target thumbv7em-none-eabihf --bin aipmc-stm32-example
probe-rs run target/thumbv7em-none-eabihf/debug/aipmc-stm32-example
```

---

## Roadmap

- [x] Core protocol data structures (AgentCard / AgentSkill / AcpMessage)
- [x] Binary protocol codec
- [x] UART transport implementation
- [x] Integration tests (17 test cases)
- [ ] WiFi transport (based on esp-wifi)
- [ ] BLE transport
- [ ] Agent discovery protocol implementation
- [ ] Session management (initialize / clearContext)
- [ ] Authorization and authentication flow
- [ ] Streaming message transport
- [ ] ESP-32 full HAL adaptation
- [ ] STM-32 full HAL adaptation
- [ ] CI/CD automated builds

## References

- [GB/Z 185 National Standard for Artificial Intelligence Agent Interconnection](https://std.samr.gov.cn/gb/search/gbDetailed?id=52A05F8113453761E06397BE0A0AFEB5)
- [ACPs-community Python Implementation](https://atomgit.com/AIP-PUB/ACPs-community)
- [ACPs Protocol Specification](https://atomgit.com/AIP-PUB/ACPs-community/tree/main/acps-specs)

## License

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
