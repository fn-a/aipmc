//! # 消息编解码器
//!
//! [`AcpCodec`] 提供协议消息的便捷编解码接口，是对 [`AcpMessage`] 方法的薄封装。
//!
//! 典型用法：
//! ```rust
//! use aipmc::basic::{AcpCodec, AcpMessage, AcpMethod};
//!
//! let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
//! let encoded = AcpCodec::encode(&msg).unwrap();
//! let decoded = AcpCodec::decode(&encoded).unwrap();
//! ```

use crate::error::AipResult;
use crate::message::AcpMessage;

/// ACPs-MCU 协议编解码器
///
/// 提供协议消息与字节流之间的转换功能。
pub struct AcpCodec;

impl AcpCodec {
    /// 将消息编码为字节流
    ///
    /// 等价于 `msg.encode()`，提供更语义化的调用方式。
    pub fn encode(msg: &AcpMessage) -> AipResult<heapless::Vec<u8, 540>> {
        msg.encode()
    }

    /// 从字节流解码消息
    ///
    /// 等价于 `AcpMessage::decode(buf)`，提供更语义化的调用方式。
    pub fn decode(buf: &[u8]) -> AipResult<AcpMessage> {
        AcpMessage::decode(buf)
    }
}
