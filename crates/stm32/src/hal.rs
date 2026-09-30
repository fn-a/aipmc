//! # STM-32 硬件抽象层
//!
//! [`Stm32Hal`] 封装 STM-32 的芯片系列识别、外设初始化与能力描述生成，
//! 作为 AIP 协议栈与 STM-32 底层外设之间的桥梁。
//!
//! ## 芯片系列选择
//!
//! 通过 Cargo feature 选择目标系列（最多一个，未启用时按 `stm32f4` 处理）：
//!
//! | Feature    | 芯片系列   |
//! |------------|------------|
//! | `stm32f4`  | STM32F4xx  |
//! | `stm32h7`  | STM32H7xx  |
//! | `stm32l4`  | STM32L4xx  |
//! | `stm32g4`  | STM32G4xx  |
//!
//! ## 当前状态
//!
//! `stm32-hal` / `embassy-stm32` 尚未对接：`init_uart` 只提供占位实现，
//! 返回 [`AipError::NotImplemented`](basic::error::AipError::NotImplemented)（**不再 panic**）。
//!
//! ## AIC 的来源
//!
//! [`AgentCapabilitySpec`] 的必填字段 `aic` 由注册
//! 中心签发，MCU 侧只做**校验**（见 `basic::aic`），不在设备上生成。因此
//! [`Stm32Hal::capability_spec`] 要求调用方传入已预置的 AIC。

use basic::acs::AgentCapabilitySpec;
use basic::aic::Aic;
use basic::error::AipResult;

// 同时启用多个系列 feature 时直接编译失败，避免"静默选了其中一个"。
#[cfg(any(
    all(feature = "stm32f4", feature = "stm32h7"),
    all(feature = "stm32f4", feature = "stm32l4"),
    all(feature = "stm32f4", feature = "stm32g4"),
    all(feature = "stm32h7", feature = "stm32l4"),
    all(feature = "stm32h7", feature = "stm32g4"),
    all(feature = "stm32l4", feature = "stm32g4"),
))]
compile_error!("STM-32 系列 feature 只能启用一个：stm32f4 / stm32h7 / stm32l4 / stm32g4");

/// 受支持的 STM-32 芯片系列
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Series {
    /// STM32F4xx（默认）
    #[default]
    Stm32F4,
    /// STM32H7xx
    Stm32H7,
    /// STM32L4xx
    Stm32L4,
    /// STM32G4xx
    Stm32G4,
}

impl Series {
    /// 系列的小写标识，可用于构造端点 URL 或日志标识
    pub const fn id_prefix(self) -> &'static str {
        match self {
            Series::Stm32F4 => "stm32f4",
            Series::Stm32H7 => "stm32h7",
            Series::Stm32L4 => "stm32l4",
            Series::Stm32G4 => "stm32g4",
        }
    }

    /// 系列的展示名（可直接用作 ACS `name` 的默认值）
    pub const fn display_name(self) -> &'static str {
        match self {
            Series::Stm32F4 => "STM32F4",
            Series::Stm32H7 => "STM32H7",
            Series::Stm32L4 => "STM32L4",
            Series::Stm32G4 => "STM32G4",
        }
    }
}

#[cfg(feature = "stm32f4")]
const SELECTED_SERIES: Series = Series::Stm32F4;
#[cfg(feature = "stm32h7")]
const SELECTED_SERIES: Series = Series::Stm32H7;
#[cfg(feature = "stm32l4")]
const SELECTED_SERIES: Series = Series::Stm32L4;
#[cfg(feature = "stm32g4")]
const SELECTED_SERIES: Series = Series::Stm32G4;
#[cfg(not(any(
    feature = "stm32f4",
    feature = "stm32h7",
    feature = "stm32l4",
    feature = "stm32g4"
)))]
const SELECTED_SERIES: Series = Series::Stm32F4;

/// 返回编译期选定的芯片系列
pub const fn series() -> Series {
    SELECTED_SERIES
}

/// STM-32 硬件抽象层
///
/// 提供 UART/SPI 初始化、能力描述生成等平台特定功能。
/// 在实际部署中，会依赖 `stm32-hal` / `embassy-stm32` 等 crate。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stm32Hal;

impl Stm32Hal {
    /// 创建 STM-32 HAL 实例
    pub const fn new() -> Self {
        Self
    }

    /// 返回当前编译目标芯片系列
    pub const fn series(&self) -> Series {
        series()
    }

    /// 初始化 UART 模块
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`](basic::error::AipError::NotImplemented) - STM-32 HAL 尚未对接（当前总是返回）
    pub fn init_uart(&mut self) -> AipResult<()> {
        Err(basic::error::AipError::NotImplemented)
    }

    /// 生成该系列的智能体能力描述骨架（ACS `AgentCapabilitySpec`）
    ///
    /// `name` 默认取系列展示名（如 `"STM32H7"`），调用方可用
    /// `AgentCapabilitySpec::set_name` / `set_description` 覆盖，并追加端点、
    /// 技能与硬件能力。
    ///
    /// # 参数
    /// - `aic`: 由注册中心签发、预置在设备上的智能体身份码
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`](basic::error::AipError::BufferOverflow) - 字段超出 ACS 容量
    pub fn capability_spec(&self, aic: Aic) -> AipResult<AgentCapabilitySpec> {
        AgentCapabilitySpec::new(aic, series().display_name(), "STM-32 ACPs agent node")
    }
}
