//! # SPI 传输
//!
//! [`SpiTransport`] 是 [`StreamTransport`](crate::stream::StreamTransport) 的
//! SPI 场景别名。SPI 本身是**面向事务**的总线：需要由板级代码把"片选 + 读写"
//! 封装成实现了 `embedded_io::Read + Write` 的字节流句柄（例如通过 DMA 中断
//! 驱动的收发队列），随后即可复用完全相同的分帧逻辑。
//!
//! ## 典型速率
//!
//! | 场景             | 速率         |
//! |------------------|--------------|
//! | 板级高速互联     | 1 ~ 40 Mbps  |
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use transport::SpiTransport;
//!
//! // `spi_stream` 需实现 embedded_io::Read + Write
//! let mut transport = SpiTransport::new(spi_stream);
//! transport.send(&msg)?;
//! let reply = transport.recv()?;
//! ```

/// SPI 传输通道
///
/// 泛型参数 `T` 为实现了 `embedded_io::Read + embedded_io::Write` 的 SPI 字节流句柄。
pub type SpiTransport<T> = crate::stream::StreamTransport<T>;
