//! # aipmc-esp32: AIP 智能体互联协议 ESP-32 平台适配层
//!
//! 提供 ESP-32 系列芯片的硬件抽象层（HAL），封装芯片识别、WiFi/BLE 初始化、
//! 设备信息查询等平台特定功能，使上层协议栈无需关注硬件细节。
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
//! 同时启用多个芯片 feature 会触发 `compile_error!`。
//!
//! ## 当前状态
//!
//! `esp-hal` / `esp-wifi` 尚未对接。已经可用的能力是芯片识别与 ACS 能力描述
//! （`AgentCapabilitySpec`）生成；`init_wifi`（需 `wifi` feature）与 `init_ble`
//! （需 `ble` feature）返回
//! [`AipError::NotImplemented`](basic::error::AipError::NotImplemented)，
//! 不会 panic。

#![no_std]

pub mod hal;

pub use hal::{Chip, Esp32Hal, chip};
