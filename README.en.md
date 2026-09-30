# AIPMC - AIP Agent Interconnection Protocol for MCU

[中文](README.md) | **English**

A Rust implementation of the Agent Collaboration Protocol Suite (ACPs) based on the GB/Z 185 standard, targeting microcontroller (MCU) platforms such as ESP-32 and STM-32.

## Overview

AIPMC (Agent Interconnection Protocol for Micro-Controllers) is an open-source project that adapts the [ACPs-community](https://atomgit.com/AIP-PUB/ACPs-community) agent interconnection protocol for resource-constrained microcontroller scenarios. It enables ESP-32, STM-32, and other MCU devices to act as lightweight agent nodes participating in ACPs-based agent networks, providing:

- **Identity & description aligned with the spec**: identity uses an AIC (10-segment dotted OID with a CRC checksum); the capability document is the ACS `AgentCapabilitySpec` (camelCase JSON)
- **Standard Protocol Communication**: inter-agent communication based on bespoke binary frame primitives (Initialize / Stream / TaskCancel / Authorize, etc.)
- **Multiple Transport Channels**: UART and SPI are implemented (any `embedded_io::Read + Write` byte stream reuses the same framing logic); WiFi / BLE are placeholders; I2C is not implemented yet

> ⚠️ **Protocol conformance note**
>
> The data model (AIC identity / ACS description / camelCase fields) has been aligned with the
> ACPs specification. However the **transport and frame format remain
> bespoke** (magic `0xA1A2` + single-byte method codes); the specification uses JSON-RPC 2.0
> over HTTP/SSE (methods `rpc` / `stream` / `notification/*` / `group`).
> Reaching an ACPs network through a gateway still needs an adaptation layer (not implemented).

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
│  ┌────────┬────────┬────────┬────────┐      │
│  │ UART   │  SPI   │ WiFi   │  BLE   │      │
│  │  done  │  done  │ stub   │ stub   │      │
│  └────────┴────────┴────────┴────────┘      │
│         FrameReader (read_exact framing)     │
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

Protocol core library, `no_std` compatible, no platform dependencies and **no IO traits**.

- **Aic**: Agent identity code — a 10-segment dotted OID, validated on construction (format + CRC-16/CCITT-FALSE)
- **AgentCapabilitySpec**: ACS capability document (identity, protocol capabilities, endpoints, default MIME modes, skills) with camelCase JSON
- **AgentSkill / MimeMode**: Skill description (`id`/`name`/`description`/`version`/`tags`/`examples`/`inputModes`/`outputModes`)
- **AgentCapability**: MCU hardware capability (sensor/actuator/…), serialized into `x-mcu.capabilities`
- **AcpProtocol / AcpMethod / AcpStatusCode / AcpsBinding**: the bespoke frame definitions, including a queryable method-to-spec binding
- **AcpMessage**: Protocol message containing header + Session-ID + Payload (encoding is self-consistent; decoding never panics)
- **AcpHeader / flags**: Header fields plus FIN/COMP/ENC flag accessors
- **AcpCodec**: Stateless codec (`encode` / `encode_to` / `decode` / `decode_frame`)
- **AipError**: Unified error type implementing `core::error::Error` (usable in `no_std`)

#### transport (aipmc-transport)

Transport layer abstraction based on `embedded-io` traits; `no_std` and allocation-free.

- **FrameReader**: Reads and validates one frame at a time from any `embedded_io::Read` using `read_exact`
- **StreamTransport**: Whole-frame send plus frame-wise receive; `UartTransport` / `SpiTransport` are scenario aliases
- **WifiTransport / BleTransport**: Placeholders returning `AipError::NotImplemented` (**they never panic**)

| Transport | Use Case | Typical Bandwidth | Status |
|-----------|----------|-------------------|--------|
| UART | Wired inter-MCU communication | 9600~921600 bps | Implemented |
| SPI | High-speed board-level communication | 1~40 Mbps | Implemented (board code must expose transactions as a byte stream) |
| WiFi | ESP-32 wireless networking | 1~54 Mbps | Placeholder, returns `NotImplemented` |
| BLE | Low-power wireless | 1~2 Mbps | Placeholder, returns `NotImplemented` |
| I2C | Low-speed sensor bus | 100~400 Kbps | Not implemented (not an ordered byte stream, so `StreamTransport` does not apply) |

#### esp32 (aipmc-esp32)

ESP-32 platform adaptation layer, `no_std`. Chip identification (`esp32` / `esp32s2` / `esp32s3` /
`esp32c3` / `esp32c6` features, mutually exclusive, conflicting selection triggers `compile_error!`)
and ACS capability-spec generation `capability_spec(aic)` are implemented; `init_wifi` (needs `wifi`) and `init_ble` (needs `ble`)
return `AipError::NotImplemented`. `esp-hal` / `esp-wifi` are not integrated yet.

#### stm32 (aipmc-stm32)

STM-32 platform adaptation layer, `no_std`. Series identification (`stm32f4` / `stm32h7` /
`stm32l4` / `stm32g4` features, mutually exclusive) and ACS capability-spec generation are implemented;
`init_uart` returns `AipError::NotImplemented`. `stm32-hal` / `embassy-stm32` are not integrated yet.

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

> **Note**: the table above states *design intent*, not a field-level mapping onto the ACPs
> specification. In the current ACPs community implementation (`acps-specs` v02.01) the protocol
> methods are `rpc` / `stream` / `notification/*` / `group` (JSON-RPC 2.0), the agent description
> object is `AgentCapabilitySpec` (ACS), the identity field is `aic`, and wire fields are
> camelCase. This project implements none of those methods and has no session handshake beyond a Session-ID field.

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

Method codes are bespoke to this protocol (the spec uses JSON-RPC method name strings).
The table below gives the **actual** correspondence; it is queryable in code via
`AcpMethod::acps_binding()` and locked down by tests:

| Code | Name | Direction | ACPs correspondence |
|------|------|-----------|---------------------|
| 0x01 | Initialize | C→S | none (MCU extension; the spec has no handshake) |
| 0x02 | Initialized | S→C | none (MCU extension), returns Session-ID |
| 0x03 | Stream | Bidirectional | JSON-RPC method `stream` (may span multiple frames) |
| 0x04 | TaskCancel | C→S | TaskCommand `cancel` |
| 0x05 | ClearContext | C→S | none (MCU extension; no such concept in the spec) |
| 0x06 | Authorize | C→S | none (the spec expresses this via `securitySchemes` + mTLS) |
| 0x07 | Deauthorize | C→S | none (MCU extension) |
| 0x08 | Push | S→C | `notification/*` (Partner→Leader HTTP POST) |
| 0x10 | Discover | Broadcast | ADP discovery path `/discover` |
| 0x11 | CapabilitySpec | S→C | ACS document object `AgentCapabilitySpec` (response to Discover) |
| 0x20 | Heartbeat | Bidirectional | none (MCU extension), keep-alive |
| 0x21 | HeartbeatAck | Bidirectional | none (MCU extension), heartbeat acknowledgment |

> Response mapping is provided by `AcpMethod::response_method()`:
> `Initialize → Initialized`, `Discover → CapabilitySpec`, `Heartbeat → HeartbeatAck`,
> everything else maps to itself.

### Payload Formats

> The payload shapes below are **conventions** (the current implementation only provides the
> codec and the data model, not these flows). Identity always uses `aic` (an AIC string) and
> capability documents use ACS camelCase fields.

#### Initialize (0x01)

**Request Payload:**
```json
{
  "protocolVersion": "02.01",
  "aic": "1.2.156.3088.1.0001.00001.ABC123.000001.176F",
  "capabilities": ["streaming"]
}
```

**Response Payload:**
```json
{
  "session_id": 12345,
  "status": 0,
  "aic": "1.2.156.3088.1.0001.00002.ABC124.000003.XXXX"
}
```

#### Stream (0x03)

**Request Payload:**
```json
{
  "skill_id": "temp.read",
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
    "tag": "sensor"
  }
}
```

**Response Payload (CapabilitySpec 0x11 — ACS subset plus MCU extensions):**
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
  "endPoints": [{ "url": "aip://gateway.local/agents/esp32-01", "transport": "HTTP_JSON" }],
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
     │   (AgentCapabilitySpec JSON)      │
     │                          │
```

---

## Project Structure

```
aipmc/
├── Cargo.toml                    # [package] aipmc + [workspace] sub-crates + [workspace.lints]
├── src/
│   └── lib.rs                    # aipmc unified export layer (no_std)
├── crates/
│   ├── basic/                    # Core protocol library (no_std, no IO dependency)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── aic.rs            # Aic: identity-code parsing/validation (CRC-16 + Base36)
│   │       ├── acs.rs            # AgentCapabilitySpec / AgentProvider / AgentEndPoint / Transport
│   │       ├── skill.rs          # AgentSkill / MimeMode
│   │       ├── protocol.rs       # bespoke frame definitions (methods/status/header/flags/AcpsBinding)
│   │       ├── message.rs        # AcpMessage codec
│   │       ├── codec.rs          # AcpCodec stateless codec
│   │       ├── util.rs           # crate-internal helpers
│   │       └── error.rs          # AipError
│   ├── transport/                # Transport layer abstraction (no_std)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── frame.rs          # FrameReader: read_exact framing + validation
│   │       ├── stream.rs         # StreamTransport: whole-frame send/receive
│   │       ├── uart.rs           # UART alias
│   │       ├── spi.rs            # SPI alias
│   │       ├── wifi.rs           # WiFi placeholder (returns NotImplemented)
│   │       └── ble.rs            # BLE placeholder (returns NotImplemented)
│   ├── esp32/                    # ESP-32 platform adaptation (chip features + AgentCapabilitySpec)
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/{lib.rs, hal.rs}
│   └── stm32/                    # STM-32 platform adaptation (series features + AgentCapabilitySpec)
│       ├── Cargo.toml
│       ├── README.md
│       └── src/{lib.rs, hal.rs}
├── examples/
│   ├── README.md                 # how to build the examples, and their limits
│   ├── esp32/                    # ESP-32 agent firmware skeleton (standalone package)
│   └── stm32/                    # STM-32 agent firmware skeleton (standalone package)
├── tests/
│   ├── integration.rs            # integration test entry point
│   ├── basic/                    # protocol core tests (protocol / message / agent)
│   └── transport/                # transport tests (frame / stream)
└── README.md / README.en.md
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
use aipmc::basic::acs::{AgentCapability, AgentCapabilitySpec, AgentEndPoint, Transport};
use aipmc::basic::aic::Aic;
use aipmc::basic::skill::AgentSkill;
use aipmc::basic::{AcpMessage, AcpMethod};

// The identity code is issued by the registry; parsing on-device validates it
// (format + segment count + Base36 + prefix)
let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
assert!(aic.is_entity());
// With a SALT you can also verify the level-10 CRC
assert!(aic.verify_checksum(aipmc::basic::aic::REGISTRY_DEFAULT_SALT));

// ACS capability document: field names and JSON casing follow the spec.
// Note: ~11 KB, so keep it in static/.bss on firmware; setters take &mut self.
let mut spec =
    AgentCapabilitySpec::new(aic, "ESP32-Sensor", "Temperature sensor agent").unwrap();
let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
skill.add_tag("sensor").unwrap();
spec.add_skill(skill).unwrap();
spec.add_hardware_capability(AgentCapability::SensorRead) // MCU extension: x-mcu
    .unwrap();
spec.add_end_point(
    AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
)
.unwrap();
assert!(spec.validate().is_ok());
assert!(spec.find_skill("temp.read").is_some());

// Bespoke binary frame: self-consistent encoding, panic-free decoding
let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
let encoded = msg.encode().unwrap();
let decoded = AcpMessage::decode(&encoded).unwrap();
```

### Running Tests

```bash
# Whole workspace (unit + integration + doctests)
cargo test --workspace
```

### MCU profile (no std, no allocator)

```toml
[dependencies]
aipmc = { version = "0.0.1", default-features = false }
```

```bash
# Cross-compile check (STM-32 target as an example)
cargo build --no-default-features --target thumbv7em-none-eabihf
```

### Building the ESP-32 Example

```bash
cd examples/esp32
cargo build --release --target xtensa-esp32-none-elf
espflash flash target/xtensa-esp32-none-elf/release/aipmc-esp32-example
```

### Building the STM-32 Example

```bash
cd examples/stm32
cargo build --release --target thumbv7em-none-eabihf
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/aipmc-stm32-example
```

## Roadmap

### Done

- [x] Spec-aligned data model: AIC identity code (10-segment OID + CRC-16/CCITT-FALSE + Base36 checksum)
- [x] ACS capability document `AgentCapabilitySpec` (camelCase; spec `endPoints` / `capabilities` / `defaultInputModes` fields)
- [x] Skills `AgentSkill` (spec `id`/`name`/`description`/`version`/`tags`/`examples`/`inputModes`/`outputModes`)
- [x] MCU extensions explicitly namespaced (root-level / skill-level `x-mcu`), omitted when empty
- [x] Bespoke binary codec (`AcpHeader` / `AcpMessage` / `AcpCodec`): self-consistent encoding, panic-free decoding
- [x] Header validation (magic / version / method / `payload_len` range) and checksum verification
- [x] FIN / COMP / ENC flags and fragmentation helpers
- [x] Queryable method-to-spec binding (`AcpMethod::acps_binding()`) with response mapping, locked by tests
- [x] Byte-stream framing (`FrameReader` with `read_exact` + length validation) and `StreamTransport`
- [x] UART / SPI transports
- [x] ESP-32 / STM-32 chip features and ACS capability-spec generation (`capability_spec(aic)`)
- [x] `no_std` and allocator-free builds; `alloc` can be switched off entirely
- [x] Integration tests (99 cases) + doctests (7), covering every known defect and the upstream AIC golden vectors
- [x] CI: `fmt` / `clippy -D warnings` / `test` / `no_std` cross build / `rustdoc`

### To do

- [ ] **Gateway adaptation layer**: frame ↔ JSON-RPC, method code ↔ spec method, `AcpStatusCode` ↔ `TaskState`/error codes
- [ ] Agent discovery flow (Discover broadcast / response / result aggregation)
- [ ] Session management (Initialize / session table / expiry / ClearContext)
- [ ] Authorization flow (Authorize / Deauthorize / `securitySchemes`)
- [ ] Streaming fragmentation and reassembly (FIN semantics defined, reassembly state machine missing)
- [ ] Task state machine (accepted / working / awaiting-input / awaiting-completion / completed / canceled / failed / rejected)
- [ ] Receive-side resynchronisation (magic scanning) for noisy UART links
- [ ] **Move the ACS to flash**: keep it as JSON in `.rodata`, validated at build time,
      dropping runtime RAM from ~11 KB to ~1 KB (for 32 KB-RAM class MCUs)
- [ ] WiFi transport (`esp-wifi` integration)
- [ ] BLE transport (GATT characteristic send/receive)
- [ ] I2C transport (needs a dedicated implementation on top of the I2C HAL)
- [ ] Full ESP-32 / STM-32 HAL adaptation and real firmware examples (startup code, linker scripts)
- [ ] Hardware-in-the-loop (HIL) tests on embedded targets
- [ ] (Optional, option C) implement an ACPs transport subset on the MCU so devices can join ACPs networks directly

## Known Limitations

| Limitation | Description |
|------------|-------------|
| Transport layer is not spec-conformant | The data model is aligned, but framing/transport remain bespoke; entering an ACPs network needs a gateway adapter |
| `securitySchemes` not emitted | One of the ACS required fields; an MCU cannot run TLS/mTLS on its own, so the gateway fills it in at registration |
| AIC cannot be generated | Generation is a registry responsibility; the device only validates, so the AIC must be provisioned |
| WiFi / BLE not implemented | The APIs exist but return `AipError::NotImplemented` (no longer panic) |
| Platform layer not wired to a HAL | Only chip identification and an ACS skeleton exist; no real peripheral drivers |
| No receive-side resync | A corrupted header cannot be re-aligned automatically; the upper layer must reset the link |
| 32 KB RAM needs a low skill count | `AgentCapabilitySpec` occupies ~11 KB in `.bss` (1 skill) / ~18.5 KB (8 skills) |
| Weak frame checksum | An 8-bit arithmetic sum only catches accidental corruption, not tampering (the AIC CRC-16 is a separate mechanism) |
| Examples are not flashable as-is | They only type-check; platform startup code is missing |
| `--all-features` does not work | Platform crates have mutually exclusive chip features; enabling several triggers `compile_error!` |

## References

- [GB/Z 185 National Standard for Artificial Intelligence Agent Interconnection](https://std.samr.gov.cn/gb/search/gbDetailed?id=52A05F8113453761E06397BE0A0AFEB5)
- [ACPs-community Python Implementation](https://atomgit.com/AIP-PUB/ACPs-community)
- [ACPs Protocol Specification](https://atomgit.com/AIP-PUB/ACPs-community/tree/main/acps-specs)

## License

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
