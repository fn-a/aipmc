//! # aipmc-transport: AIP 智能体互联协议传输层
//!
//! 提供 ACPs 协议消息在不同物理通道上的传输抽象，基于 `embedded-io` trait。
//!
//! ## 支持的传输方式
//!
//! | 模块       | 传输方式 | 适用场景           | 典型波特率/带宽  |
//! |------------|----------|--------------------|------------------|
//! | [`uart`]   | UART     | MCU 间有线通信     | 9600~921600 bps  |
//! | [`wifi`]   | WiFi     | ESP-32 无线联网    | 1~54 Mbps        |
//! | [`ble`]    | BLE      | 低功耗无线         | 1~2 Mbps         |

#![no_std]

pub mod ble;
pub mod uart;
pub mod wifi;

pub use uart::UartTransport;
