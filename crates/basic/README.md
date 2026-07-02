# aipmc-basic

AIP 智能体互联协议核心库 — 基于 GB/Z 185 标准的 ACPs 协议实现。

## 概述

`aipmc-basic` 是 AIPMC 项目的核心协议库，实现了面向 MCU 场景的 ACPs（Agent Collaboration Protocol Suite）二进制协议。采用 `no_std` 设计，无平台依赖，可独立测试。

## 核心模块

| 模块 | 说明 |
|------|------|
| [`protocol`] | ACPs-MCU 协议定义：方法码、状态码、帧头格式 |
| [`message`] | 协议消息 `AcpMessage`：帧头 + Session-ID + Payload |
| [`codec`] | 编解码器 `AcpCodec`：encode/decode 便捷接口 |
| [`agent`] | 智能体卡片 `AgentCard`：身份、能力、技能描述 |
| [`skill`] | 智能体技能 `AgentSkill`：输入输出模式定义 |
| [`error`] | 统一错误类型 `AipError` |

## 协议帧格式

```
+--------+--------+--------+--------+--------+--------+--------+--------+
| Magic0 | Magic1 | Ver    | Method | Flags  | LenHi  | LenLo  | Chksum |
+--------+--------+--------+--------+--------+--------+--------+--------+
| SID0   | SID1   | SID2   | SID3   | Payload ...                          |
+--------+--------+--------+--------+-------------------------------------+
```

- Magic: `0xA1A2` (2B)
- Version: `1` (1B)
- Method: 方法码 (1B)
- Flags: FIN/COMP/ENC (1B)
- Payload Length: 含 Session-ID (2B, 大端序)
- Checksum: 算术校验和 (1B)
- Session-ID: 会话标识 (4B, 大端序)

## 使用示例

```rust
use aipmc::basic::{AgentCard, AgentSkill, AcpMessage, AcpMethod, AgentCapability};

// 构建智能体卡片
let agent = AgentCard::new("esp32-01", "TempSensor", "Temperature sensor")
    .unwrap()
    .with_capability(AgentCapability::SensorRead)
    .unwrap()
    .with_skill(
        AgentSkill::new("temp-read", "ReadTemp", "Read temperature")
            .unwrap()
            .with_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
            .unwrap()
    )
    .unwrap()
    .with_endpoint("aip://esp32-01.local")
    .unwrap();

// 编解码消息
let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
let encoded = msg.encode().unwrap();
let decoded = AcpMessage::decode(&encoded).unwrap();
assert_eq!(decoded.session_id, 0);
```

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `alloc` | 是 | 启用动态内存分配支持（JSON 序列化等） |
| `std` | 否 | 启用标准库支持（含 alloc） |
| `defmt` | 否 | 启用 `defmt` 格式化输出 |
