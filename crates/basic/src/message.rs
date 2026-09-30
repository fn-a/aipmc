//! # 协议消息定义与编解码
//!
//! [`AcpMessage`] 是 ACPs-MCU 协议的核心消息结构，包含帧头（[`AcpHeader`]）、
//! 会话标识（Session-ID）和负载（Payload）三部分。
//!
//! ## 消息结构
//!
//! ```text
//! [AcpHeader (8B)] [Session-ID (4B)] [Payload (变长, 最多 512B)]
//! ```
//!
//! `payload_len = 4 + payload.len()`，即长度字段**包含** Session-ID。
//!
//! ## 使用示例
//!
//! ```rust
//! use basic::{AcpMessage, AcpMethod};
//!
//! // 创建带负载的消息
//! let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
//!
//! // 编码为字节流
//! let encoded = msg.encode().unwrap();
//!
//! // 从字节流解码（会校验帧头、长度与校验和）
//! let decoded = AcpMessage::decode(&encoded).unwrap();
//! assert_eq!(decoded.session_id, 0);
//! assert_eq!(&decoded.payload[..], b"hello");
//! ```
//!
//! ## 编解码契约
//!
//! - **编码自洽**：`payload` 是长度的唯一事实来源；[`AcpMessage::encode`] 与
//!   [`AcpMessage::encode_to`] 写出的 `payload_len` / `magic` / `version` / 校验和
//!   始终由实际负载推导，即便调用方直接改写过 `header` 字段也不会产生自相矛盾的帧。
//! - **解码不 panic**：任何畸形输入（长度不足、Magic 错误、版本不支持、方法码未知、
//!   `payload_len` 越界、数据长度与声明不一致、校验和不匹配）都返回 [`AipError`]。
//! - **零额外拷贝**：[`AcpMessage::encode_to`] 直接写入调用方缓冲区；
//!   [`AcpMessage::validate_checksum`] 不申请任何临时缓冲。
//! - **流式友好**：输入缓冲区可包含多个帧，[`AcpMessage::decode_frame`] 会返回实际
//!   消耗的字节数；[`AcpMessage::decode`] 只解码第一帧。

use crate::AipResult;
use crate::error::AipError;
use crate::protocol::{
    ACP_HEADER_LEN, ACP_MAGIC, ACP_MAX_FRAME_LEN, ACP_MAX_PAYLOAD_LEN, ACP_SESSION_ID_LEN,
    ACP_VERSION, AcpHeader, AcpMethod, AcpStatusCode,
};

/// ACPs-MCU 协议消息
///
/// 一条完整的协议消息由帧头、会话标识和负载三部分组成。
/// 负载最大容量为 512 字节，适用于 MCU 资源受限场景。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AcpMessage {
    /// 协议帧头，包含方法码、标志位、负载长度和校验和
    ///
    /// 注意：`payload_len` / `magic` / `version` 以本结构体的 `payload` 实际长度为准，
    /// 编码时会被自动同步，无需手工维护。
    pub header: AcpHeader,
    /// 会话标识，由服务端在 Initialize 响应中分配，大端序
    pub session_id: u32,
    /// 业务负载数据，最大 512 字节
    pub payload: heapless::Vec<u8, ACP_MAX_PAYLOAD_LEN>,
}

impl AcpMessage {
    /// 创建空负载消息
    ///
    /// `payload_len` 会被正确设置为 4（仅 Session-ID），保证编码结果可被
    /// [`AcpMessage::decode`] 还原。
    ///
    /// # 参数
    /// - `method`: 协议方法码
    /// - `session_id`: 会话标识（初始化阶段为 0）
    pub fn new(method: AcpMethod, session_id: u32) -> Self {
        let mut msg = Self {
            header: AcpHeader::new(method, ACP_SESSION_ID_LEN as u16),
            session_id,
            payload: heapless::Vec::new(),
        };
        msg.refresh_checksum();
        msg
    }

