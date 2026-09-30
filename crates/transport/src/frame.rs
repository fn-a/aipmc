//! # 分帧读取与 IO 错误映射
//!
//! [`FrameReader`] 从任意 `embedded_io::Read` 数据源按帧读取 ACPs-MCU 协议帧，
//! 是 UART / SPI / 其他字节流通道共用的分帧实现。
//!
//! ## 分帧流程
//!
//! 1. `read_exact` 读取 8 字节帧头，解析出 `payload_len`
//! 2. `read_exact` 读取 `payload_len` 字节（Session-ID + 业务负载）
//! 3. 校验帧头字段与校验和，组装为 [`AcpMessage`]
//!
//! 每一步都使用 `read_exact`，因此不会出现"只读到半个帧"的静默错误；帧头声明的
//! 长度超过单帧上限时返回 [`AipError::InvalidPayload`]，不会截断后继续。

use basic::error::{AipError, AipResult};
use basic::message::AcpMessage;
use basic::protocol::{ACP_HEADER_LEN, ACP_MAX_PAYLOAD_LEN, ACP_SESSION_ID_LEN, AcpHeader};
use embedded_io::{ErrorKind, Read, ReadExactError};

/// 帧体（Session-ID + 业务负载）缓冲区长度
pub const FRAME_BODY_LEN: usize = ACP_SESSION_ID_LEN + ACP_MAX_PAYLOAD_LEN;

/// 将底层 `embedded_io` 错误映射为统一错误类型
///
/// 映射规则：
/// - `TimedOut` → [`AipError::Timeout`]
/// - `InvalidInput` / `InvalidData` → [`AipError::InvalidMessage`]
/// - `Unsupported` → [`AipError::NotImplemented`]
/// - 其他 → [`AipError::TransportError`]
pub fn map_io_error<E: embedded_io::Error>(err: E) -> AipError {
    match err.kind() {
        ErrorKind::TimedOut => AipError::Timeout,
        ErrorKind::InvalidInput | ErrorKind::InvalidData => AipError::InvalidMessage,
        ErrorKind::Unsupported => AipError::NotImplemented,
        _ => AipError::TransportError,
    }
}

/// 将 `read_exact` 的错误映射为统一错误类型
///
/// 与 [`map_io_error`] 的区别是额外区分"流提前结束"：
/// 对端断开或数据被截断时返回 [`AipError::UnexpectedEof`]。
pub fn map_read_exact_error<E: embedded_io::Error>(err: ReadExactError<E>) -> AipError {
    match err {
        ReadExactError::UnexpectedEof => AipError::UnexpectedEof,
        ReadExactError::Other(inner) => map_io_error(inner),
    }
}

/// 按帧读取 ACPs-MCU 协议消息
///
/// 泛型参数 `T` 为实现了 `embedded_io::Read` 的字节流数据源，
/// 可以是 UART、SPI、TCP socket 等任何"有序字节流"。
///
/// 内部只使用固定大小的栈缓冲区（8 + 516 字节），不进行动态分配。
pub struct FrameReader<T> {
    /// 底层字节流
    inner: T,
}

impl<T> FrameReader<T>
where
    T: Read,
{
    /// 创建分帧读取器
    ///
    /// # 参数
    /// - `inner`: 实现了 `embedded_io::Read` 的字节流数据源
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// 读取并解析一个完整的协议帧
    ///
    /// 调用会阻塞直到读完一帧或发生错误。可安全地在循环中反复调用。
    ///
    /// # 错误
    /// - [`AipError::UnexpectedEof`] - 流在帧结束前结束
    /// - [`AipError::Timeout`] - 底层读取超时
    /// - [`AipError::TransportError`] - 底层读写失败
    /// - [`AipError::InvalidHeader`] - Magic 不匹配
    /// - [`AipError::UnsupportedVersion`] - 协议版本不支持
    /// - [`AipError::UnsupportedMethod`] - 方法码无效
    /// - [`AipError::InvalidPayload`] - `payload_len` 越界
    /// - [`AipError::ChecksumMismatch`] - 校验和不匹配
    pub fn read_frame(&mut self) -> AipResult<AcpMessage> {
        // 第一步：读取固定 8 字节帧头
        let mut header_buf = [0u8; ACP_HEADER_LEN];
        self.inner
            .read_exact(&mut header_buf)
            .map_err(map_read_exact_error)?;
        let header = AcpHeader::from_bytes(&header_buf)?;

        // 第二步：读取 Session-ID + 业务负载（长度由帧头声明，已校验上限）
        let body_len = ACP_SESSION_ID_LEN + header.body_len();
        let mut body = [0u8; FRAME_BODY_LEN];
        self.inner
            .read_exact(&mut body[..body_len])
            .map_err(map_read_exact_error)?;

        // 第三步：校验和覆盖帧头前 7 字节 + 帧体
        if header.compute_checksum(&body[..body_len]) != header.checksum {
            return Err(AipError::ChecksumMismatch);
        }

        // 第四步：组装消息（from_parts 会再次确认 payload_len 与实际负载一致）
        let mut sid = [0u8; ACP_SESSION_ID_LEN];
        sid.copy_from_slice(&body[..ACP_SESSION_ID_LEN]);
        AcpMessage::from_parts(
            header,
            u32::from_be_bytes(sid),
            &body[ACP_SESSION_ID_LEN..body_len],
        )
    }

    /// 消费读取器，返回底层数据源
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// 获取底层数据源的可变引用
    ///
    /// 便于在同一个设备上复用读写句柄（UART 的 `Read` 与 `Write` 通常是同一外设）。
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
