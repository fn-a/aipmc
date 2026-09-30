//! # 字节流传输层
//!
//! [`StreamTransport`] 把任意 `embedded_io::Read + Write` 的字节流封装为
//! "收发 ACPs 协议消息"的传输通道，是 UART / SPI 等有序字节流通道的共用实现。
//!
//! - 发送：把消息编码进固定栈缓冲区（[`ACP_MAX_FRAME_LEN`] 字节）后用
//!   `write_all` 一次写出，不会出现"只写了一半"的静默截断
//! - 接收：委托 [`FrameReader`] 按帧 `read_exact`，
//!   校验长度与校验和
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use transport::UartTransport;
//! use aipmc::basic::{AcpMessage, AcpMethod};
//!
//! let uart = /* 实现 embedded_io::Read + embedded_io::Write 的外设 */;
//! let mut transport = UartTransport::new(uart);
//!
//! // 发送消息
//! let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
//! transport.send(&msg).unwrap();
//!
//! // 接收消息（阻塞直到读满一帧）
//! let received = transport.recv().unwrap();
//! ```

use basic::error::AipResult;
use basic::message::AcpMessage;
use basic::protocol::ACP_MAX_FRAME_LEN;
use embedded_io::{Read, Write};

use crate::frame::{FrameReader, map_io_error};

/// 字节流传输通道
///
/// 泛型参数 `T` 需同时实现 `embedded_io::Read` 与 `embedded_io::Write`
/// （UART / SPI 外设通常由同一个句柄提供这两组操作）。
pub struct StreamTransport<T> {
    /// 分帧读取器，同时持有底层设备句柄
    reader: FrameReader<T>,
}

impl<T> StreamTransport<T>
where
    T: Read + Write,
{
    /// 创建字节流传输通道
    ///
    /// # 参数
    /// - `stream`: 实现了 `embedded_io::Read + Write` 的字节流设备
    pub fn new(stream: T) -> Self {
        Self {
            reader: FrameReader::new(stream),
        }
    }

    /// 发送协议消息
    ///
    /// 消息先编码进栈缓冲区，再经 `write_all` 完整写出。
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`](basic::error::AipError::BufferOverflow) - 编码结果超出单帧上限
    /// - [`AipError::Timeout`](basic::error::AipError::Timeout) /
    ///   [`AipError::TransportError`](basic::error::AipError::TransportError) - 底层写入失败
    pub fn send(&mut self, msg: &AcpMessage) -> AipResult<()> {
        let mut buf = [0u8; ACP_MAX_FRAME_LEN];
        let len = msg.encode_to(&mut buf)?;
        self.reader
            .inner_mut()
            .write_all(&buf[..len])
            .map_err(map_io_error)
    }

    /// 接收协议消息
    ///
    /// 阻塞直到读满一帧。错误语义见 [`FrameReader::read_frame`]。
    ///
    /// # 错误
    /// - [`AipError::UnexpectedEof`](basic::error::AipError::UnexpectedEof) - 流在帧结束前结束
    /// - [`AipError::ChecksumMismatch`](basic::error::AipError::ChecksumMismatch) - 校验和不匹配
    /// - [`AipError::InvalidHeader`](basic::error::AipError::InvalidHeader) /
    ///   [`AipError::UnsupportedVersion`](basic::error::AipError::UnsupportedVersion) /
    ///   [`AipError::InvalidPayload`](basic::error::AipError::InvalidPayload) - 帧头或长度非法
    /// - [`AipError::TransportError`](basic::error::AipError::TransportError) /
    ///   [`AipError::Timeout`](basic::error::AipError::Timeout) - 底层读取失败
    pub fn recv(&mut self) -> AipResult<AcpMessage> {
        self.reader.read_frame()
    }

    /// 消费传输通道，返回底层设备
    pub fn into_inner(self) -> T {
        self.reader.into_inner()
    }

    /// 获取底层设备的可变引用
    pub fn inner_mut(&mut self) -> &mut T {
        self.reader.inner_mut()
    }
}
