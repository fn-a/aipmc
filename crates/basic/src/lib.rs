//! # aipmc-basic: AIP 智能体互联协议核心库
//!
//! 面向资源受限微控制器（MCU）的 ACPs 协议核心实现，`no_std` 设计，
//! **不依赖任何 IO trait**（分帧收发属于 `aipmc-transport`）。
//!
//! ## 定位与对齐程度
//!
//! 本 crate 同时提供两类东西：
//!
//! 1. **与 ACPs 规范对齐的数据模型**：身份码 [`aic::Aic`]（10 级点分 OID）、
//!    能力描述 [`acs::AgentCapabilitySpec`]（camelCase、ACS 字段名）、
//!    技能 [`skill::AgentSkill`]。
//! 2. **自有的 MCU 二进制帧协议**：[`protocol`] / [`message`] 定义的帧格式与
//!    单字节方法码，规范中没有对应物，跨网关时需要转换。
//!
//! ## 核心模块
//!
//! - [`aic`] - 智能体身份码 AIC 的解析与校验（CRC-16/CCITT-FALSE + Base36）
//! - [`acs`] - 智能体能力描述 `AgentCapabilitySpec`（ACS 的 MCU 子集）
//! - [`skill`] - 技能 `AgentSkill` 与 MIME 类型 `MimeMode`
//! - [`protocol`] - 自有二进制帧定义：方法码、状态码、帧头、标志位
//! - [`message`] - 协议消息 `AcpMessage`（帧头 + Session-ID + 负载）
//! - [`codec`] - 无状态编解码器 `AcpCodec`
//! - [`error`] - 统一错误类型 `AipError`
//!
//! ## Feature Flags
//!
//! | Feature | 默认 | 说明 |
//! |---------|------|------|
//! | `alloc` | 是 | 允许 AIC/ACS/技能 的 JSON 序列化（依赖 `alloc`） |
//! | `std`   | 否 | 启用标准库（隐含 `alloc`） |
//! | `defmt` | 否 | 为核心类型派生 `defmt::Format` |
//!
//! 纯 MCU 场景建议关闭默认特性：
//! `aipmc-basic = { version = "0.0.1", default-features = false }`

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod acs;
pub mod aic;
pub mod codec;
pub mod error;
pub mod message;
pub mod protocol;
pub mod skill;
mod util;

pub use acs::{
    ACPS_PROTOCOL_VERSION, AgentCapabilities, AgentCapability, AgentCapabilitySpec, AgentEndPoint,
    AgentProvider, McuSpecExtensions, MqProtocolVersion, Transport,
};
pub use aic::{
    AIC_MAX_LEN, AIC_OID_PREFIX, AIC_SEGMENTS, Aic, Crc16CcittFalse, REGISTRY_DEFAULT_SALT,
};
pub use codec::AcpCodec;
pub use error::{AipError, AipResult};
pub use message::AcpMessage;
pub use protocol::{
    ACP_HEADER_LEN, ACP_MAGIC, ACP_MAX_FRAME_LEN, ACP_MAX_PAYLOAD_LEN, ACP_MAX_PAYLOAD_LEN_WIRE,
    ACP_MIN_PAYLOAD_LEN, ACP_SESSION_ID_LEN, ACP_VERSION, AcpHeader, AcpMethod, AcpProtocol,
    AcpStatusCode, AcpsBinding, flags,
};
pub use skill::{AgentSkill, McuSkillExtensions, MimeMode};

/// 体积预算的**编译期**约束
///
/// MCU 上这些类型通常是静态分配的，体积回归会直接吃掉 RAM 预算。
/// 放在 `const` 上下文里的好处是：交叉编译（`cargo build --target <bare-metal>`）
/// 就会检查，不需要跑测试。
const _: () = {
    use core::mem::size_of;

    // 8B 帧头 + 4B Session-ID + 512B 负载 + heapless 长度字段
    assert!(size_of::<AcpMessage>() <= 600);
    // 规范化后的 AIC 最长 53 字节
    assert!(size_of::<Aic>() <= 128);
    // 单个技能
    assert!(size_of::<AgentSkill>() <= 2048);
    // 一个完整的能力描述（8 个技能 + 4 个端点 + 扩展）
    assert!(size_of::<AgentCapabilitySpec>() <= 16384);
};
