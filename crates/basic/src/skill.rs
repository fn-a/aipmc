//! # ACS 技能描述
//!
//! [`AgentSkill`] 对齐 ACPs 规范 `ACPs-spec-ACS-v02.01` 的 `AgentSkill`：
//! 必填字段为 `id` / `name` / `description` / `version` / `tags`，可选字段为
//! `examples` / `inputModes` / `outputModes`。
//!
//! 规范**没有** per-skill 的 JSON Schema 字段（结构化负载只在运行时通过
//! `DataItem.data` + `metadata.schema` 表达）。本实现为了 MCU 侧的技能分发与参数
//! 自检，保留了输入/输出模式，但把它们放进 MCU 扩展命名空间 `x-mcu`：
//!
//! ```json
//! {
//!   "id": "temp.read",
//!   "name": "ReadTemperature",
//!   "description": "Read temperature",
//!   "version": "0.1.0",
//!   "tags": ["sensor"],
//!   "x-mcu": { "inputSchema": "{...}", "outputSchema": "{...}" }
//! }
//! ```
//!
//! 扩展为空时不会出现在 JSON 中，因此只使用 ACS 字段时输出是规范的子集。
//!
//! ## 使用示例
//!
//! ```rust
//! use basic::skill::AgentSkill;
//!
//! let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
//! skill.add_tag("sensor").unwrap();
//! skill
//!     .set_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
//!     .unwrap();
//! assert!(skill.has_tag("sensor"));
//! ```

use serde::{Deserialize, Serialize};

use crate::error::{AipError, AipResult};
use crate::util::try_str;

/// `skip_serializing_if` 用的辅助函数（`heapless::Vec` 判空）
fn is_empty<T, const N: usize>(v: &heapless::Vec<T, N>) -> bool {
    v.is_empty()
}

/// `skip_serializing_if` 用的辅助函数（`heapless::String` 判空）
fn is_empty_str<const N: usize>(s: &heapless::String<N>) -> bool {
    s.is_empty()
}

/// MIME 类型（ACS 的 `defaultInputModes` / `inputModes` 等字段的取值）
///
/// 只做「包含 `/`」的最简校验：MCU 上不引入完整的 MIME 解析表。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(transparent)]
pub struct MimeMode(heapless::String<48>);

impl MimeMode {
    /// 常量长度必然不超过 48 字节，因此不会失败
    fn from_static(value: &'static str) -> Self {
        let mut out: heapless::String<48> = heapless::String::new();
        // 传入的均为编译期常量，长度小于容量
        let _ = out.push_str(value);
        Self(out)
    }

    /// `application/json`
    pub fn json() -> Self {
        Self::from_static("application/json")
    }

    /// `text/plain`
    pub fn text() -> Self {
        Self::from_static("text/plain")
    }

    /// `application/octet-stream`
    pub fn octet_stream() -> Self {
        Self::from_static("application/octet-stream")
    }

    /// 解析 MIME 类型
    ///
    /// # 错误
    /// - [`AipError::InvalidAcs`] - 不含 `/`、以 `/` 开头或结尾
    /// - [`AipError::BufferOverflow`] - 超过 48 字节
    pub fn new(value: &str) -> AipResult<Self> {
        let (ty, sub) = value.split_once('/').ok_or(AipError::InvalidAcs)?;
        if ty.is_empty() || sub.is_empty() {
            return Err(AipError::InvalidAcs);
        }
        Ok(Self(try_str(value)?))
    }

    /// 原始字符串
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// MCU 扩展：技能级 `x-mcu`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct McuSkillExtensions {
    /// 输入参数模式（ACS 无对应字段）
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub input_schema: heapless::String<128>,
    /// 输出结果模式（ACS 无对应字段）
    #[serde(default, skip_serializing_if = "is_empty_str")]
    pub output_schema: heapless::String<128>,
}

impl McuSkillExtensions {
    /// 是否为空（为空时不会出现在 JSON 中）
    pub fn is_empty(&self) -> bool {
        self.input_schema.is_empty() && self.output_schema.is_empty()
    }
}

