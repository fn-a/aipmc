# aipmc-basic

AIP 智能体互联协议核心库 — MCU 场景下的 ACPs 数据模型与自有二进制帧编解码。

> **对齐程度**：身份（AIC）与能力描述（ACS `AgentCapabilitySpec`）已按 ACPs 规范
> v02.01 对齐；传输与帧格式仍是本项目自有的。MCU 特有信息统一放在 `x-mcu` 命名空间。

## 概述

`aipmc-basic` 是 AIPMC 项目的核心协议库。采用 `no_std` 设计，无平台依赖，
**不依赖任何 IO trait**（分帧收发属于 `aipmc-transport`）。

## 核心模块

| 模块 | 说明 |
|------|------|
| `aic` | `Aic` 智能体身份码：10 级点分 OID 的解析与校验（CRC-16/CCITT-FALSE + Base36） |
| `acs` | `AgentCapabilitySpec` / `AgentProvider` / `AgentEndPoint` / `Transport` / `AgentCapabilities` / `MqProtocolVersion` |
| `skill` | `AgentSkill` 技能描述、`MimeMode` MIME 类型 |
| `protocol` | 自有帧定义：方法码、状态码、帧头、标志位、`AcpsBinding` |
| `message` | 协议消息 `AcpMessage`：帧头 + Session-ID + Payload |
| `codec` | 编解码器 `AcpCodec`：无状态 encode/decode 便捷接口 |
| `error` | 统一错误类型 `AipError`（实现 `core::error::Error`） |

## 身份码（AIC）

10 级点分 OID，构造即校验：

```
1-4:  OID 前缀 1.2.156.3088
5:    版本号          Base36，1 位
6:    ARSP 序号       Base36，1~6 位
7:    Vendor 序号     Base36，1~6 位
8:    本体序列号       Base36，1~9 位
9:    实例序列号       Base36，1~9 位（全 0 = 本体 AIC）
10:   CRC 校验码      Base36，固定 4 位
```

- 规范化：去空白 + 转大写，因此 `Aic::parse(" 1.2.156.3088...176f ")` 与规范形式相等
- 校验码：`CRC-16/CCITT-FALSE(body_1_9 ++ SALT)` 再 Base36 编码为 4 位
- **生成不在 MCU 上做**：注册中心签发，设备侧预置 + 校验

```rust
use basic::aic::{Aic, REGISTRY_DEFAULT_SALT};

let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
assert!(aic.is_entity());
assert_eq!(aic.ontology_prefix(), "1.2.156.3088.1.0001.00001.ABC123");
assert!(aic.verify_checksum(REGISTRY_DEFAULT_SALT));
```

## 能力描述（ACS）

字段名与 JSON 大小写与规范一致，MCU 扩展放在 `x-mcu`：

```json
{
  "aic": "1.2.156.3088.1.0001.00001.ABC123.000001.176F",
  "active": true,
  "lastModifiedTime": "2026-07-03T09:26:02+08:00",
  "protocolVersion": "02.01",
  "name": "ESP32-Sensor",
  "description": "Temperature sensor agent",
  "version": "0.1.0",
  "provider": { "countryCode": "CN" },
  "endPoints": [{ "url": "aip://gateway.local/agents/esp32-01", "transport": "HTTP_JSON" }],
  "capabilities": { "streaming": true, "notification": false, "messageQueue": ["amqp:4.2"] },
  "defaultInputModes": ["application/json"],
  "defaultOutputModes": ["application/json"],
  "skills": [
    {
      "id": "temp.read",
      "name": "ReadTemperature",
      "description": "Read temperature",
      "version": "0.1.0",
      "tags": ["sensor"],
      "x-mcu": { "inputSchema": "{...}", "outputSchema": "{...}" }
    }
  ],
  "x-mcu": { "capabilities": ["sensorRead"] }
}
```

