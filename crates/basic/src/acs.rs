//! # ACS — Agent Capability Spec（智能体能力描述）
//!
//! 对齐 ACPs 规范 `ACPs-spec-ACS-v02.01`。ACS 的 **MCU 子集**：
//! 字段名、JSON 大小写与枚举取值都按规范命名，
//! MCU 特有的信息统一放在 `x-mcu` 命名空间下。
//!
//! ## 与规范的对应关系
//!
//! | ACS 根字段 | 本实现 | 说明 |
//! |-----------|--------|------|
//! | `aic` | ✅ [`Aic`] | 由自由字符串 `agent_id` 改为规范身份码 |
//! | `active` | ✅ | |
//! | `lastModifiedTime` | ✅ | ISO-8601 字符串，由调用方写入 |
//! | `protocolVersion` | ✅ | 默认 [`ACPS_PROTOCOL_VERSION`] |
//! | `name` / `description` / `version` | ✅ | |
//! | `provider` | ✅ [`AgentProvider`] | 全部字段可选 |
//! | `endPoints` | ✅ [`AgentEndPoint`] | `transport` 为规范枚举 + MCU 扩展取值 |
//! | `capabilities` | ✅ [`AgentCapabilities`] | `streaming` / `notification` / `messageQueue` |
//! | `defaultInputModes` / `defaultOutputModes` | ✅ [`MimeMode`] | MIME 列表 |
//! | `skills` | ✅ [`AgentSkill`] | |
//! | `securitySchemes` | ⛔ 未实现 | 见下文「MCU 子集省略项」 |
//! | `iconUrl` / `documentationUrl` / `webAppUrl` / `entityUserId` / `entityMeta` / `certificate` | ⛔ 未实现 | 可选字段，MCU 场景无意义 |
//!
//! ### MCU 子集省略项
//!
//! `securitySchemes` 在规范中是必填字段，但 MCU 设备无法独立承担 TLS/mTLS 与
//! OAuth2 流程。因此本实现不输出该字段：设备侧输出的 JSON 是 ACS 的**核心字段子集**，
//! 注册到 ACPs 网络时应由网关（Leader/Registry）补齐 `securitySchemes` 等字段。
//!
//! ### MCU 扩展
//!
//! - 根级 `x-mcu.capabilities`：硬件能力枚举（传感器/执行器/…），ACS 无对应概念
//! - 技能级 `x-mcu.inputSchema` / `x-mcu.outputSchema`：技能参数模式，ACS 规定
//!   结构化负载只在运行时通过 `DataItem.data` + `metadata.schema` 表达
//! - `AgentEndPoint.transport` 的 `MCU_*` 取值：ACS 只定义了 `JSONRPC` /
//!   `HTTP_JSON` / `AMQP` 三种
//!
//! 扩展字段在为空时不会出现在 JSON 中；严格遵循 ACS 的消费方应忽略或剥离 `x-mcu`。
//!
//! ## 使用示例
//!
//! ```rust
//! use basic::acs::{AgentCapabilitySpec, AgentEndPoint, AgentSkill, Transport};
//! use basic::aic::Aic;
//!
//! let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
//! let mut spec = AgentCapabilitySpec::new(aic, "ESP32-Sensor", "Temperature sensor agent")
//!     .unwrap();
//!
//! // 注意：设置接口一律是 `&mut self`，见下方「内存占用」说明
//! let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
//! skill.add_tag("sensor").unwrap();
//! spec.add_skill(skill).unwrap();
//! spec.add_end_point(
//!     AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
//! )
//! .unwrap();
//!
//! assert!(spec.find_skill("temp.read").is_some());
//! ```

use serde::{Deserialize, Serialize};

use crate::aic::Aic;
use crate::error::{AipError, AipResult};
use crate::util::try_str;

// 技能与 MIME 类型属于 ACS 的组成部分，这里一并重导出，
// 便于 `use basic::acs::{AgentCapabilitySpec, AgentSkill};` 一次拿到常用类型。
pub use crate::skill::{AgentSkill, McuSkillExtensions, MimeMode};

/// 本实现对应的 ACPs 规范版本
pub const ACPS_PROTOCOL_VERSION: &str = "02.01";

