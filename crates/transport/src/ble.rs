//! # BLE 传输实现
//!
//! BLE 的 GATT 特征值天然是**消息边界明确**的（一次 notify / write 即一个报文），
//! 因此对接 BLE 协议栈时通常不需要额外的分帧逻辑，直接对
//! [`AcpMessage::encode_to`](basic::message::AcpMessage::encode_to) 的输出做单次
//! 特征值写入即可。
//!
//! ## 当前状态
//!
//! **未实现**。本模块只提供占位 API：所有收发方法返回
//! [`AipError::NotImplemented`]，不会 panic。

use basic::error::{AipError, AipResult};
use basic::message::AcpMessage;

/// BLE 传输通道
///
/// 通过低功耗蓝牙实现 ACPs 协议消息的无线传输，
/// 适用于电池供电、低功耗要求的智能体场景。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BleTransport;

impl BleTransport {
    /// 创建 BLE 传输层实例
    ///
    /// 注意：当前版本未对接 BLE 协议栈，实例化后调用 `send` / `recv`
    /// 会返回 [`AipError::NotImplemented`]。
    pub const fn new() -> Self {
        Self
    }

    /// 通过 BLE 发送协议消息
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`] - 尚未对接 BLE 协议栈（当前总是返回）
    pub fn send(&mut self, _msg: &AcpMessage) -> AipResult<()> {
        Err(AipError::NotImplemented)
    }

    /// 通过 BLE 接收协议消息
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`] - 尚未对接 BLE 协议栈（当前总是返回）
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        Err(AipError::NotImplemented)
    }
}
