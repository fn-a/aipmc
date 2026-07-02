//! # aipmc: AIP 智能体互联协议 MCU 实现
//!
//! 本 crate 是 AIPMC 项目的统一导出层，将所有子模块重新导出，
//! 用户只需引入 `aipmc` 即可使用完整的协议栈功能。
//!
//! ## 子模块
//!
//! | 模块         | crate           | 说明                         |
//! |--------------|-----------------|------------------------------|
//! | [`basic`]     | aipmc-basic      | 核心协议库（协议/消息/编解码） |
//! | [`transport`] | aipmc-transport | 传输层（UART/WiFi/BLE）      |
//! | [`esp32`]    | aipmc-esp32     | ESP-32 平台适配（需 feature） |
//! | [`stm32`]    | aipmc-stm32     | STM-32 平台适配（需 feature） |
//!
//! ## 使用示例
//!
//! ```rust
//! use aipmc::basic::{AgentCard, AgentSkill, AcpMessage, AcpMethod, AgentCapability};
//!
//! let agent = AgentCard::new("dev-01", "MyAgent", "A test agent")
//!     .unwrap()
//!     .with_capability(AgentCapability::SensorRead)
//!     .unwrap()
//!     .with_skill(AgentSkill::new("temp", "ReadTemp", "Read temperature").unwrap())
//!     .unwrap();
//!
//! let msg = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"hello").unwrap();
//! let encoded = msg.encode().unwrap();
//! let decoded = AcpMessage::decode(&encoded).unwrap();
//! assert_eq!(decoded.session_id, 0);
//! ```

/// 核心协议库：协议定义、消息编解码、智能体卡片、技能描述
pub use basic;

/// 传输层：UART/WiFi/BLE 传输抽象
pub use transport;

/// ESP-32 平台适配层（需启用 `esp32` feature）
#[cfg(feature = "esp32")]
pub use esp32;

/// STM-32 平台适配层（需启用 `stm32` feature）
#[cfg(feature = "stm32")]
pub use stm32;