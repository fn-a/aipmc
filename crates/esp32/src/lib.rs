//! # aipmc-esp32: AIP 智能体互联协议 ESP-32 平台适配层
//!
//! 提供 ESP-32 系列芯片的硬件抽象层（HAL），封装 WiFi/BLE 初始化、
//! GPIO 控制等平台特定功能，使上层协议栈无需关注硬件细节。
//!
//! ## 支持的芯片
//!
//! | Feature   | 芯片型号      |
//! |-----------|--------------|
//! | `esp32`   | ESP32        |
//! | `esp32s2` | ESP32-S2     |
//! | `esp32s3` | ESP32-S3     |
//! | `esp32c3` | ESP32-C3     |
//! | `esp32c6` | ESP32-C6     |
//!
//! ## 当前状态
//!
//! 骨架实现，待对接 `esp-hal` / `esp-wifi` 后完善。

#![no_std]

pub mod hal;

pub use hal::Esp32Hal;