/// 默认的历史修改时间占位（规范要求 ISO-8601，MCU 上通常由上位机写入）
pub const DEFAULT_LAST_MODIFIED_TIME: &str = "1970-01-01T00:00:00+08:00";

/// 智能体硬件能力
///
/// **MCU 扩展**：ACS 用 `capabilities{streaming,notification,messageQueue}` 描述
/// 协议能力，用 `skills[].tags` 描述功能；这里的硬件能力枚举是本实现的补充，
/// 序列化到 `x-mcu.capabilities`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub enum AgentCapability {
    /// 传感器数据读取
    SensorRead,
    /// 执行器控制
    ActuatorControl,
    /// 通信能力（WiFi/BLE/LoRa 等）
    Communication,
    /// 计算能力（ML 推理、信号处理等）
    Compute,
    /// 存储能力
    Storage,
    /// 显示能力
    Display,
    /// 自定义能力
    Custom(heapless::String<32>),
}

/// ACS `capabilities`：协议能力开关
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    /// 是否支持流式输出
    pub streaming: bool,
    /// 是否支持异步通知（`notification/*`）
    pub notification: bool,
    /// 支持的消息队列协议版本，形如 `"amqp:4.2"`
    pub message_queue: heapless::Vec<MqProtocolVersion, 4>,
}

impl AgentCapabilities {
    /// 最小能力集：不支持流式与通知，无消息队列
    pub fn none() -> Self {
        Self::default()
    }

    /// 设置是否支持流式输出
    pub fn set_streaming(&mut self, streaming: bool) {
        self.streaming = streaming;
    }

    /// 设置是否支持异步通知
    pub fn set_notification(&mut self, notification: bool) {
        self.notification = notification;
    }

    /// 追加消息队列协议版本
    ///
    /// # 错误
    /// - [`AipError::InvalidAcs`] - 格式不是 `{protocol}:{versionExpr}` 或协议不受支持
    /// - [`AipError::BufferOverflow`] - 条目超过 4 个
    pub fn add_message_queue(&mut self, mq: MqProtocolVersion) -> AipResult<()> {
        self.message_queue
            .push(mq)
            .map_err(|_| AipError::BufferOverflow)
    }
}

/// 消息队列协议版本，形如 `"amqp:4.2"`、`"mqtt:5"`、`"kafka:*"`
///
/// 规范允许的协议词表：`mqtt` / `amqp` / `kafka` / `redis` / `rabbitmq`。
/// 反序列化时同样会校验，非法取值返回 `CodecError`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(transparent)]
pub struct MqProtocolVersion(heapless::String<24>);

impl MqProtocolVersion {
    /// 规范允许的协议词表
    pub const PROTOCOLS: [&'static str; 5] = ["mqtt", "amqp", "kafka", "redis", "rabbitmq"];

    /// 解析并校验 `{protocol}:{versionExpr}`
    ///
    /// # 错误
    /// - [`AipError::InvalidAcs`] - 缺少分隔符、协议不在词表中或版本表达式为空
    /// - [`AipError::BufferOverflow`] - 超过 24 字节
    pub fn new(value: &str) -> AipResult<Self> {
        let (protocol, version) = value.split_once(':').ok_or(AipError::InvalidAcs)?;
        let known = Self::PROTOCOLS
            .iter()
            .any(|p| p.eq_ignore_ascii_case(protocol));
        if !known || version.is_empty() {
            return Err(AipError::InvalidAcs);
        }
        Ok(Self(try_str(value)?))
    }

    /// 协议名（冒号前）
    pub fn protocol(&self) -> &str {
        self.0.split_once(':').map(|(p, _)| p).unwrap_or("")
    }

    /// 版本表达式（冒号后）
    pub fn version_expression(&self) -> &str {
        self.0.split_once(':').map(|(_, v)| v).unwrap_or("")
    }

    /// 原始字符串
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<'de> Deserialize<'de> for MqProtocolVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MqVisitor;

        impl<'a> serde::de::Visitor<'a> for MqVisitor {
            type Value = MqProtocolVersion;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a message-queue protocol version such as \"amqp:4.2\"")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                MqProtocolVersion::new(value).map_err(|_| E::custom("invalid messageQueue entry"))
            }
        }

        deserializer.deserialize_str(MqVisitor)
    }
}

