//! # UART 传输实现
//!
//! [`UartTransport`] 基于 `embedded-io` 的 `Read` + `Write` trait 实现，
//! 适用于 MCU 之间的 UART 有线通信场景。
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use aipmc::transport::UartTransport;
//! use aipmc::basic::{AcpMessage, AcpMethod};
//!
//! let uart = /* 实现 embedded_io::Read + embedded_io::Write 的外设 */;
//! let mut transport = UartTransport::new(uart);
//!
//! // 发送消息
//! let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
//! transport.send(&msg).unwrap();
//!
//! // 接收消息
//! let received = transport.recv().unwrap();
//! ```

use basic::error::{AipError, AipResult};
use basic::message::AcpMessage;
use basic::protocol::{AcpHeader, ACP_HEADER_LEN};
use embedded_io::{Read, Write};

/// UART 传输层
///
/// 泛型参数 `T` 为实现了 `embedded_io::Read` + `embedded_io::Write` 的 UART 外设。
/// 内部按帧格式逐段读取：先读 8 字节帧头，再根据 payload_len 读取剩余负载。
pub struct UartTransport<T> {
    /// 底层 UART 外设
    inner: T,
}

impl<T> UartTransport<T>
where
    T: Read + Write,
{
    /// 创建 UART 传输层实例
    ///
    /// # 参数
    /// - `uart`: 实现了 `Read` + `Write` 的 UART 外设
    pub fn new(uart: T) -> Self {
        Self { inner: uart }
    }

    /// 发送协议消息
    ///
    /// 将消息编码为字节流后通过 UART 发送。
    ///
    /// # 错误
    /// - [`AipError::TransportError`] - UART 写入失败
    pub fn send(&mut self, msg: &AcpMessage) -> AipResult<()> {
        let encoded = msg.encode()?;
        self.inner
            .write(&encoded)
            .map_err(|_| AipError::TransportError)?;
        Ok(())
    }

    /// 接收协议消息
    ///
    /// 按帧格式逐段读取：先读 8 字节帧头，解析 payload_len，
    /// 再读取 Session-ID (4B) + Payload 数据，最后解码为 [`AcpMessage`]。
    ///
    /// # 错误
    /// - [`AipError::TransportError`] - UART 读取失败
    /// - [`AipError::InvalidHeader`] - 帧头 Magic 不匹配
    /// - [`AipError::InvalidPayload`] - 负载长度不匹配
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        // 第一步：读取帧头（固定 8 字节）
        let mut header_buf = [0u8; ACP_HEADER_LEN];
        self.inner
            .read(&mut header_buf)
            .map_err(|_| AipError::TransportError)?;
        let header = AcpHeader::from_bytes(&header_buf)?;

        // 第二步：读取 Session-ID (4B) + Payload
        let payload_total = header.payload_len as usize + 4;
        let mut payload_buf = [0u8; 520];
        let read_len = payload_total.min(payload_buf.len());
        self.inner
            .read(&mut payload_buf[..read_len])
            .map_err(|_| AipError::TransportError)?;

        // 第三步：拼接帧头+负载，解码为消息
        let mut full_buf: heapless::Vec<u8, 540> = heapless::Vec::new();
        full_buf
            .extend_from_slice(&header_buf)
            .map_err(|_| AipError::BufferOverflow)?;
        full_buf
            .extend_from_slice(&payload_buf[..read_len])
            .map_err(|_| AipError::BufferOverflow)?;

        AcpMessage::decode(&full_buf)
    }

    /// 消费传输层，返回底层 UART 外设
    pub fn into_inner(self) -> T {
        self.inner
    }
}
