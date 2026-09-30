//! # UART 传输
//!
//! [`UartTransport`] 是 [`StreamTransport`](crate::stream::StreamTransport) 的
//! UART 场景别名：UART 是**有序字节流**，因此只需要"按帧读、整帧写"的分帧逻辑。
//!
//! ## 典型波特率
//!
//! | 场景             | 波特率            |
//! |------------------|-------------------|
//! | 调试/低带宽      | 9600 ~ 115200 bps |
//! | MCU 间高速互联   | 460800 ~ 921600 bps |
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use transport::UartTransport;
//!
//! let mut transport = UartTransport::new(uart);
//! transport.send(&msg)?;
//! let reply = transport.recv()?;
//! ```

/// UART 传输通道
///
/// 泛型参数 `T` 为实现了 `embedded_io::Read + embedded_io::Write` 的 UART 外设。
pub type UartTransport<T> = crate::stream::StreamTransport<T>;
