//! # ESP-32 硬件抽象层
//!
//! [`Esp32Hal`] 封装 ESP-32 的芯片识别、无线模块初始化与能力描述生成，
//! 作为 AIP 协议栈与 ESP-32 底层外设之间的桥梁。
//!
//! ## 芯片选择
//!
//! 通过 Cargo feature 选择目标芯片（最多一个，未启用时按 `esp32` 处理）：
//!
//! | Feature   | 芯片型号  |
//! |-----------|-----------|
//! | `esp32`   | ESP32     |
//! | `esp32s2` | ESP32-S2  |
//! | `esp32s3` | ESP32-S3  |
//! | `esp32c3` | ESP32-C3  |
//! | `esp32c6` | ESP32-C6  |
//!
//! ## 当前状态
//!
//! `esp-hal` / `esp-wifi` 尚未对接：`init_wifi` / `init_ble` 只提供占位实现，
//! 返回 [`AipError::NotImplemented`](basic::error::AipError::NotImplemented)（**不再 panic**）。
//! 这两个方法分别由 `wifi` / `ble` feature 控制。
//!
//! ## AIC 的来源
//!
//! [`AgentCapabilitySpec`] 的必填字段 `aic` 由注册
//! 中心签发，MCU 侧只做**校验**（见 `basic::aic`），不在设备上生成。因此
//! [`Esp32Hal::capability_spec`] 要求调用方传入已预置的 AIC。

use basic::acs::AgentCapabilitySpec;
use basic::aic::Aic;
use basic::error::AipResult;

// 同时启用多个芯片 feature 时直接编译失败，避免"静默选了其中一个"。
#[cfg(any(
    all(feature = "esp32", feature = "esp32s2"),
    all(feature = "esp32", feature = "esp32s3"),
    all(feature = "esp32", feature = "esp32c3"),
    all(feature = "esp32", feature = "esp32c6"),
    all(feature = "esp32s2", feature = "esp32s3"),
    all(feature = "esp32s2", feature = "esp32c3"),
    all(feature = "esp32s2", feature = "esp32c6"),
    all(feature = "esp32s3", feature = "esp32c3"),
    all(feature = "esp32s3", feature = "esp32c6"),
    all(feature = "esp32c3", feature = "esp32c6"),
))]
compile_error!("ESP-32 芯片 feature 只能启用一个：esp32 / esp32s2 / esp32s3 / esp32c3 / esp32c6");

/// 受支持的 ESP-32 芯片型号
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Chip {
    /// ESP32（默认）
    #[default]
    Esp32,
    /// ESP32-S2
    Esp32S2,
    /// ESP32-S3
    Esp32S3,
    /// ESP32-C3
    Esp32C3,
    /// ESP32-C6
    Esp32C6,
}

impl Chip {
    /// 芯片型号的小写标识，可用于构造端点 URL 或日志标识
    pub const fn id_prefix(self) -> &'static str {
        match self {
            Chip::Esp32 => "esp32",
            Chip::Esp32S2 => "esp32s2",
            Chip::Esp32S3 => "esp32s3",
            Chip::Esp32C3 => "esp32c3",
            Chip::Esp32C6 => "esp32c6",
        }
    }

    /// 芯片的展示名（可直接用作 ACS `name` 的默认值）
    pub const fn display_name(self) -> &'static str {
        match self {
            Chip::Esp32 => "ESP32",
            Chip::Esp32S2 => "ESP32-S2",
            Chip::Esp32S3 => "ESP32-S3",
            Chip::Esp32C3 => "ESP32-C3",
            Chip::Esp32C6 => "ESP32-C6",
        }
    }

    /// 芯片是否具备 BLE 能力
    ///
    /// ESP32 / ESP32-S3 / ESP32-C3 / ESP32-C6 均支持 BLE；ESP32-S2 无蓝牙。
    pub const fn has_ble(self) -> bool {
        !matches!(self, Chip::Esp32S2)
    }
}

#[cfg(feature = "esp32")]
const SELECTED_CHIP: Chip = Chip::Esp32;
#[cfg(feature = "esp32s2")]
const SELECTED_CHIP: Chip = Chip::Esp32S2;
#[cfg(feature = "esp32s3")]
const SELECTED_CHIP: Chip = Chip::Esp32S3;
#[cfg(feature = "esp32c3")]
const SELECTED_CHIP: Chip = Chip::Esp32C3;
#[cfg(feature = "esp32c6")]
const SELECTED_CHIP: Chip = Chip::Esp32C6;
#[cfg(not(any(
    feature = "esp32",
    feature = "esp32s2",
    feature = "esp32s3",
    feature = "esp32c3",
    feature = "esp32c6"
)))]
const SELECTED_CHIP: Chip = Chip::Esp32;

/// 返回编译期选定的芯片型号
pub const fn chip() -> Chip {
    SELECTED_CHIP
}

/// ESP-32 硬件抽象层
///
/// 提供无线模块初始化、能力描述生成等平台特定功能。
/// 在实际部署中，会依赖 `esp-hal` 和 `esp-wifi` crate。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Esp32Hal;

impl Esp32Hal {
    /// 创建 ESP-32 HAL 实例
    pub const fn new() -> Self {
        Self
    }

    /// 返回当前编译目标芯片型号
    pub const fn chip(&self) -> Chip {
        chip()
    }

    /// 初始化 WiFi 模块
    ///
    /// 需要启用 `wifi` feature。
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`](basic::error::AipError::NotImplemented) - `esp-wifi` 尚未对接（当前总是返回）
    #[cfg(feature = "wifi")]
    pub fn init_wifi(&mut self) -> AipResult<()> {
        Err(basic::error::AipError::NotImplemented)
    }

    /// 初始化 BLE 模块
    ///
    /// 需要启用 `ble` feature。
    ///
    /// # 错误
    /// - [`AipError::NotImplemented`](basic::error::AipError::NotImplemented) - `esp-wifi` 的 BLE 部分尚未对接（当前总是返回）
    #[cfg(feature = "ble")]
    pub fn init_ble(&mut self) -> AipResult<()> {
        Err(basic::error::AipError::NotImplemented)
    }

    /// 生成该芯片的能力描述骨架（ACS `AgentCapabilitySpec`）
    ///
    /// `name` 默认取芯片展示名（如 `"ESP32-S3"`），`description` 为固定文案；
    /// 调用方可用 `AgentCapabilitySpec::set_name` / `set_description` 覆盖，
    /// 并追加端点、技能与硬件能力。
    ///
    /// # 参数
    /// - `aic`: 由注册中心签发、预置在设备上的智能体身份码
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`](basic::error::AipError::BufferOverflow) - 字段超出 ACS 容量
    pub fn capability_spec(&self, aic: Aic) -> AipResult<AgentCapabilitySpec> {
        AgentCapabilitySpec::new(aic, chip().display_name(), "ESP-32 ACPs agent node")
    }
}
