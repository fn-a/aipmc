//! # STM-32 硬件抽象层
//!
//! [`Stm32Hal`] 封装 STM-32 的硬件初始化和设备信息查询，
//! 作为 AIP 协议栈与 STM-32 底层外设之间的桥梁。
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 `stm32-hal` crate 后完善。

use basic::agent::AgentCard;
use basic::error::AipResult;

/// STM-32 硬件抽象层
///
/// 提供 UART/SPI 初始化、设备信息查询等平台特定功能。
/// 在实际部署中，会依赖 `stm32-hal` 等 crate。
pub struct Stm32Hal;

impl Stm32Hal {
    /// 创建 STM-32 HAL 实例
    pub fn new() -> Self {
        Self
    }

    /// 初始化 UART 模块
    ///
    /// TODO: 对接 STM-32 HAL 实现 UART 初始化流程
    pub fn init_uart(&mut self) -> AipResult<()> {
        todo!("STM-32 UART init via stm32 HAL")
    }

    /// 获取设备默认信息，生成 AgentCard
    ///
    /// 返回包含 STM-32 默认标识的智能体卡片。
    pub fn get_device_info(&self) -> AipResult<AgentCard> {
        AgentCard::new("stm32-default", "STM32-Agent", "STM-32 AIP agent node")
    }
}
