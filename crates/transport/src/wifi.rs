//! # WiFi 传输实现（ESP-32 专用）
//!
//! WiFi 传输层依赖 ESP-32 的 `esp-wifi` crate，适用于无线局域网通信场景。
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 `esp-wifi` 后完善。

use basic::error::AipResult;
use basic::message::AcpMessage;

/// WiFi 传输层（ESP-32 专用）
///
/// 通过 ESP-32 的 WiFi 模块实现 ACPs 协议消息的无线传输，
/// 适用于需要局域网通信的智能体场景。
pub struct WifiTransport;

impl WifiTransport {
    /// 创建 WiFi 传输层实例
    pub fn new() -> Self {
        Self
    }

    /// 通过 WiFi 发送协议消息
    ///
    /// TODO: 对接 esp-wifi 实现
    pub fn send(&mut self, _msg: &AcpMessage) -> AipResult<()> {
        todo!("WiFi transport implementation for ESP-32")
    }

    /// 通过 WiFi 接收协议消息
    ///
    /// TODO: 对接 esp-wifi 实现
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        todo!("WiFi transport implementation for ESP-32")
    }
}
