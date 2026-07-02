//! # 智能体技能定义
//!
//! [`AgentSkill`] 描述智能体可以执行的具体功能，对应 ACPs 协议中的技能描述。
//!
//! 每个技能包含：
//! - **id**: 技能唯一标识（如 "temp-read"）
//! - **name**: 技能显示名称（如 "ReadTemperature"）
//! - **description**: 技能功能描述
//! - **tags**: 技能标签列表，用于分类和发现
//! - **input_schema**: 输入参数 JSON Schema
//! - **output_schema**: 输出结果 JSON Schema
//!
//! ## 使用示例
//!
//! ```rust
//! use aipmc::basic::AgentSkill;
//!
//! let skill = AgentSkill::new("temp-read", "ReadTemperature", "Read temperature from DHT22")
//!     .unwrap()
//!     .with_tag("sensor")
//!     .unwrap()
//!     .with_schemas(
//!         r#"{"type":"object","properties":{"unit":{"type":"string"}}}"#,
//!         r#"{"type":"object","properties":{"value":{"type":"number"}}}"#,
//!     )
//!     .unwrap();
//! ```

#[cfg(feature = "alloc")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::error::{AipError, AipResult};

/// 将 `&str` 转换为 `heapless::String<N>`，失败时返回 BufferOverflow
fn try_str<const N: usize>(s: &str) -> AipResult<heapless::String<N>> {
    heapless::String::try_from(s).map_err(|_| AipError::BufferOverflow)
}

/// 智能体技能
///
/// 描述智能体可以执行的具体功能。每个技能定义了用途、标签、
/// 输入输出模式，支持多技能组合。
///
/// 字段容量限制适配 MCU 场景：
/// - id: 32 字节
/// - name: 64 字节
/// - description: 256 字节
/// - tags: 最多 4 个，每个 16 字节
/// - schema: 128 字节
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AgentSkill {
    /// 技能唯一标识，如 "temp-read"、"relay-ctrl"
    pub id: heapless::String<32>,
    /// 技能显示名称，如 "ReadTemperature"
    pub name: heapless::String<64>,
    /// 技能功能描述
    pub description: heapless::String<256>,
    /// 技能标签列表，用于分类发现（最多 4 个）
    pub tags: heapless::Vec<heapless::String<16>, 4>,
    /// 输入参数 JSON Schema
    pub input_schema: heapless::String<128>,
    /// 输出结果 JSON Schema
    pub output_schema: heapless::String<128>,
}

impl AgentSkill {
    /// 创建新的技能描述
    ///
    /// # 参数
    /// - `id`: 技能唯一标识
    /// - `name`: 技能显示名称
    /// - `description`: 功能描述
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 字符串超出容量限制
    pub fn new(id: &str, name: &str, description: &str) -> AipResult<Self> {
        Ok(Self {
            id: try_str(id)?,
            name: try_str(name)?,
            description: try_str(description)?,
            tags: heapless::Vec::new(),
            input_schema: heapless::String::new(),
            output_schema: heapless::String::new(),
        })
    }

    /// 添加标签
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 标签数超过 4 个或字符串超长
    pub fn with_tag(mut self, tag: &str) -> AipResult<Self> {
        self.tags
            .push(try_str(tag)?)
            .map_err(|_| AipError::BufferOverflow)?;
        Ok(self)
    }

    /// 设置输入/输出 JSON Schema
    ///
    /// # 参数
    /// - `input`: 输入参数的 JSON Schema 字符串
    /// - `output`: 输出结果的 JSON Schema 字符串
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - Schema 字符串超过 128 字节
    pub fn with_schemas(mut self, input: &str, output: &str) -> AipResult<Self> {
        self.input_schema = try_str(input)?;
        self.output_schema = try_str(output)?;
        Ok(self)
    }
}

#[cfg(feature = "alloc")]
impl AgentSkill {
    /// 序列化为 JSON 字符串（需要 alloc feature）
    pub fn to_json(&self) -> AipResult<String> {
        serde_json::to_string(self).map_err(|_| AipError::CodecError)
    }

    /// 从 JSON 字符串反序列化（需要 alloc feature）
    pub fn from_json(json: &str) -> AipResult<Self> {
        serde_json::from_str(json).map_err(|_| AipError::CodecError)
    }
}
