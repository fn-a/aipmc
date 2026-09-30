//! # 消息编解码器
//!
//! [`AcpCodec`] 提供协议消息的便捷编解码接口，是对 [`AcpMessage`] 方法的薄封装。
//!
//! 本模块是**无状态**的：它不持有缓冲区，也不感知字节流边界。需要从
//! `embedded_io::Read` 逐帧收取数据时，请使用 `aipmc-transport` 提供的
//! `FrameReader`（见 `aipmc_transport::frame`）。
//!
//! 典型用法：
//! ```rust
//! use basic::{AcpCodec, AcpMessage, AcpMethod};
//!
//! let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
//! let encoded = AcpCodec::encode(&msg).unwrap();
//! let decoded = AcpCodec::decode(&encoded).unwrap();
//! ```

use crate::error::AipResult;
use crate::message::AcpMessage;
use crate::protocol::ACP_MAX_FRAME_LEN;

/// ACPs-MCU 协议编解码器
///
/// 提供协议消息与字节流之间的转换功能。
pub struct AcpCodec;

impl AcpCodec {
    /// 将消息编码为字节流
    ///
    /// 等价于 `msg.encode()`，提供更语义化的调用方式。
    pub fn encode(msg: &AcpMessage) -> AipResult<heapless::Vec<u8, ACP_MAX_FRAME_LEN>> {
        msg.encode()
    }

    /// 将消息编码进调用方提供的缓冲区，返回写入的字节数
    ///
    /// 等价于 `msg.encode_to(out)`，适合零动态分配的 MCU 场景。
    pub fn encode_to(msg: &AcpMessage, out: &mut [u8]) -> AipResult<usize> {
        msg.encode_to(out)
    }

    /// 从字节流解码消息（校验校验和）
    ///
    /// 等价于 `AcpMessage::decode(buf)`，提供更语义化的调用方式。
    pub fn decode(buf: &[u8]) -> AipResult<AcpMessage> {
        AcpMessage::decode(buf)
    }

    /// 从字节流解码消息，并返回消耗的字节数
    ///
    /// 等价于 `AcpMessage::decode_frame(buf)`，用于连续到达的字节流分帧。
    pub fn decode_frame(buf: &[u8]) -> AipResult<(AcpMessage, usize)> {
        AcpMessage::decode_frame(buf)
    }
}
