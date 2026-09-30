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
//! 帧总长度为 `ACP_HEADER_LEN + payload_len`，其中 `payload_len` **包含** 4 字节
//! Session-ID，因此合法取值区间为 `[4, 516]`（见 [`ACP_MIN_PAYLOAD_LEN`] /
//! [`ACP_MAX_PAYLOAD_LEN`]）。
//!
//! ## 方法码与 ACPs 规范的对应关系
//!
//! 方法码是本协议自有的（规范使用 JSON-RPC 方法名字符串），下表给出**真实**的
//! 对应关系，可在代码中通过 [`AcpMethod::acps_binding`] 查询：
//!
//! | MCU Code | 方法             | ACPs 对应                                       |
//! |----------|------------------|-------------------------------------------------|
//! | 0x01     | `Initialize`     | 无（MCU 扩展；规范没有握手流程）                 |
//! | 0x02     | `Initialized`    | 无（MCU 扩展）                                   |
//! | 0x03     | `Stream`         | JSON-RPC 方法 `stream`                          |
//! | 0x04     | `TaskCancel`     | TaskCommand `cancel`                            |
//! | 0x05     | `ClearContext`   | 无（MCU 扩展；规范没有该概念）                   |
//! | 0x06     | `Authorize`      | 无（规范用 `securitySchemes` + mTLS 表达）       |
//! | 0x07     | `Deauthorize`    | 无（MCU 扩展）                                   |
//! | 0x08     | `Push`           | `notification/*`（Partner→Leader HTTP POST）     |
//! | 0x10     | `Discover`       | ADP 发现路径 `/discover`                         |
//! | 0x11     | `CapabilitySpec` | ACS 文档对象 `AgentCapabilitySpec`               |
//! | 0x20     | `Heartbeat`      | 无（MCU 扩展）                                   |
//! | 0x21     | `HeartbeatAck`   | 无（MCU 扩展）                                   |

use crate::AipResult;
use crate::error::AipError;

/// 协议版本号，当前为 1
pub const ACP_VERSION: u8 = 1;

/// 协议帧 Magic 标识 `0xA1A2`，用于识别 ACPs-MCU 协议帧
pub const ACP_MAGIC: u16 = 0xA1A2;

/// 帧头固定长度（8 字节）
pub const ACP_HEADER_LEN: usize = 8;

/// Session-ID 字段长度（4 字节），永远包含在 `payload_len` 中
pub const ACP_SESSION_ID_LEN: usize = 4;

/// `payload_len` 最小值：仅含 Session-ID、业务负载为空
pub const ACP_MIN_PAYLOAD_LEN: u16 = ACP_SESSION_ID_LEN as u16;

/// 单帧业务负载最大长度
pub const ACP_MAX_PAYLOAD_LEN: usize = 512;

/// `payload_len` 最大值：Session-ID + 最大业务负载
pub const ACP_MAX_PAYLOAD_LEN_WIRE: u16 = ACP_MAX_PAYLOAD_LEN as u16 + ACP_SESSION_ID_LEN as u16;

/// 单帧最大总长度（帧头 + Session-ID + 最大业务负载）
pub const ACP_MAX_FRAME_LEN: usize = ACP_HEADER_LEN + ACP_MAX_PAYLOAD_LEN_WIRE as usize;

/// 帧标志位掩码定义
///
/// 对应帧头 `Flags` 字段，随帧头一起参与校验和计算。
pub mod flags {
    /// Bit0：1 = 最后一帧（FIN），0 = 后续还有分片
    pub const FIN: u8 = 0b0000_0001;
    /// Bit1：1 = payload 已压缩（预留，当前实现不做压缩）
    pub const COMP: u8 = 0b0000_0010;
    /// Bit2：1 = payload 已加密（预留，当前实现不做加密）
    pub const ENC: u8 = 0b0000_0100;
    /// Bit3~7：保留位，发送方必须置 0，接收方必须忽略
    pub const RSV: u8 = 0b1111_1000;
}

