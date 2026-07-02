//! # BLE 传输实现
//!
//! BLE（低功耗蓝牙）传输层适用于电池供电场景的短距离无线通信。
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 BLE 协议栈后完善。

use basic::error::AipResult;
use basic::message::AcpMessage;

/// BLE 传输层
///
/// 通过低功耗蓝牙实现 ACPs 协议消息的无线传输，
/// 适用于电池供电、低功耗要求的智能体场景。
pub struct BleTransport;

impl BleTransport {
    /// 创建 BLE 传输层实例
    pub fn new() -> Self {
        Self
    }

    /// 通过 BLE 发送协议消息
    ///
    /// TODO: 对接 BLE 协议栈实现
    pub fn send(&mut self, _msg: &AcpMessage) -> AipResult<()> {
        todo!("BLE transport implementation")
    }

    /// 通过 BLE 接收协议消息
    ///
    /// TODO: 对接 BLE 协议栈实现
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        todo!("BLE transport implementation")
    }
}