/// 智能体技能（ACS `AgentSkill`）
///
/// 容量限制（适配 MCU）：
/// - `id` 32B，`name` 64B，`description` 256B，`version` 16B
/// - `tags` 最多 4 个（每个 16B），`examples` 最多 2 个（每个 64B）
/// - `inputModes` / `outputModes` 各最多 2 个
/// - `x-mcu.inputSchema` / `x-mcu.outputSchema` 各 128B
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    /// 技能唯一标识，建议点号命名空间，如 `"temp.read"`
    pub id: heapless::String<32>,
    /// 技能显示名称，如 `"ReadTemperature"`
    pub name: heapless::String<64>,
    /// 技能功能描述
    pub description: heapless::String<256>,
    /// 技能版本
    pub version: heapless::String<16>,
    /// 技能标签，用于发现与过滤
    pub tags: heapless::Vec<heapless::String<16>, 4>,
    /// 调用示例（可选）
    #[serde(default, skip_serializing_if = "is_empty")]
    pub examples: heapless::Vec<heapless::String<64>, 2>,
    /// 输入 MIME 类型（覆盖智能体默认值，可选）
    #[serde(default, skip_serializing_if = "is_empty")]
    pub input_modes: heapless::Vec<MimeMode, 2>,
    /// 输出 MIME 类型（覆盖智能体默认值，可选）
    #[serde(default, skip_serializing_if = "is_empty")]
    pub output_modes: heapless::Vec<MimeMode, 2>,
    /// [MCU 扩展] 技能级扩展信息
    #[serde(
        rename = "x-mcu",
        default,
        skip_serializing_if = "McuSkillExtensions::is_empty"
    )]
    pub mcu: McuSkillExtensions,
}

impl AgentSkill {
    /// 创建技能描述
    ///
    /// 默认 `version = "0.1.0"`，标签/示例/MIME/扩展均为空。
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 字符串超出容量限制
    pub fn new(id: &str, name: &str, description: &str) -> AipResult<Self> {
        Ok(Self {
            id: try_str(id)?,
            name: try_str(name)?,
            description: try_str(description)?,
            version: try_str("0.1.0")?,
            tags: heapless::Vec::new(),
            examples: heapless::Vec::new(),
            input_modes: heapless::Vec::new(),
            output_modes: heapless::Vec::new(),
            mcu: McuSkillExtensions::default(),
        })
    }

    /// 设置技能版本
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 16 字节
    pub fn set_version(&mut self, version: &str) -> AipResult<()> {
        self.version = try_str(version)?;
        Ok(())
    }

    /// 添加标签
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 标签超过 4 个或字符串超长
    pub fn add_tag(&mut self, tag: &str) -> AipResult<()> {
        self.tags
            .push(try_str(tag)?)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 添加调用示例
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 示例超过 2 个或字符串超长
    pub fn add_example(&mut self, example: &str) -> AipResult<()> {
        self.examples
            .push(try_str(example)?)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 设置输入/输出 MIME 类型
    ///
    /// 会先清空现有列表。
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 2 个
    pub fn set_modes(&mut self, input: MimeMode, output: MimeMode) -> AipResult<()> {
        self.input_modes.clear();
        self.output_modes.clear();
        self.input_modes
            .push(input)
            .map_err(|_| AipError::BufferOverflow)?;
        self.output_modes
            .push(output)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 设置输入/输出参数模式（MCU 扩展）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 模式字符串超过 128 字节
    pub fn set_schemas(&mut self, input: &str, output: &str) -> AipResult<()> {
        self.mcu.input_schema = try_str(input)?;
        self.mcu.output_schema = try_str(output)?;
        Ok(())
    }

    /// 是否具备指定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

#[cfg(feature = "alloc")]
impl AgentSkill {
    /// 序列化为 JSON 字符串（需要 `alloc` feature）
    pub fn to_json(&self) -> AipResult<alloc::string::String> {
        serde_json::to_string(self).map_err(|_| AipError::CodecError)
    }

    /// 从 JSON 字符串反序列化（需要 `alloc` feature）
    pub fn from_json(json: &str) -> AipResult<Self> {
        serde_json::from_str(json).map_err(|_| AipError::CodecError)
    }
}