/// 方法码与 ACPs 规范的对应关系
///
/// 方法码本身是本协议自有的；本枚举把「对应什么」变成可查询、可测试的代码事实，
/// 避免文档与实现再次漂移。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AcpsBinding {
    /// 对应 ACPs 的 JSON-RPC 方法名
    Method(&'static str),
    /// 对应 ACPs 的 TaskCommand 取值
    TaskCommand(&'static str),
    /// 对应 ACPs 的文档对象或服务路径
    Document(&'static str),
    /// 纯 MCU 扩展：规范中没有对应概念
    McuExtension,
}

/// ACPs 协议方法码
///
/// 方法码采用单字节编码。注意：**方法码与规范并不是一一对应的**——
/// 规范使用 JSON-RPC 方法名字符串（`rpc` / `stream` / `group` / `notification/*`），
/// 这里既有对应规范方法/命令的码位，也有纯 MCU 扩展码位。
/// 逐条对应关系见 [`AcpMethod::acps_binding`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AcpMethod {
    /// 客户端发起会话初始化（MCU 扩展）
    Initialize = 0x01,
    /// 服务端确认初始化完成，返回 Session-ID（MCU 扩展）
    Initialized = 0x02,
    /// 消息流式传输，支持多帧分片（对应 ACPs 方法 `stream`）
    Stream = 0x03,
    /// 取消当前正在执行的任务（对应 ACPs TaskCommand `cancel`）
    TaskCancel = 0x04,
    /// 清除多轮对话上下文（MCU 扩展）
    ClearContext = 0x05,
    /// 客户端向服务端发送授权信息（MCU 扩展；规范用 securitySchemes/mTLS）
    Authorize = 0x06,
    /// 客户端取消当前授权（MCU 扩展）
    Deauthorize = 0x07,
    /// 服务端向客户端推送通知（对应 ACPs 的 `notification/*` 通知路径）
    Push = 0x08,
    /// \[MCU扩展\] 智能体发现请求，广播方式发送（对应 ADP `/discover`）
    Discover = 0x10,
    /// \[MCU扩展\] 返回 [`AgentCapabilitySpec`](crate::acs::AgentCapabilitySpec)，
    /// 是 `Discover` 的响应（对应 ACS 文档对象）
    CapabilitySpec = 0x11,
    /// \[MCU扩展\] 心跳保活，维持连接活跃状态
    Heartbeat = 0x20,
    /// \[MCU扩展\] 心跳确认，Heartbeat 的响应
    HeartbeatAck = 0x21,
}

impl AcpMethod {
    /// 返回方法的单字节编码，等价于 `self as u8`
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// 判断是否属于「与规范对应的码位」（0x01~0x0F）
    pub const fn is_standard(self) -> bool {
        self.code() <= 0x0F
    }

    /// 判断是否为 MCU 扩展码位（0x10 及以上）
    pub const fn is_mcu_extension(self) -> bool {
        !self.is_standard()
    }

    /// 该方法在 ACPs 规范中的对应物
    ///
    /// 返回 [`AcpsBinding::McuExtension`] 表示规范中没有对应概念，
    /// 属于本协议的自有扩展，跨网关时需要显式转换。
    pub const fn acps_binding(self) -> AcpsBinding {
        match self {
            AcpMethod::Stream => AcpsBinding::Method("stream"),
            AcpMethod::TaskCancel => AcpsBinding::TaskCommand("cancel"),
            AcpMethod::Push => AcpsBinding::Method("notification/set"),
            AcpMethod::Discover => AcpsBinding::Document("/discover"),
            AcpMethod::CapabilitySpec => AcpsBinding::Document("AgentCapabilitySpec"),
            AcpMethod::Initialize
            | AcpMethod::Initialized
            | AcpMethod::ClearContext
            | AcpMethod::Authorize
            | AcpMethod::Deauthorize
            | AcpMethod::Heartbeat
            | AcpMethod::HeartbeatAck => AcpsBinding::McuExtension,
        }
    }

    /// 返回该方法对应的响应方法码
    ///
    /// - `Initialize` → `Initialized`（会话初始化应答）
    /// - `Discover` → `CapabilitySpec`（发现应答携带 ACS）
    /// - `Heartbeat` → `HeartbeatAck`
    /// - 其余方法返回自身
    pub const fn response_method(self) -> Self {
        match self {
            AcpMethod::Initialize => AcpMethod::Initialized,
            AcpMethod::Discover => AcpMethod::CapabilitySpec,
            AcpMethod::Heartbeat => AcpMethod::HeartbeatAck,
            other => other,
        }
    }
}

impl TryFrom<u8> for AcpMethod {
    type Error = AipError;

    fn try_from(value: u8) -> AipResult<Self> {
        match value {
            0x01 => Ok(AcpMethod::Initialize),
            0x02 => Ok(AcpMethod::Initialized),
            0x03 => Ok(AcpMethod::Stream),
            0x04 => Ok(AcpMethod::TaskCancel),
            0x05 => Ok(AcpMethod::ClearContext),
            0x06 => Ok(AcpMethod::Authorize),
            0x07 => Ok(AcpMethod::Deauthorize),
            0x08 => Ok(AcpMethod::Push),
            0x10 => Ok(AcpMethod::Discover),
            0x11 => Ok(AcpMethod::CapabilitySpec),
            0x20 => Ok(AcpMethod::Heartbeat),
            0x21 => Ok(AcpMethod::HeartbeatAck),
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
    /// 会话过期
    SessionExpired = 0x4E,
    /// 内部错误
    InternalError = 0x50,
    /// 服务不可用
    ServiceUnavailable = 0x53,
}

impl AcpStatusCode {
    /// 返回状态码的单字节编码，等价于 `self as u8`
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// 判断是否为成功类状态码（0x00~0x0F）
    pub const fn is_success(self) -> bool {
        self.code() <= 0x0F
    }

    /// 判断是否为客户端错误（0x40~0x4F）
    pub const fn is_client_error(self) -> bool {
        matches!(self.code(), 0x40..=0x4F)
    }

    /// 判断是否为服务端错误（0x50~0x5F）
    pub const fn is_server_error(self) -> bool {
        matches!(self.code(), 0x50..=0x5F)
    }
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
            0x4E => Ok(AcpStatusCode::SessionExpired),
            0x50 => Ok(AcpStatusCode::InternalError),
            0x53 => Ok(AcpStatusCode::ServiceUnavailable),
            _ => Err(AipError::UnsupportedStatusCode),
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
    /// 负载长度（含 Session-ID 4 字节），大端序，合法区间 `[4, 516]`
    pub payload_len: u16,
    /// 算术校验和，对帧所有字节（Checksum 字段填 0）求和 mod 256
    pub checksum: u8,
}

impl AcpHeader {
    /// 创建新的帧头，自动填充 Magic 和 Version
    ///
    /// # 参数
    /// - `method`: 协议方法码
    /// - `payload_len`: 负载长度（含 Session-ID 4 字节），调用方需保证
    ///   `payload_len >= ACP_MIN_PAYLOAD_LEN`；否则 `to_bytes()` 产生的帧头
    ///   会被 [`AcpHeader::from_bytes`] 拒绝
    pub fn new(method: AcpMethod, payload_len: u16) -> Self {
        debug_assert!(
            payload_len >= ACP_MIN_PAYLOAD_LEN,
            "payload_len 必须至少包含 4 字节 Session-ID"
        );
        Self {
            magic: ACP_MAGIC,
            version: ACP_VERSION,
            method,
            flags: 0,
            payload_len,
            checksum: 0,
        }
    }

    /// 创建帧头并校验 `payload_len` 是否落在合法区间
    ///
    /// # 错误
    /// - [`AipError::InvalidPayload`] - `payload_len` 小于 4 或大于 516
    pub fn try_new(method: AcpMethod, payload_len: u16) -> AipResult<Self> {
        if !(ACP_MIN_PAYLOAD_LEN..=ACP_MAX_PAYLOAD_LEN_WIRE).contains(&payload_len) {
            return Err(AipError::InvalidPayload);
        }
        Ok(Self {
            magic: ACP_MAGIC,
            version: ACP_VERSION,
            method,
            flags: 0,
            payload_len,
            checksum: 0,
        })
    }

    /// 本帧总长度（帧头 + `payload_len`，后者已含 Session-ID）
    pub const fn frame_len(&self) -> usize {
        ACP_HEADER_LEN + self.payload_len as usize
    }

    /// 业务负载长度（即 `payload_len` 去掉 4 字节 Session-ID）
    ///
    /// 使用饱和减法，即使 `payload_len` 被构造成小于 4 也不会 panic。
    pub const fn body_len(&self) -> usize {
        (self.payload_len as usize).saturating_sub(ACP_SESSION_ID_LEN)
    }

    /// Bit0 FIN：本帧是否为分片传输的最后一帧
    pub const fn is_final(&self) -> bool {
        self.flags & flags::FIN != 0
    }

    /// 设置 Bit0 FIN 标志
    pub fn set_final(&mut self, final_frame: bool) {
        if final_frame {
            self.flags |= flags::FIN;
        } else {
            self.flags &= !flags::FIN;
        }
    }

    /// Bit1 COMP：payload 是否已压缩
    pub const fn is_compressed(&self) -> bool {
        self.flags & flags::COMP != 0
    }

    /// 设置 Bit1 COMP 标志
    pub fn set_compressed(&mut self, compressed: bool) {
        if compressed {
            self.flags |= flags::COMP;
        } else {
            self.flags &= !flags::COMP;
        }
    }

    /// Bit2 ENC：payload 是否已加密
    pub const fn is_encrypted(&self) -> bool {
        self.flags & flags::ENC != 0
    }

    /// 设置 Bit2 ENC 标志
    pub fn set_encrypted(&mut self, encrypted: bool) {
        if encrypted {
            self.flags |= flags::ENC;
        } else {
            self.flags &= !flags::ENC;
        }
    }

    /// 是否存在非法的保留位（Bit3~7 必须为 0）
    pub const fn has_reserved_bits(&self) -> bool {
        self.flags & flags::RSV != 0
    }

    /// 将帧头序列化为 8 字节大端序数组
    pub fn to_bytes(&self) -> [u8; ACP_HEADER_LEN] {
        let mut buf = [0u8; ACP_HEADER_LEN];
        buf[0] = (self.magic >> 8) as u8; // Magic 高字节
        buf[1] = (self.magic & 0xFF) as u8; // Magic 低字节
        buf[2] = self.version; // 版本号
        buf[3] = self.method as u8; // 方法码
        buf[4] = self.flags; // 标志位
        buf[5] = (self.payload_len >> 8) as u8; // 负载长度高字节
        buf[6] = (self.payload_len & 0xFF) as u8; // 负载长度低字节
        buf[7] = self.checksum; // 校验和
        buf
    }

    /// 从 8 字节大端序数组反序列化帧头
    ///
    /// # 错误
    /// - [`AipError::InvalidHeader`] - Magic 不匹配
    /// - [`AipError::UnsupportedVersion`] - 协议版本号不是 [`ACP_VERSION`]
    /// - [`AipError::UnsupportedMethod`] - 方法码无效
    /// - [`AipError::InvalidPayload`] - `payload_len` 不在 `[4, 516]` 区间内
    pub fn from_bytes(buf: &[u8; ACP_HEADER_LEN]) -> AipResult<Self> {
        let magic = ((buf[0] as u16) << 8) | (buf[1] as u16);
        if magic != ACP_MAGIC {
            return Err(AipError::InvalidHeader);
        }
        if buf[2] != ACP_VERSION {
            return Err(AipError::UnsupportedVersion);
        }
        let payload_len = ((buf[5] as u16) << 8) | (buf[6] as u16);
        if !(ACP_MIN_PAYLOAD_LEN..=ACP_MAX_PAYLOAD_LEN_WIRE).contains(&payload_len) {
            return Err(AipError::InvalidPayload);
        }
        Ok(Self {
            magic,
            version: buf[2],
            method: AcpMethod::try_from(buf[3])?,
            flags: buf[4],
            payload_len,
            checksum: buf[7],
        })
    }

    /// 计算帧校验和
    ///
    /// 对帧头前 7 字节和完整负载（Session-ID + 业务负载）求算术和（mod 256），
    /// 计算时校验和字段本身视为 0。
    pub fn compute_checksum(&self, payload: &[u8]) -> u8 {
        let bytes = self.to_bytes();
        // 帧头前 7 字节参与计算（第 8 字节为 checksum 本身，填 0）
        let mut sum = bytes[..7].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        // 负载所有字节参与计算
        for &b in payload {
            sum = sum.wrapping_add(b);
        }
        sum
    }
}

/// ACPs-MCU 协议元数据，提供协议版本号查询
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AcpProtocol;

impl AcpProtocol {
    /// 获取当前协议版本号
    pub const fn version() -> u8 {
        ACP_VERSION
    }

    /// 判断给定版本号是否与本实现兼容（当前仅支持精确匹配）
    pub const fn is_compatible(version: u8) -> bool {
        version == ACP_VERSION
    }
}