/// ACS `provider`：智能体提供方信息（全部字段可选）
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct AgentProvider {
    /// 国家/地区代码，规范默认 `"CN"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<heapless::String<4>>,
    /// 组织名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<heapless::String<64>>,
    /// 部门
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<heapless::String<64>>,
    /// 组织网址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<heapless::String<128>>,
    /// 许可证标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<heapless::String<32>>,
    /// 提供方名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<heapless::String<64>>,
    /// 联系邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<heapless::String<64>>,
}

impl AgentProvider {
    /// 创建带国家代码的默认提供方（`countryCode = "CN"`）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 字段超长（理论不可达）
    pub fn cn() -> AipResult<Self> {
        Ok(Self {
            country_code: Some(try_str("CN")?),
            ..Self::default()
        })
    }

    /// 设置组织名称
    pub fn set_organization(&mut self, organization: &str) -> AipResult<()> {
        self.organization = Some(try_str(organization)?);
        Ok(())
    }

    /// 设置提供方名称
    pub fn set_name(&mut self, name: &str) -> AipResult<()> {
        self.name = Some(try_str(name)?);
        Ok(())
    }

    /// 设置联系邮箱
    pub fn set_email(&mut self, email: &str) -> AipResult<()> {
        self.email = Some(try_str(email)?);
        Ok(())
    }

    /// 设置国家/地区代码
    pub fn set_country_code(&mut self, code: &str) -> AipResult<()> {
        self.country_code = Some(try_str(code)?);
        Ok(())
    }
}

/// ACS `endPoints[].transport` 的取值
///
/// 前三个是规范定义的值；`MCU_*` 是本实现的扩展取值，严格遵循 ACS 的消费方
/// 需要由网关翻译。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Transport {
    /// JSON-RPC over HTTP(S)
    #[serde(rename = "JSONRPC")]
    JsonRpc,
    /// 纯 HTTP + JSON
    #[serde(rename = "HTTP_JSON")]
    HttpJson,
    /// AMQP（群组模式）
    #[serde(rename = "AMQP")]
    Amqp,
    /// [MCU 扩展] 有线串口
    #[serde(rename = "MCU_UART")]
    McuUart,
    /// [MCU 扩展] SPI
    #[serde(rename = "MCU_SPI")]
    McuSpi,
    /// [MCU 扩展] 低功耗蓝牙
    #[serde(rename = "MCU_BLE")]
    McuBle,
    /// [MCU 扩展] WiFi
    #[serde(rename = "MCU_WIFI")]
    McuWifi,
}

impl Transport {
    /// 是否为规范定义的取值（非 MCU 扩展）
    pub const fn is_acps_standard(self) -> bool {
        matches!(
            self,
            Transport::JsonRpc | Transport::HttpJson | Transport::Amqp
        )
    }
}

/// ACS `endPoints[]`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct AgentEndPoint {
    /// 端点 URL，如 `aip://gateway.local/agents/esp32-01`
    pub url: heapless::String<128>,
    /// 传输方式
    pub transport: Transport,
}

impl AgentEndPoint {
    /// 创建端点
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - URL 超过 128 字节
    pub fn new(url: &str, transport: Transport) -> AipResult<Self> {
        Ok(Self {
            url: try_str(url)?,
            transport,
        })
    }
}

/// MCU 扩展：根级 `x-mcu`
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct McuSpecExtensions {
    /// 硬件能力
    #[serde(default)]
    pub capabilities: heapless::Vec<AgentCapability, 4>,
}