| 规范字段 | 本实现 |
|----------|--------|
| `aic` / `active` / `lastModifiedTime` / `protocolVersion` / `name` / `description` / `version` / `provider` / `endPoints` / `capabilities` / `defaultInputModes` / `defaultOutputModes` / `skills` | 已实现，同名同大小写 |
| `securitySchemes` | 不输出（MCU 无法独立承担 TLS/mTLS，注册时由网关补齐） |
| `iconUrl` / `documentationUrl` / `webAppUrl` / `entityUserId` / `entityMeta` / `certificate` | 未实现（可选字段） |
| 硬件能力 / 技能输入输出模式 / `MCU_*` 传输取值 | MCU 扩展：`x-mcu` 或扩展枚举值 |

### 使用示例

```rust
use basic::acs::{AgentCapability, AgentCapabilitySpec, AgentEndPoint, Transport};
use basic::aic::Aic;
use basic::skill::AgentSkill;

let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
let mut spec =
    AgentCapabilitySpec::new(aic, "ESP32-Sensor", "Temperature sensor agent").unwrap();

// 设置接口是 &mut self：按值 builder 会为每一步复制整个结构体（见下表）
let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
skill.add_tag("sensor").unwrap();
spec.add_skill(skill).unwrap();
spec.add_hardware_capability(AgentCapability::SensorRead)
    .unwrap();
spec.add_end_point(
    AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
)
.unwrap();

assert!(spec.validate().is_ok());              // 必填字段 + ISO-8601 时间格式
assert!(spec.find_skill("temp.read").is_some());
let json = spec.to_json().unwrap();            // camelCase + x-mcu
```

## 自有二进制帧格式

```
+--------+--------+--------+--------+--------+--------+--------+--------+
| Magic0 | Magic1 | Ver    | Method | Flags  | LenHi  | LenLo  | Chksum |
+--------+--------+--------+--------+--------+--------+--------+--------+
| SID0   | SID1   | SID2   | SID3   | Payload ...                          |
+--------+--------+--------+--------+-------------------------------------+
```

- Magic: `0xA1A2` (2B)；Version: `1` (1B，只接受 `ACP_VERSION`)
- Flags: FIN/COMP/ENC/RSV (1B)
- Payload Length: 含 Session-ID (2B 大端)，合法区间 `[4, 516]`
- Checksum: 帧头前 7 字节 + 帧体的算术和 (1B)
- Session-ID: 4B 大端

方法码与规范的对应关系可在代码中查询（`AcpMethod::acps_binding()`）：
`Stream` ↔ 规范方法 `stream`、`TaskCancel` ↔ TaskCommand `cancel`、
`Discover` ↔ ADP `/discover`、`CapabilitySpec` ↔ ACS `AgentCapabilitySpec`；
其余（`Initialize`/`Initialized`/`ClearContext`/`Authorize`/`Deauthorize`/
`Heartbeat`/`HeartbeatAck`）为 MCU 扩展。

## 编解码契约

- **编码自洽**：`payload` 是长度的唯一事实来源；`encode` / `encode_to` 写出的
  `magic` / `version` / `payload_len` / 校验和都由实际负载推导，
  即使调用方直接改写过 `header` 字段，产出的帧也一定能被 `decode` 还原。
- **解码不 panic**：长度不足、Magic 错误、版本不支持、方法码未知、
  `payload_len` 越界、数据长度与声明不一致、校验和不匹配，全部返回 `AipError`。
- **零额外拷贝**：`encode_to` 直接写入调用方缓冲区；`validate_checksum` 不申请临时缓冲。
- **流式友好**：`decode_frame` 返回消耗的字节数，可在连续字节流上反复调用。

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `alloc` | 是 | 允许 AIC/ACS/技能 的 JSON 往返（`to_json` / `from_json`），依赖 `alloc` |
| `std` | 否 | 启用标准库（隐含 `alloc`） |
| `defmt` | 否 | 为核心类型派生 `defmt::Format`（同时打开 `heapless/defmt`） |

纯 MCU 场景（无全局分配器）：

```toml
[dependencies]
basic = { package = "aipmc-basic", version = "0.0.1", default-features = false }
```
