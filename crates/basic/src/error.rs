//! # 错误类型定义
//!
//! 定义 AIP 协议栈中所有模块共用的错误枚举 [`AipError`] 和结果类型 [`AipResult`]。
//!
//! 错误分类：
//! - **协议层**：InvalidMessage / InvalidHeader / InvalidPayload / ChecksumMismatch
//! - **编解码层**：CodecError / BufferOverflow / UnsupportedVersion / UnsupportedMethod
//! - **传输层**：TransportError / Timeout
//! - **会话层**：AgentNotFound / SkillNotFound / AuthenticationFailed / SessionExpired
//! - **内部错误**：InternalError

/// AIP 协议栈统一错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AipError {
    /// 缓冲区溢出，数据超出 heapless 容器容量
    BufferOverflow,
    /// 编解码错误，序列化/反序列化失败
    CodecError,
    /// 无效消息，帧格式不符合协议规范
    InvalidMessage,
    /// 无效帧头，Magic 不匹配或字段越界
    InvalidHeader,
    /// 无效负载，负载长度与帧头声明不一致
    InvalidPayload,
    /// 校验和不匹配，数据可能在传输中损坏
    ChecksumMismatch,
    /// 传输层错误，UART/SPI/BLE 读写失败
    TransportError,
    /// 智能体未找到，目标 agent_id 不存在
    AgentNotFound,
    /// 技能未找到，目标 skill_id 不存在
    SkillNotFound,
    /// 认证失败，密钥或令牌无效
    AuthenticationFailed,
    /// 会话已过期，需要重新初始化
    SessionExpired,
    /// 操作超时，对端未在规定时间内响应
    Timeout,
    /// 不支持的协议版本
    UnsupportedVersion,
    /// 不支持的方法码
    UnsupportedMethod,
    /// 内部错误，不可归类的运行时异常
    InternalError,
}

impl core::fmt::Display for AipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AipError::BufferOverflow => write!(f, "buffer overflow"),
            AipError::CodecError => write!(f, "codec error"),
            AipError::InvalidMessage => write!(f, "invalid message"),
            AipError::InvalidHeader => write!(f, "invalid header"),
            AipError::InvalidPayload => write!(f, "invalid payload"),
            AipError::ChecksumMismatch => write!(f, "checksum mismatch"),
            AipError::TransportError => write!(f, "transport error"),
            AipError::AgentNotFound => write!(f, "agent not found"),
            AipError::SkillNotFound => write!(f, "skill not found"),
            AipError::AuthenticationFailed => write!(f, "authentication failed"),
            AipError::SessionExpired => write!(f, "session expired"),
            AipError::Timeout => write!(f, "timeout"),
            AipError::UnsupportedVersion => write!(f, "unsupported version"),
            AipError::UnsupportedMethod => write!(f, "unsupported method"),
            AipError::InternalError => write!(f, "internal error"),
        }
    }
}

/// AIP 协议栈统一结果类型
pub type AipResult<T> = Result<T, AipError>;
