//! # aipmc: AIP 智能体互联协议 MCU 实现
//!
//! 本 crate 是 AIPMC 项目的统一导出层，将所有子模块重新导出，
//! 用户只需引入 `aipmc` 即可使用完整的协议栈功能。
//!
//! ## 子模块
//!
//! | 模块          | crate            | 说明                          |
//! |---------------|------------------|-------------------------------|
//! | [`basic`]     | aipmc-basic      | 核心协议库（协议/消息/编解码）  |
//! | [`transport`] | aipmc-transport  | 传输层（UART/SPI/WiFi/BLE）    |
//! | `esp32`       | aipmc-esp32      | ESP-32 平台适配（需 feature）  |
//! | `stm32`       | aipmc-stm32      | STM-32 平台适配（需 feature）  |
//!
//! ## no_std
//!
//! 本 crate 自身为 `#![no_std]`，三个子 crate 同样是 `no_std`。
//! `std` feature 只用于把标准库能力转发给 [`basic`]（JSON 序列化）。
//! 纯 MCU 场景请关闭默认特性：
//!
//! ```toml
//! [dependencies]
//! aipmc = { version = "0.0.1", default-features = false }
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use aipmc::basic::acs::{AgentCapabilitySpec, AgentEndPoint, Transport};
//! use aipmc::basic::aic::Aic;
//! use aipmc::basic::skill::AgentSkill;
//! use aipmc::basic::{AcpMessage, AcpMethod};
//!
//! // 身份码由注册中心签发；设备侧校验后使用
//! let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
//!
//! // ACS 约 11KB，建议放在 static/.bss；设置接口是 &mut self
//! let mut spec = AgentCapabilitySpec::new(aic, "ESP32-Sensor", "Temperature sensor agent")
//!     .unwrap();
//! let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
//! skill.add_tag("sensor").unwrap();
//! spec.add_skill(skill).unwrap();
//! spec.add_end_point(
//!     AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
//! )
//! .unwrap();
//! assert!(spec.find_skill("temp.read").is_some());
//!
//! // 自有二进制帧：编码自洽、解码不 panic
//! let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
//! let encoded = msg.encode().unwrap();
//! let decoded = AcpMessage::decode(&encoded).unwrap();
//! assert_eq!(decoded.session_id, 0);
//! assert_eq!(&decoded.payload[..], b"hello");
//! ```

#![no_std]

/// 核心协议库：AIC 身份码、ACS 能力描述、技能、自有二进制帧编解码
pub use basic;

/// 传输层：按帧收发与 UART/SPI/WiFi/BLE 抽象
pub use transport;

/// ESP-32 平台适配层（需启用 `esp32` feature）
#[cfg(feature = "esp32")]
pub use esp32;

/// STM-32 平台适配层（需启用 `stm32` feature）
#[cfg(feature = "stm32")]
pub use stm32;
