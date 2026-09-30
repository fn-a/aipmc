//! 传输层测试

mod frame;
mod stream;

use core::convert::Infallible;
use std::collections::VecDeque;

use embedded_io::{ErrorType, Read, Write};

/// 可复现的收发闭环设备：`rx` 为待读入字节，`tx` 记录写出的字节
///
/// 用于验证"写出恰好一帧"和"读入恰好一帧"，不需要真实串口。
#[derive(Debug, Default)]
pub(crate) struct MockDuplex {
    /// 待读取的字节
    pub(crate) rx: VecDeque<u8>,
    /// 已写出的字节
    pub(crate) tx: Vec<u8>,
}

impl MockDuplex {
    /// 构造一个"输入已预置"的设备
    pub(crate) fn with_input(bytes: &[u8]) -> Self {
        Self {
            rx: bytes.iter().copied().collect(),
            tx: Vec::new(),
        }
    }
}

impl ErrorType for MockDuplex {
    type Error = Infallible;
}

impl Read for MockDuplex {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        let n = buf.len().min(self.rx.len());
        for slot in buf.iter_mut().take(n) {
            // n 已由 rx.len() 约束，pop_front 必然成功
            *slot = self.rx.pop_front().unwrap_or(0);
        }
        Ok(n)
    }
}

impl Write for MockDuplex {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// 立即失败的设备，用于验证底层错误映射
#[derive(Debug)]
pub(crate) struct FailingDevice {
    /// 每次读/写返回的错误类型
    pub(crate) kind: embedded_io::ErrorKind,
}

impl ErrorType for FailingDevice {
    type Error = embedded_io::ErrorKind;
}

impl Read for FailingDevice {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Err(self.kind)
    }
}

impl Write for FailingDevice {
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        Err(self.kind)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Err(self.kind)
    }
}
