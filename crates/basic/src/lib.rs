//! # aipmc-basic: AIP 智能体互联协议核心库
//!
//! 基于 GB/Z 185 标准的 ACPs（Agent Collaboration Protocol Suite）协议核心实现，
//! 面向资源受限的微控制器（MCU）场景，采用 `no_std` 设计。
//!
//! ## 核心模块
//!
//! - [`agent`] - 智能体卡片（AgentCard）定义，描述智能体的身份、能力与技能
//! - [`skill`] - 智能体技能（AgentSkill）定义，描述可执行的具体功能
//! - [`protocol`] - ACPs-MCU 二进制协议定义，包含方法码、状态码、帧头格式
//! - [`message`] - 协议消息（AcpMessage）编解码，包含帧头 + 会话ID + 负载
//! - [`codec`] - 消息编解码器（AcpCodec），提供 encode/decode 便捷接口
//! - [`error`] - 统一错误类型（AipError），涵盖协议/编解码/传输/会话等错误

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod agent;
pub mod codec;
pub mod error;
pub mod message;
pub mod protocol;
pub mod skill;

pub use agent::{AgentCard, AgentCapability};
pub use codec::AcpCodec;
pub use error::{AipError, AipResult};
pub use message::AcpMessage;
pub use protocol::{AcpMethod, AcpProtocol, AcpStatusCode};
pub use skill::AgentSkill;
