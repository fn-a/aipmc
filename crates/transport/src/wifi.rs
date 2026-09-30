//! # WiFi 传输实现（ESP-32 专用）
//!
//! WiFi 通道最终仍然表现为一条**有序字节流**（TCP/UDP 之上的自有分帧），
//! 因此分帧逻辑复用 [`StreamTransport`](crate::stream::StreamTransport)。
//!
//! ## 当前状态
//!
//! **未实现**。本模块只提供占位 API：所有收发方法返回
//! [`AipError::NotImplemented`]，不会 panic。对接 `esp-wifi` 后，把 socket 封装为
//! 实现 `embedded_io::Read + Write` 的字节流句柄即可直接替换实现。

use basic::error::{AipError, AipResult};
use basic::message::AcpMessage;

/// WiFi 传输通道（ESP-32 专用）
///
/// 通过 ESP-32 的 WiFi 模块实现 ACPs 协议消息的无线传输，
/// 适用于需要局域网通信的智能体场景。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WifiTransport;

impl WifiTransport {
    /// 创建 WiFi 传输层实例
    ///
    /// 注意：当前版本未对接 `esp-wifi`，实例化后调用 `send` / `recv`
    /// 会返回 [`AipError::NotImplemented`]。
    pub const fn new() -> Self {
        Self
    }

    /// 通过 WiFi 发送协议消息
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`] - 尚未对接 `esp-wifi`（当前总是返回）
    pub fn send(&mut self, _msg: &AcpMessage) -> AipResult<()> {
        Err(AipError::NotImplemented)
    }

    /// 通过 WiFi 接收协议消息
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`] - 尚未对接 `esp-wifi`（当前总是返回）
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        Err(AipError::NotImplemented)
    }
}
