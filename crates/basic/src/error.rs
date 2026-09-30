//! # 错误类型定义
//!
//! 定义 AIP 协议栈中所有模块共用的错误枚举 [`AipError`] 和结果类型 [`AipResult`]。
//!
//! 错误分类：
//! - **协议层**：InvalidMessage / InvalidHeader / InvalidPayload / ChecksumMismatch
//! - **编解码层**：CodecError / BufferOverflow / UnsupportedVersion / UnsupportedMethod /
//!   UnsupportedStatusCode
//! - **传输层**：TransportError / UnexpectedEof / Timeout
//! - **会话层**：AgentNotFound / SkillNotFound / AuthenticationFailed / SessionExpired
//! - **未实现**：NotImplemented
//! - **内部错误**：InternalError

/// AIP 协议栈统一错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AipError {
    /// 缓冲区溢出，数据超出 heapless 容器容量或协议上限
    BufferOverflow,
    /// 编解码错误，序列化/反序列化失败
    CodecError,
    /// 无效消息，帧格式不符合协议规范
    InvalidMessage,
    /// 无效帧头，Magic 不匹配或字段越界
    InvalidHeader,
    /// 无效负载，负载长度与帧头声明不一致或超出合法区间
    InvalidPayload,
    /// 校验和不匹配，数据可能在传输中损坏
    ChecksumMismatch,
    /// 传输层错误，UART/SPI/BLE 读写失败
    TransportError,
    /// 读取过程中提前到达流末尾，帧不完整
    UnexpectedEof,
    /// 智能体未找到，目标 AIC 不存在
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
    /// 不支持的状态码
    UnsupportedStatusCode,
    /// 无效的智能体身份码（AIC）：段数、字符集、长度或前缀不符合规范
    InvalidAic,
    /// 无效的智能体能力描述（ACS）：必填字段不满足规范约束
    InvalidAcs,
    /// 功能尚未实现（当前构建未启用对应 feature 或平台适配未完成）
    NotImplemented,
    /// 内部错误，不可归类的运行时异常
    InternalError,
}

impl core::fmt::Display for AipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            AipError::BufferOverflow => "buffer overflow",
            AipError::CodecError => "codec error",
            AipError::InvalidMessage => "invalid message",
            AipError::InvalidHeader => "invalid header",
            AipError::InvalidPayload => "invalid payload",
            AipError::ChecksumMismatch => "checksum mismatch",
            AipError::TransportError => "transport error",
            AipError::UnexpectedEof => "unexpected end of stream",
            AipError::AgentNotFound => "agent not found",
            AipError::SkillNotFound => "skill not found",
            AipError::AuthenticationFailed => "authentication failed",
            AipError::SessionExpired => "session expired",
            AipError::Timeout => "timeout",
            AipError::UnsupportedVersion => "unsupported version",
            AipError::UnsupportedMethod => "unsupported method",
            AipError::UnsupportedStatusCode => "unsupported status code",
            AipError::InvalidAic => "invalid agent identity code (AIC)",
            AipError::InvalidAcs => "invalid agent capability spec (ACS)",
            AipError::NotImplemented => "not implemented",
            AipError::InternalError => "internal error",
        };
        f.write_str(msg)
    }
}

impl core::error::Error for AipError {}

/// AIP 协议栈统一结果类型
pub type AipResult<T> = Result<T, AipError>;
