//! # aipmc-stm32: AIP 智能体互联协议 STM-32 平台适配层
//!
//! 提供 STM-32 系列芯片的硬件抽象层（HAL），封装芯片系列识别、UART/SPI 初始化、
//! 设备信息查询等平台特定功能，使上层协议栈无需关注硬件细节。
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
//! 同时启用多个系列 feature 会触发 `compile_error!`。
//!
//! ## 当前状态
//!
//! `stm32-hal` / `embassy-stm32` 尚未对接。已经可用的能力是系列识别与 ACS 能力描述
//! （`AgentCapabilitySpec`）生成；`init_uart` 返回
//! [`AipError::NotImplemented`](basic::error::AipError::NotImplemented)，不会 panic。

#![no_std]

pub mod hal;

pub use hal::{Series, Stm32Hal, series};