    /// 创建带负载的消息
    ///
    /// 自动计算 `payload_len` 和校验和。
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
            header: AcpHeader::new(method, ACP_SESSION_ID_LEN as u16),
            session_id,
            payload: heapless::Vec::new(),
        };
        msg.payload
            .extend_from_slice(payload)
            .map_err(|_| AipError::BufferOverflow)?;
        msg.refresh_checksum();
        Ok(msg)
    }

    /// 由已解析的帧头与字段组装消息
    ///
    /// 供分帧接收层（如 `aipmc-transport` 的 `FrameReader`）在完成按需读取后直接
    /// 构造消息，避免再把帧头与负载拼成一个连续缓冲区。
    ///
    /// 本方法会校验 `header.payload_len` 与实际负载长度一致，但**不校验校验和**：
    /// 调用方若需要校验和保证，应随后调用 [`AcpMessage::validate_checksum`]。
    ///
    /// # 错误
    /// - [`AipError::InvalidPayload`] - `header.payload_len != 4 + payload.len()`
    /// - [`AipError::BufferOverflow`] - 负载超过 512 字节
    pub fn from_parts(header: AcpHeader, session_id: u32, payload: &[u8]) -> AipResult<Self> {
        if header.payload_len as usize != ACP_SESSION_ID_LEN + payload.len() {
            return Err(AipError::InvalidPayload);
        }
        let mut buf: heapless::Vec<u8, ACP_MAX_PAYLOAD_LEN> = heapless::Vec::new();
        buf.extend_from_slice(payload)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(Self {
            header,
            session_id,
            payload: buf,
        })
    }

    /// 生成当前消息的响应消息
    ///
    /// 响应方法码由 [`AcpMethod::response_method`] 决定：
    /// `Initialize → Initialized`、`Discover → CapabilitySpec`、`Heartbeat → HeartbeatAck`，
    /// 其余方法保持不变。响应负载首字节为状态码，其后为业务数据。
    ///
    /// # 参数
    /// - `status`: 响应状态码
    /// - `payload`: 响应负载数据（不含状态码字节）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 负载超过 511 字节（需为状态码留出 1 字节）
    pub fn response(&self, status: AcpStatusCode, payload: &[u8]) -> AipResult<Self> {
        if payload.len() + 1 > ACP_MAX_PAYLOAD_LEN {
            return Err(AipError::BufferOverflow);
        }
        let mut msg = Self {
            header: AcpHeader::new(
                self.header.method.response_method(),
                ACP_SESSION_ID_LEN as u16,
            ),
            session_id: self.session_id,
            payload: heapless::Vec::new(),
        };
        // 响应负载首字节为状态码
        msg.payload
            .push(status as u8)
            .map_err(|_| AipError::BufferOverflow)?;
        msg.payload
            .extend_from_slice(payload)
            .map_err(|_| AipError::BufferOverflow)?;
        msg.refresh_checksum();
        Ok(msg)
    }

    /// 读取响应负载中的状态码
    ///
    /// 与 [`AcpMessage::response`] 对称：响应负载的首字节为状态码。
    ///
    /// # 错误
    /// - [`AipError::InvalidPayload`] - 负载为空
    /// - [`AipError::UnsupportedStatusCode`] - 首字节不是已知状态码
    pub fn status(&self) -> AipResult<AcpStatusCode> {
        let first = *self.payload.first().ok_or(AipError::InvalidPayload)?;
        AcpStatusCode::try_from(first)
    }

    /// 本帧在字节流中占用的总长度（帧头 + Session-ID + 业务负载）
    pub fn frame_len(&self) -> usize {
        ACP_HEADER_LEN + ACP_SESSION_ID_LEN + self.payload.len()
    }

    /// 构造真正写入字节流的帧头
    ///
    /// `magic` / `version` 强制为协议常量，`payload_len` 由实际负载推导，
    /// 因此编码结果必然能被 [`AcpMessage::decode`] 还原。
    fn wire_header(&self) -> AcpHeader {
        let mut header = self.header.clone();
        header.magic = ACP_MAGIC;
        header.version = ACP_VERSION;
        header.payload_len = (ACP_SESSION_ID_LEN + self.payload.len()) as u16;
        header
    }

    /// 以给定帧头为前缀计算本消息的校验和（不申请任何临时缓冲区）
    fn checksum_with(&self, header: &AcpHeader) -> u8 {
        let bytes = header.to_bytes();
        let mut sum = bytes[..7].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        for b in self.session_id.to_be_bytes() {
            sum = sum.wrapping_add(b);
        }
        for &b in &self.payload {
            sum = sum.wrapping_add(b);
        }
        sum
    }

    /// 构造"线上帧头"：字段由实际负载推导，且校验和总是重算
    ///
    /// 供编码路径使用，保证 `encode()` / `encode_to()` 输出必然是合法帧。
    fn wire_header_with_checksum(&self) -> AcpHeader {
        let mut header = self.wire_header();
        header.checksum = self.checksum_with(&header);
        header
    }

    /// 依据实际负载同步帧头字段（`payload_len`、`magic`、`version`）并重算校验和
    ///
    /// 在直接修改 `session_id` / `payload` / `header.flags` 之后调用即可；
    /// 即便忘记调用，[`AcpMessage::encode`] 仍会按实际负载编码，只是结构体内的
    /// `header` 字段会与线上字节不一致。
    pub fn refresh_checksum(&mut self) {
        self.header = self.wire_header_with_checksum();
    }

    /// 将完整消息编码为字节流（帧头 + Session-ID + Payload）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 编码结果超过单帧上限（理论不可达）
    pub fn encode(&self) -> AipResult<heapless::Vec<u8, ACP_MAX_FRAME_LEN>> {
        let mut buf: heapless::Vec<u8, ACP_MAX_FRAME_LEN> = heapless::Vec::new();
        buf.extend_from_slice(&self.wire_header_with_checksum().to_bytes())
            .map_err(|_| AipError::BufferOverflow)?;
        buf.extend_from_slice(&self.session_id.to_be_bytes())
            .map_err(|_| AipError::BufferOverflow)?;
        buf.extend_from_slice(&self.payload)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(buf)
    }

    /// 将完整消息编码进调用方提供的缓冲区（零额外拷贝）
    ///
    /// # 参数
    /// - `out`: 目标缓冲区，长度需不小于 [`AcpMessage::frame_len`]
    ///
    /// # 返回
    /// 实际写入的字节数，等于 [`AcpMessage::frame_len`]
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - `out` 空间不足
    pub fn encode_to(&self, out: &mut [u8]) -> AipResult<usize> {
        let total = self.frame_len();
        if out.len() < total {
            return Err(AipError::BufferOverflow);
        }
        let header = self.wire_header_with_checksum();
        let sid_end = ACP_HEADER_LEN + ACP_SESSION_ID_LEN;
        out[..ACP_HEADER_LEN].copy_from_slice(&header.to_bytes());
        out[ACP_HEADER_LEN..sid_end].copy_from_slice(&self.session_id.to_be_bytes());
        out[sid_end..total].copy_from_slice(&self.payload);
        Ok(total)
    }

    /// 从字节流解码消息，并校验帧校验和
    ///
    /// 输入缓冲区允许包含多于一个帧的数据（例如 UART 连续读入的字节流），
    /// 本方法只解码第一帧并忽略其后的数据；若需要知道消耗了多少字节，
    /// 请使用 [`AcpMessage::decode_frame`]。
    ///
    /// # 参数
    /// - `buf`: 至少包含一个完整协议帧的字节流
    ///
    /// # 错误
    /// - [`AipError::InvalidMessage`] - 数据长度不足一个最小帧
    /// - [`AipError::InvalidHeader`] - 帧头 Magic 不匹配
    /// - [`AipError::UnsupportedVersion`] - 协议版本不支持
    /// - [`AipError::UnsupportedMethod`] - 方法码无效
    /// - [`AipError::InvalidPayload`] - `payload_len` 越界或数据长度与声明不一致
    /// - [`AipError::BufferOverflow`] - 负载超过 512 字节
    /// - [`AipError::ChecksumMismatch`] - 校验和不匹配（数据可能在传输中损坏）
    pub fn decode(buf: &[u8]) -> AipResult<Self> {
        let (msg, _) = Self::decode_frame(buf)?;
        Ok(msg)
    }

    /// 从字节流解码消息，并返回消耗的字节数
    ///
    /// 适用于流式/分帧接收场景：调用方据此把已消费的字节从缓冲区中移除。
    ///
    /// # 返回
    /// `(消息, 消耗字节数)`，消耗字节数等于 `ACP_HEADER_LEN + payload_len`
    ///
    /// # 错误
    /// 与 [`AcpMessage::decode`] 相同
    pub fn decode_frame(buf: &[u8]) -> AipResult<(Self, usize)> {
        let msg = Self::decode_unverified(buf)?;
        if !msg.validate_checksum() {
            return Err(AipError::ChecksumMismatch);
        }
        let consumed = msg.frame_len();
        Ok((msg, consumed))
    }

    /// 从字节流解码消息，但**跳过校验和校验**
    ///
    /// 仅在调用方需要自行处理校验失败（例如先解析状态码再决定是否丢弃）时使用。
    /// 其余字段（Magic / 版本 / 方法码 / `payload_len` / 数据长度）仍然全部校验，
    /// 因此本方法同样不会 panic。
    ///
    /// # 错误
    /// 与 [`AcpMessage::decode`] 相同，但不包含 [`AipError::ChecksumMismatch`]
    pub fn decode_unverified(buf: &[u8]) -> AipResult<Self> {
        // 最小帧长度 = 帧头(8B) + Session-ID(4B)
        if buf.len() < ACP_HEADER_LEN + ACP_SESSION_ID_LEN {
            return Err(AipError::InvalidMessage);
        }
        // 解码帧头（其中已校验 magic / version / method / payload_len 区间）
        let header_bytes: [u8; ACP_HEADER_LEN] = buf[..ACP_HEADER_LEN]
            .try_into()
            .map_err(|_| AipError::InvalidHeader)?;
        let header = AcpHeader::from_bytes(&header_bytes)?;

        let frame_len = header.frame_len();
        if buf.len() < frame_len {
            return Err(AipError::InvalidPayload);
        }

        // 解码 Session-ID（大端序，紧跟帧头之后）
        let sid_end = ACP_HEADER_LEN + ACP_SESSION_ID_LEN;
        let session_id = u32::from_be_bytes(
            buf[ACP_HEADER_LEN..sid_end]
                .try_into()
                .map_err(|_| AipError::InvalidMessage)?,
        );

        // body_len 由 from_bytes 保证 <= ACP_MAX_PAYLOAD_LEN
        Self::from_parts(header, session_id, &buf[sid_end..frame_len])
    }

    /// 验证消息校验和是否正确
    ///
    /// 以实际负载重算校验和并与帧头中的值比较，用于检测传输中的数据损坏。
    /// 不申请任何临时缓冲区。
    pub fn validate_checksum(&self) -> bool {
        let header = self.wire_header();
        self.checksum_with(&header) == header.checksum
    }
}
