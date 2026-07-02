//! # ACPs-MCU 协议定义
//!
//! 定义基于 GB/Z 185 标准适配 MCU 场景的二进制协议常量、方法码、状态码和帧头格式。
//!
//! ## 帧格式
//!
//! ```text
//! +--------+--------+--------+--------+--------+--------+--------+--------+
//! | Magic0 | Magic1 | Ver    | Method | Flags  | LenHi  | LenLo  | Chksum |
//! +--------+--------+--------+--------+--------+--------+--------+--------+
//! | SID0   | SID1   | SID2   | SID3   | Payload ...                          |
//! +--------+--------+--------+--------+-------------------------------------+
//! ```
//!
//! ## 方法码映射
//!
//! | ACPs 原语            | MCU Code | 说明             |
//! |----------------------|----------|------------------|
//! | initialize           | 0x01     | 会话初始化       |
//! | notifications/init   | 0x02     | 初始化确认       |
//! | message/stream       | 0x03     | 消息流式传输     |
//! | tasks/cancel         | 0x04     | 取消任务         |
//! | clearContext         | 0x05     | 清除上下文       |
//! | authorize            | 0x06     | 授权认证         |
//! | deauthorize          | 0x07     | 取消授权         |
//! | push                 | 0x08     | 服务端推送       |
//! | discover             | 0x10     | 智能体发现       |
//! | heartbeat            | 0x20     | 心跳保活         |

use crate::error::AipError;
use crate::AipResult;

/// 协议版本号，当前为 1
pub const ACP_VERSION: u8 = 1;

/// 协议帧 Magic 标识 `0xA1A2`，用于识别 ACPs-MCU 协议帧
pub const ACP_MAGIC: u16 = 0xA1A2;

/// 帧头固定长度（8 字节）
pub const ACP_HEADER_LEN: usize = 8;

/// ACPs 协议方法码
///
/// 包含 ACPs 标准方法（initialize/message/stream 等）和 MCU 扩展方法（discover/heartbeat）。
/// 方法码采用单字节编码，0x01~0x0F 为标准方法，0x10~0x2F 为 MCU 扩展方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AcpMethod {
    /// 客户端发起会话初始化（对应 ACPs initialize）
    Initialize = 0x01,
    /// 服务端确认初始化完成，返回 Session-ID（对应 ACPs notifications/initialized）
    Initialized = 0x02,
    /// 消息流式传输，支持多帧分片（对应 ACPs message/stream）
    MessageStream = 0x03,
    /// 取消当前正在执行的任务（对应 ACPs tasks/cancel）
    TaskCancel = 0x04,
    /// 清除多轮对话上下文（对应 ACPs clearContext）
    ClearContext = 0x05,
    /// 客户端向服务端发送授权信息（对应 ACPs authorize）
    Authorize = 0x06,
    /// 客户端取消当前授权（对应 ACPs deauthorize）
    Deauthorize = 0x07,
    /// 服务端向客户端推送通知（对应 ACPs push，适用于异步长耗时任务）
    Push = 0x08,
    /// [MCU扩展] 智能体发现请求，广播方式发送
    Discover = 0x10,
    /// [MCU扩展] 心跳保活，维持连接活跃状态
    Heartbeat = 0x20,
}

impl TryFrom<u8> for AcpMethod {
    type Error = AipError;

    fn try_from(value: u8) -> AipResult<Self> {
        match value {
            0x01 => Ok(AcpMethod::Initialize),
            0x02 => Ok(AcpMethod::Initialized),
            0x03 => Ok(AcpMethod::MessageStream),
            0x04 => Ok(AcpMethod::TaskCancel),
            0x05 => Ok(AcpMethod::ClearContext),
            0x06 => Ok(AcpMethod::Authorize),
            0x07 => Ok(AcpMethod::Deauthorize),
            0x08 => Ok(AcpMethod::Push),
            0x10 => Ok(AcpMethod::Discover),
            0x20 => Ok(AcpMethod::Heartbeat),
            _ => Err(AipError::UnsupportedMethod),
        }
    }
}

/// ACPs 协议响应状态码
///
/// 状态码采用单字节编码，0x00~0x0F 为成功类，0x40~0x4F 为客户端错误，0x50~0x5F 为服务端错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AcpStatusCode {
    /// 成功
    Ok = 0x00,
    /// 已接受，异步处理中
    Accepted = 0x01,
    /// 部分内容（流式传输中间帧）
    PartialContent = 0x02,
    /// 请求格式错误
    BadRequest = 0x40,
    /// 未授权
    Unauthorized = 0x41,
    /// 智能体/技能未找到
    NotFound = 0x44,
    /// 方法不支持
    MethodNotSupported = 0x45,
    /// 内部错误
    InternalError = 0x50,
    /// 服务不可用
    ServiceUnavailable = 0x53,
    /// 会话过期
    SessionExpired = 0x4E,
}

