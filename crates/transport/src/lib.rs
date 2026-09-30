//! # aipmc-transport: AIP 智能体互联协议传输层
//!
//! 提供 ACPs 协议消息在不同物理通道上的传输抽象，基于 `embedded-io` trait。
//! 本 crate 是 `no_std` 的，只依赖 `basic` 与 `embedded-io`，不进行动态分配。
//!
//! ## 分层
//!
//! ```text
//! ┌───────────────────────────────────────────────┐
//! │ StreamTransport<T>   (UART / SPI 别名)         │  整帧发送 + 按帧接收
//! ├───────────────────────────────────────────────┤
//! │ FrameReader<T>                                 │  read_exact 分帧 + 校验
//! ├───────────────────────────────────────────────┤
//! │ embedded_io::Read + Write                      │  厂商 HAL / 驱动
//! └───────────────────────────────────────────────┘
//! ```
//!
//! ## 支持的传输方式
//!
//! | 模块      | 传输方式 | 适用场景           | 状态       |
//! |-----------|----------|--------------------|------------|
//! | [`uart`]  | UART     | MCU 间有线通信     | 已实现     |
//! | [`spi`]   | SPI      | 高速板级通信       | 已实现     |
//! | [`wifi`]  | WiFi     | ESP-32 无线联网    | 占位，未对接 |
//! | [`ble`]   | BLE      | 低功耗无线         | 占位，未对接 |
//!
//! > I2C 不是"有序字节流"（每次事务带从机地址与读写方向），无法直接套用
//! > `StreamTransport`；如需支持请基于目标平台的 I2C HAL 单独实现，参见
//! > 根目录 README 的路线图。

#![no_std]

pub mod ble;
pub mod frame;
pub mod spi;
pub mod stream;
pub mod uart;
pub mod wifi;

pub use ble::BleTransport;
pub use frame::{FRAME_BODY_LEN, FrameReader, map_io_error, map_read_exact_error};
pub use spi::SpiTransport;
pub use stream::StreamTransport;
pub use uart::UartTransport;
pub use wifi::WifiTransport;