impl McuSpecExtensions {
    /// 是否为空（为空时不会出现在 JSON 中）
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

/// 智能体能力描述（ACS 的 MCU 子集）
///
/// 字段名与 JSON 大小写对齐 ACS；构造即校验（`aic` 必须是合法 AIC）。
///
/// 容量限制（适配 MCU）：
/// - `name` 64B，`description` 256B，`version` 16B，`lastModifiedTime` 32B
/// - `endPoints` 最多 4 个，`defaultInputModes` / `defaultOutputModes` 各最多 4 个
/// - `skills` 最多 8 个，`x-mcu.capabilities` 最多 4 个
///
/// ## 内存占用（`riscv32imc-unknown-none-elf` 实测）
///
/// | 项 | 字节 |
/// |----|------|
/// | `size_of::<AgentCapabilitySpec>()`（1 个技能） | 10 900 |
/// | 每个额外 `AgentSkill` | +1 088 |
/// | 满载（8 个技能 + 4 个端点） | ≈ 18 500 |
///
/// 这意味着两个设计约束：
///
/// 1. **不要把本结构体放在栈上**。用 `static`/`static mut`（`.bss`）或 `Box`
///    承载，只把 `&mut` 传进构建函数。
/// 2. **设置接口一律是 `&mut self`**（`set_*` / `add_*`）。按值 builder
///    （`with_*(self) -> Self`）每调用一次就要复制整个结构体，实测链式构建会产生
///    约 36 KB 的栈帧，在 32 KB RAM 的 MCU 上直接爆栈。
///
/// ## 与规范的对应
///
/// `securitySchemes` 未实现（MCU 无法独立承担 TLS/mTLS），
/// MCU 特有字段放在 `x-mcu` 命名空间，为空时不序列化。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilitySpec {
    /// 智能体身份码（AIC）
    pub aic: Aic,
    /// 是否激活
    pub active: bool,
    /// 最近修改时间（ISO-8601）
    pub last_modified_time: heapless::String<32>,
    /// 遵循的 ACPs 协议版本
    pub protocol_version: heapless::String<8>,
    /// 显示名称
    pub name: heapless::String<64>,
    /// 功能描述
    pub description: heapless::String<256>,
    /// 智能体版本
    pub version: heapless::String<16>,
    /// 提供方信息
    pub provider: AgentProvider,
    /// 端点列表
    pub end_points: heapless::Vec<AgentEndPoint, 4>,
    /// 协议能力
    pub capabilities: AgentCapabilities,
    /// 默认输入 MIME 类型
    pub default_input_modes: heapless::Vec<MimeMode, 4>,
    /// 默认输出 MIME 类型
    pub default_output_modes: heapless::Vec<MimeMode, 4>,
    /// 技能列表
    pub skills: heapless::Vec<AgentSkill, 8>,
    /// [MCU 扩展] 根级扩展信息
    #[serde(
        rename = "x-mcu",
        default,
        skip_serializing_if = "McuSpecExtensions::is_empty"
    )]
    pub mcu: McuSpecExtensions,
}

impl AgentCapabilitySpec {
    /// 创建能力描述
    ///
    /// 默认 `active = true`、`version = "0.1.0"`、`protocolVersion = "02.01"`、
    /// `defaultInputModes` / `defaultOutputModes` 为 `application/json`。
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 字符串超出容量限制
    pub fn new(aic: Aic, name: &str, description: &str) -> AipResult<Self> {
        let mut default_input_modes = heapless::Vec::new();
        default_input_modes
            .push(MimeMode::json())
            .map_err(|_| AipError::BufferOverflow)?;
        let mut default_output_modes = heapless::Vec::new();
        default_output_modes
            .push(MimeMode::json())
            .map_err(|_| AipError::BufferOverflow)?;

        Ok(Self {
            aic,
            active: true,
            last_modified_time: try_str(DEFAULT_LAST_MODIFIED_TIME)?,
            protocol_version: try_str(ACPS_PROTOCOL_VERSION)?,
            name: try_str(name)?,
            description: try_str(description)?,
            version: try_str("0.1.0")?,
            provider: AgentProvider::cn()?,
            end_points: heapless::Vec::new(),
            capabilities: AgentCapabilities::none(),
            default_input_modes,
            default_output_modes,
            skills: heapless::Vec::new(),
            mcu: McuSpecExtensions::default(),
        })
    }

    /// 设置智能体版本
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 16 字节
    pub fn set_version(&mut self, version: &str) -> AipResult<()> {
        self.version = try_str(version)?;
        Ok(())
    }

    /// 覆盖显示名称
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 64 字节
    pub fn set_name(&mut self, name: &str) -> AipResult<()> {
        self.name = try_str(name)?;
        Ok(())
    }

    /// 覆盖功能描述
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 256 字节
    pub fn set_description(&mut self, description: &str) -> AipResult<()> {
        self.description = try_str(description)?;
        Ok(())
    }

