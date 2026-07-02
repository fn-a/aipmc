//! # aipmc-stm32: AIP 智能体互联协议 STM-32 平台适配层
//!
//! 提供 STM-32 系列芯片的硬件抽象层（HAL），封装 UART/SPI 初始化、
//! GPIO 控制等平台特定功能，使上层协议栈无需关注硬件细节。
//!
//! ## 支持的芯片系列
//!
//! | Feature    | 芯片系列        |
//! |------------|----------------|
//! | `stm32f4`  | STM32F4xx      |
//! | `stm32h7`  | STM32H7xx      |
//! | `stm32l4`  | STM32L4xx      |
//! | `stm32g4`  | STM32G4xx      |
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 `stm32-hal` 后完善。

#![no_std]

pub mod hal;

pub use hal::Stm32Hal;
