//! # 智能体卡片定义
//!
//! [`AgentCard`] 是 ACPs 协议中描述智能体身份和能力的核心数据结构，
//! 对应 ACPs 标准中的 AgentCard 概念。
//!
//! 智能体卡片包含：
//! - **agent_id**: 智能体唯一标识
//! - **name**: 显示名称
//! - **description**: 功能描述
//! - **version**: 智能体版本
//! - **capabilities**: 能力列表（传感器读取、执行器控制等）
//! - **skills**: 技能列表
//! - **endpoint**: 通信端点地址（如 `aip://esp32-sensor-01.local`）
//!
//! ## 使用示例
//!
//! ```rust
//! use aipmc::basic::{AgentCard, AgentCapability, AgentSkill};
//!
//! let agent = AgentCard::new("esp32-01", "TempSensor", "Temperature sensor agent")
//!     .unwrap()
//!     .with_capability(AgentCapability::SensorRead)
//!     .unwrap()
//!     .with_skill(
//!         AgentSkill::new("temp-read", "ReadTemp", "Read temperature").unwrap()
//!     )
//!     .unwrap()
//!     .with_endpoint("aip://esp32-01.local")
//!     .unwrap();
//! ```

#[cfg(feature = "alloc")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::error::{AipError, AipResult};
use crate::skill::AgentSkill;

/// 将 `&str` 转换为 `heapless::String<N>`，失败时返回 BufferOverflow
fn try_str<const N: usize>(s: &str) -> AipResult<heapless::String<N>> {
    heapless::String::try_from(s).map_err(|_| AipError::BufferOverflow)
}

/// 智能体能力类型
///
/// 描述智能体具备的硬件/软件能力，用于智能体发现时的匹配过滤。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AgentCapability {
    /// 传感器数据读取能力
    SensorRead,
    /// 执行器控制能力
    ActuatorControl,
    /// 通信能力（WiFi/BLE/LoRa 等）
    Communication,
    /// 计算能力（ML 推理、信号处理等）
    Compute,
    /// 存储能力
    Storage,
    /// 显示能力
    Display,
    /// 自定义能力类型
    Custom(heapless::String<32>),
}

/// 智能体卡片
///
/// 用于描述智能体的身份、能力和技能，对应 ACPs 标准中的 AgentCard。
/// 在智能体发现（Discover）过程中，服务端通过此卡片向客户端描述自身能力。
///
/// 字段容量限制适配 MCU 场景：
/// - agent_id: 64 字节
/// - name: 64 字节
/// - description: 256 字节
/// - version: 16 字节
/// - capabilities: 最多 8 个
/// - skills: 最多 8 个
/// - endpoint: 128 字节
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AgentCard {
    /// 智能体唯一标识，如 "esp32-sensor-01"
    pub agent_id: heapless::String<64>,
    /// 智能体显示名称，如 "ESP32 Temperature Sensor"
    pub name: heapless::String<64>,
    /// 智能体功能描述
    pub description: heapless::String<256>,
    /// 智能体版本号，如 "0.1.0"
    pub version: heapless::String<16>,
    /// 智能体能力列表（最多 8 个）
    pub capabilities: heapless::Vec<AgentCapability, 8>,
    /// 智能体技能列表（最多 8 个）
    pub skills: heapless::Vec<AgentSkill, 8>,
    /// 通信端点地址，如 "aip://esp32-sensor-01.local"
    pub endpoint: heapless::String<128>,
}

impl AgentCard {
    /// 创建新的智能体卡片
    ///
    /// 默认版本号为 "0.1.0"，能力列表和技能列表为空。
    ///
    /// # 参数
    /// - `agent_id`: 智能体唯一标识
    /// - `name`: 显示名称
    /// - `description`: 功能描述
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 字符串超出容量限制
    pub fn new(agent_id: &str, name: &str, description: &str) -> AipResult<Self> {
        Ok(Self {
            agent_id: try_str(agent_id)?,
            name: try_str(name)?,
            description: try_str(description)?,
            version: try_str("0.1.0")?,
            capabilities: heapless::Vec::new(),
            skills: heapless::Vec::new(),
            endpoint: heapless::String::new(),
        })
    }

    /// 添加能力描述
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 能力数超过 8 个
    pub fn with_capability(mut self, cap: AgentCapability) -> AipResult<Self> {
        self.capabilities
            .push(cap)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(self)
    }

    /// 添加技能描述
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 技能数超过 8 个
    pub fn with_skill(mut self, skill: AgentSkill) -> AipResult<Self> {
        self.skills
            .push(skill)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(self)
    }

    /// 设置通信端点地址
    ///
    /// # 参数
    /// - `endpoint`: 端点地址，格式为 `aip://<agent-id>.<transport>`
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 端点字符串超过 128 字节
    pub fn with_endpoint(mut self, endpoint: &str) -> AipResult<Self> {
        self.endpoint = try_str(endpoint)?;
        Ok(self)
    }

    /// 设置版本号
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 版本字符串超过 16 字节
    pub fn with_version(mut self, version: &str) -> AipResult<Self> {
        self.version = try_str(version)?;
        Ok(self)
    }
}

#[cfg(feature = "alloc")]
impl AgentCard {
    /// 序列化为 JSON 字符串（需要 alloc feature）
    pub fn to_json(&self) -> AipResult<String> {
        serde_json::to_string(self).map_err(|_| AipError::CodecError)
    }

    /// 从 JSON 字符串反序列化（需要 alloc feature）
    pub fn from_json(json: &str) -> AipResult<Self> {
        serde_json::from_str(json).map_err(|_| AipError::CodecError)
    }
}