impl TryFrom<u8> for AcpStatusCode {
    type Error = AipError;

    fn try_from(value: u8) -> AipResult<Self> {
        match value {
            0x00 => Ok(AcpStatusCode::Ok),
            0x01 => Ok(AcpStatusCode::Accepted),
            0x02 => Ok(AcpStatusCode::PartialContent),
            0x40 => Ok(AcpStatusCode::BadRequest),
            0x41 => Ok(AcpStatusCode::Unauthorized),
            0x44 => Ok(AcpStatusCode::NotFound),
            0x45 => Ok(AcpStatusCode::MethodNotSupported),
            0x50 => Ok(AcpStatusCode::InternalError),
            0x53 => Ok(AcpStatusCode::ServiceUnavailable),
            0x4E => Ok(AcpStatusCode::SessionExpired),
            _ => Err(AipError::InvalidMessage),
        }
    }
}

/// ACPs-MCU 协议帧头
///
/// 帧头为固定 8 字节，包含协议标识、版本、方法码、标志位、负载长度和校验和。
/// 所有多字节字段采用大端序（网络字节序）编码。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AcpHeader {
    /// 协议标识，固定值 [`ACP_MAGIC`] (0xA1A2)
    pub magic: u16,
    /// 协议版本，当前为 [`ACP_VERSION`] (1)
    pub version: u8,
    /// 方法码，标识本帧的协议操作类型
    pub method: AcpMethod,
    /// 标志位，Bit0=FIN(最后一帧), Bit1=COMP(压缩), Bit2=ENC(加密), Bit3-7=保留
    pub flags: u8,
    /// 负载长度（含 Session-ID 4 字节），大端序，最大 65535
    pub payload_len: u16,
    /// 算术校验和，对帧所有字节（Checksum 字段填 0）求和 mod 256
    pub checksum: u8,
}

impl AcpHeader {
    /// 创建新的帧头，自动填充 Magic 和 Version
    ///
    /// # 参数
    /// - `method`: 协议方法码
    /// - `payload_len`: 负载长度（含 Session-ID 4 字节）
    pub fn new(method: AcpMethod, payload_len: u16) -> Self {
        Self {
            magic: ACP_MAGIC,
            version: ACP_VERSION,
            method,
            flags: 0,
            payload_len,
            checksum: 0,
        }
    }

    /// 将帧头序列化为 8 字节大端序数组
    pub fn to_bytes(&self) -> [u8; ACP_HEADER_LEN] {
        let mut buf = [0u8; ACP_HEADER_LEN];
        buf[0] = (self.magic >> 8) as u8;          // Magic 高字节
        buf[1] = (self.magic & 0xFF) as u8;         // Magic 低字节
        buf[2] = self.version;                       // 版本号
        buf[3] = self.method as u8;                  // 方法码
        buf[4] = self.flags;                         // 标志位
        buf[5] = (self.payload_len >> 8) as u8;     // 负载长度高字节
        buf[6] = (self.payload_len & 0xFF) as u8;   // 负载长度低字节
        buf[7] = self.checksum;                      // 校验和
        buf
    }

    /// 从 8 字节大端序数组反序列化帧头
    ///
    /// # 错误
    /// - [`AipError::InvalidHeader`] - Magic 不匹配
    /// - [`AipError::UnsupportedMethod`] - 方法码无效
    pub fn from_bytes(buf: &[u8; ACP_HEADER_LEN]) -> AipResult<Self> {
        let magic = ((buf[0] as u16) << 8) | (buf[1] as u16);
        if magic != ACP_MAGIC {
            return Err(AipError::InvalidHeader);
        }
        Ok(Self {
            magic,
            version: buf[2],
            method: AcpMethod::try_from(buf[3])?,
            flags: buf[4],
            payload_len: ((buf[5] as u16) << 8) | (buf[6] as u16),
            checksum: buf[7],
        })
    }

    /// 计算帧校验和
    ///
    /// 对帧头前 7 字节和完整负载求算术和（mod 256），
    /// 计算时校验和字段本身视为 0。
    pub fn compute_checksum(&self, payload: &[u8]) -> u8 {
        let bytes = self.to_bytes();
        let mut sum: u8 = 0;
        // 帧头前 7 字节参与计算（第 8 字节为 checksum 本身，填 0）
        for i in 0..7 {
            sum = sum.wrapping_add(bytes[i]);
        }
        // 负载所有字节参与计算
        for &b in payload {
            sum = sum.wrapping_add(b);
        }
        sum
    }
}

/// ACPs-MCU 协议元数据，提供协议版本号查询
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpProtocol;

impl AcpProtocol {
    /// 获取当前协议版本号
    pub fn version() -> u8 {
        ACP_VERSION
    }
}