    /// 设置是否激活
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// 设置最近修改时间（ISO-8601）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 32 字节
    pub fn set_last_modified_time(&mut self, timestamp: &str) -> AipResult<()> {
        self.last_modified_time = try_str(timestamp)?;
        Ok(())
    }

    /// 设置协议版本
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 超过 8 字节
    pub fn set_protocol_version(&mut self, version: &str) -> AipResult<()> {
        self.protocol_version = try_str(version)?;
        Ok(())
    }

    /// 设置提供方信息
    pub fn set_provider(&mut self, provider: AgentProvider) {
        self.provider = provider;
    }

    /// 设置协议能力
    pub fn set_capabilities(&mut self, capabilities: AgentCapabilities) {
        self.capabilities = capabilities;
    }

    /// 添加端点
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 端点超过 4 个
    pub fn add_end_point(&mut self, end_point: AgentEndPoint) -> AipResult<()> {
        self.end_points
            .push(end_point)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 添加技能
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 技能超过 8 个
    pub fn add_skill(&mut self, skill: AgentSkill) -> AipResult<()> {
        self.skills
            .push(skill)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 添加硬件能力（MCU 扩展）
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 能力超过 4 个
    pub fn add_hardware_capability(&mut self, capability: AgentCapability) -> AipResult<()> {
        self.mcu
            .capabilities
            .push(capability)
            .map_err(|_| AipError::BufferOverflow)
    }

    /// 是否具备指定硬件能力（MCU 扩展）
    pub fn has_hardware_capability(&self, capability: &AgentCapability) -> bool {
        self.mcu.capabilities.iter().any(|c| c == capability)
    }

    /// 按 `skill_id` 查找技能（供 Message/Stream 分发使用）
    pub fn find_skill(&self, skill_id: &str) -> Option<&AgentSkill> {
        self.skills.iter().find(|s| s.id == skill_id)
    }

    /// 校验必填字段是否满足 ACS 约束
    ///
    /// 检查项（仅覆盖规范标记为 required 的字段是否**有内容且格式合理**）：
    /// `name` / `description` / `version` / `protocolVersion` / `lastModifiedTime` 非空，
    /// `lastModifiedTime` 具备 `YYYY-MM-DDThh:mm:ss…` 结构，
    /// `defaultInputModes` / `defaultOutputModes` 非空。
    ///
    /// `aic` 在构造期已校验；`endPoints` / `skills` 允许为空（表示尚未声明能力）。
    ///
    /// # 错误
    /// - [`AipError::InvalidAcs`] - 任一必填字段不满足约束
    pub fn validate(&self) -> AipResult<()> {
        if self.name.is_empty()
            || self.description.is_empty()
            || self.version.is_empty()
            || self.protocol_version.is_empty()
            || self.last_modified_time.is_empty()
        {
            return Err(AipError::InvalidAcs);
        }
        if !is_iso8601_like(self.last_modified_time.as_str()) {
            return Err(AipError::InvalidAcs);
        }
        if self.default_input_modes.is_empty() || self.default_output_modes.is_empty() {
            return Err(AipError::InvalidAcs);
        }
        Ok(())
    }
}

/// 粗略校验 ISO-8601 结构：`YYYY-MM-DDThh:mm:ss`（至少 19 字符）
fn is_iso8601_like(ts: &str) -> bool {
    let b = ts.as_bytes();
    if b.len() < 19 {
        return false;
    }
    b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[8..10].iter().all(|c| c.is_ascii_digit())
        && b[11..13].iter().all(|c| c.is_ascii_digit())
        && b[14..16].iter().all(|c| c.is_ascii_digit())
        && b[17..19].iter().all(|c| c.is_ascii_digit())
}

#[cfg(feature = "alloc")]
impl AgentCapabilitySpec {
    /// 序列化为 JSON 字符串（需要 `alloc` feature）
    pub fn to_json(&self) -> AipResult<alloc::string::String> {
        serde_json::to_string(self).map_err(|_| AipError::CodecError)
    }

    /// 从 JSON 字符串反序列化（需要 `alloc` feature）
    ///
    /// `aic` 字段会在反序列化时校验，非法 AIC 返回 [`AipError::CodecError`]。
    pub fn from_json(json: &str) -> AipResult<Self> {
        serde_json::from_str(json).map_err(|_| AipError::CodecError)
    }
}
