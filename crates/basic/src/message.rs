//! # 协议消息定义与编解码
//!
//! [`AcpMessage`] 是 ACPs-MCU 协议的核心消息结构，包含帧头（[`AcpHeader`]）、
//! 会话标识（Session-ID）和负载（Payload）三部分。
//!
//! ## 消息结构
//!
//! ```text
//! [AcpHeader (8B)] [Session-ID (4B)] [Payload (变长)]
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use aipmc::basic::{AcpMessage, AcpMethod};
//!
//! // 创建带负载的消息
//! let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
//!
//! // 编码为字节流
//! let encoded = msg.encode().unwrap();
//!
//! // 从字节流解码
//! let decoded = AcpMessage::decode(&encoded).unwrap();
//! assert_eq!(decoded.session_id, 0);
//! ```

use crate::error::AipError;
use crate::protocol::{AcpHeader, AcpMethod, AcpStatusCode, ACP_HEADER_LEN};
use crate::AipResult;

/// ACPs-MCU 协议消息
///
/// 一条完整的协议消息由帧头、会话标识和负载三部分组成。
/// 负载最大容量为 512 字节，适用于 MCU 资源受限场景。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AcpMessage {
    /// 协议帧头，包含方法码、标志位、负载长度和校验和
    pub header: AcpHeader,
    /// 会话标识，由服务端在 Initialize 响应中分配，大端序
    pub session_id: u32,
    /// 业务负载数据，最大 512 字节
    pub payload: heapless::Vec<u8, 512>,
}

impl AcpMessage {
    /// 创建空负载消息
    ///
    /// # 参数
    /// - `method`: 协议方法码
    /// - `session_id`: 会话标识（初始化阶段为 0）
    pub fn new(method: AcpMethod, session_id: u32) -> Self {
        Self {
            header: AcpHeader::new(method, 0),
            session_id,
            payload: heapless::Vec::new(),
        }
    }

    /// 创建带负载的消息
    ///
    /// 自动计算 payload_len 和校验和。
    ///
    /// # 参数
    /// - `method`: 协议方法码
    /// - `session_id`: 会话标识
    /// - `payload`: 业务负载数据
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 负载超过 512 字节
    pub fn with_payload(method: AcpMethod, session_id: u32, payload: &[u8]) -> AipResult<Self> {
        let mut msg = Self {
            header: AcpHeader::new(method, (payload.len() + 4) as u16),
            session_id,
            payload: heapless::Vec::new(),
        };
        msg.payload
            .extend_from_slice(payload)
            .map_err(|_| AipError::BufferOverflow)?;
        // 计算并填充校验和
        msg.header.checksum = msg.header.compute_checksum(&msg.encode_payload());
        Ok(msg)
    }

    /// 生成当前消息的响应消息
    ///
    /// 对于 Initialize 请求自动映射为 Initialized 响应，
    /// 其他方法保持不变。响应负载首字节为状态码。
    ///
    /// # 参数
    /// - `status`: 响应状态码
    /// - `payload`: 响应负载数据
    pub fn response(&self, status: AcpStatusCode, payload: &[u8]) -> AipResult<Self> {
        let method = match self.header.method {
            AcpMethod::Initialize => AcpMethod::Initialized,
            other => other,
        };
        let mut msg = Self {
            header: AcpHeader::new(method, (payload.len() + 5) as u16),
            session_id: self.session_id,
            payload: heapless::Vec::new(),
        };
        // 响应负载首字节为状态码
        msg.payload
            .extend_from_slice(&[status as u8])
            .map_err(|_| AipError::BufferOverflow)?;
        msg.payload
            .extend_from_slice(payload)
            .map_err(|_| AipError::BufferOverflow)?;
        msg.header.checksum = msg.header.compute_checksum(&msg.encode_payload());
        Ok(msg)
    }

    /// 将 Session-ID + Payload 编码为连续字节流（用于校验和计算和传输编码）
    fn encode_payload(&self) -> heapless::Vec<u8, 520> {
        let mut buf: heapless::Vec<u8, 520> = heapless::Vec::new();
        buf.extend_from_slice(&self.session_id.to_be_bytes()).ok();
        buf.extend_from_slice(&self.payload).ok();
        buf
    }

    /// 将完整消息编码为字节流（帧头 + Session-ID + Payload）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 编码结果超过 540 字节
    pub fn encode(&self) -> AipResult<heapless::Vec<u8, 540>> {
        let payload_bytes = self.encode_payload();
        let mut buf: heapless::Vec<u8, 540> = heapless::Vec::new();
        let header_bytes = self.header.to_bytes();
        buf.extend_from_slice(&header_bytes)
            .map_err(|_| AipError::BufferOverflow)?;
        buf.extend_from_slice(&payload_bytes)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(buf)
    }

    /// 从字节流解码消息
    ///
    /// # 参数
    /// - `buf`: 完整的协议帧字节流（至少包含帧头 8 字节 + Session-ID 4 字节）
    ///
    /// # 错误
    /// - [`AipError::InvalidMessage`] - 数据长度不足
    /// - [`AipError::InvalidHeader`] - 帧头 Magic 不匹配
    /// - [`AipError::UnsupportedMethod`] - 方法码无效
    /// - [`AipError::InvalidPayload`] - 负载长度与帧头声明不一致
    /// - [`AipError::BufferOverflow`] - 负载超过 512 字节
    pub fn decode(buf: &[u8]) -> AipResult<Self> {
        // 最小帧长度 = 帧头(8B) + Session-ID(4B)
        if buf.len() < ACP_HEADER_LEN + 4 {
            return Err(AipError::InvalidMessage);
        }
        // 解码帧头
        let header_bytes: [u8; ACP_HEADER_LEN] =
            buf[..ACP_HEADER_LEN].try_into().map_err(|_| AipError::InvalidHeader)?;
        let header = AcpHeader::from_bytes(&header_bytes)?;
        // 解码 Session-ID（大端序，紧跟帧头之后）
        let session_id = u32::from_be_bytes(
            buf[ACP_HEADER_LEN..ACP_HEADER_LEN + 4]
                .try_into()
                .map_err(|_| AipError::InvalidMessage)?,
        );
        // 解码负载（Session-ID 之后的 payload_len - 4 字节）
        let payload_start = ACP_HEADER_LEN + 4;
        let payload_end = payload_start + header.payload_len as usize - 4;
        if payload_end > buf.len() {
            return Err(AipError::InvalidPayload);
        }
        let mut payload: heapless::Vec<u8, 512> = heapless::Vec::new();
        payload
            .extend_from_slice(&buf[payload_start..payload_end])
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(Self {
            header,
            session_id,
            payload,
        })
    }

    /// 验证消息校验和是否正确
    ///
    /// 重新计算校验和并与帧头中的值比较，用于检测传输中的数据损坏。
    pub fn validate_checksum(&self) -> bool {
        let payload_bytes = self.encode_payload();
        self.header.compute_checksum(&payload_bytes) == self.header.checksum
    }
}
