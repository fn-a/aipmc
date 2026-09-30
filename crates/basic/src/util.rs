//! # 内部工具函数
//!
//! 仅供 crate 内部使用的辅助函数，不对外导出。

use crate::error::{AipError, AipResult};

/// 将 `&str` 转换为定长 `heapless::String<N>`
///
/// # 错误
/// - [`AipError::BufferOverflow`] - 输入字符串长度超过 `N`
#[inline]
pub(crate) fn try_str<const N: usize>(s: &str) -> AipResult<heapless::String<N>> {
    heapless::String::try_from(s).map_err(|_| AipError::BufferOverflow)
}
