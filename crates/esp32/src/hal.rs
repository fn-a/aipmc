//! # ESP-32 硬件抽象层
//!
//! [`Esp32Hal`] 封装 ESP-32 的硬件初始化和设备信息查询，
//! 作为 AIP 协议栈与 ESP-32 底层外设之间的桥梁。
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 `esp-hal` / `esp-wifi` 后完善。

use basic::agent::AgentCard;
use basic::error::AipResult;

/// ESP-32 硬件抽象层
///
/// 提供 WiFi/BLE 初始化、设备信息查询等平台特定功能。
/// 在实际部署中，会依赖 `esp-hal` 和 `esp-wifi` crate。
pub struct Esp32Hal;

impl Esp32Hal {
    /// 创建 ESP-32 HAL 实例
    pub fn new() -> Self {
        Self
    }

    /// 初始化 WiFi 模块
    ///
    /// TODO: 对接 `esp-wifi` crate 实现 WiFi 初始化流程
    pub fn init_wifi(&mut self) -> AipResult<()> {
        todo!("ESP-32 WiFi init via esp-wifi")
    }

    /// 初始化 BLE 模块
    ///
    /// TODO: 对接 `esp-wifi` crate 实现 BLE 初始化流程
    pub fn init_ble(&mut self) -> AipResult<()> {
        todo!("ESP-32 BLE init via esp-wifi")
    }

    /// 获取设备默认信息，生成 AgentCard
    ///
    /// 返回包含 ESP-32 默认标识的智能体卡片。
    pub fn get_device_info(&self) -> AipResult<AgentCard> {
        AgentCard::new("esp32-default", "ESP32-Agent", "ESP-32 AIP agent node")
    }
}
