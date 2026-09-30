//! # AIC — Agent Identity Code（智能体身份码）
//!
//! 对齐 ACPs 规范 `ACPs-spec-AIC-v02.01`：智能体身份由 **10 级点分 OID** 表示。
//!
//! ## 结构
//!
//! ```text
//! 1-4:  OID 前缀（1.2.156.3088）
//! 5:    版本号          Base36，1 位
//! 6:    ARSP 注册服务商序号 Base36，1~6 位
//! 7:    Vendor 供应商序号  Base36，1~6 位
//! 8:    Ontology SN 本体序列号 Base36，1~9 位
//! 9:    Instance SN 实例序列号 Base36，1~9 位（全 0 表示本体 AIC）
//! 10:   CRC 校验码        Base36，固定 4 位
//! ```
//!
//! 规范化规则：去除所有空白并转为大写。
//!
//! ## 校验码
//!
//! 第 10 级由**本体码**（1~9 级）的 ASCII 字节序列追加 ARSP 私有 SALT 后，
//! 经 CRC-16/CCITT-FALSE（`poly=0x1021`、`init=0xFFFF`、`refin/refout=false`、
//! `xorout=0`）计算，再以 Base36 编码为固定 4 位（左侧补 `0`）。
//!
//! ## 职责边界
//!
//! 本模块只做**解析与校验**：
//!
//! - **AIC 的生成**（分配 ARSP/Vendor 序号、摇号序列号）属于注册中心职责，
//!   在宿主机侧实现。
//! - MCU 上的 AIC 应当**预置**（编译期常量、Flash 分区或注册流程下发），
//!   本模块用于在使用前验证其合法性。
//!
//! ## 使用示例
//!
//! ```rust
//! use basic::aic::{Aic, REGISTRY_DEFAULT_SALT};
//!
//! // 规范示例（格式校验，不校验 CRC）
//! let aic = Aic::parse_with_prefix(
//!     "1.2.156.1234.1.1.34C2.478BDF.3GF546.0H9T",
//!     "1.2.156.1234",
//! )
//! .unwrap();
//! assert!(aic.is_entity());
//! assert_eq!(aic.vendor(), "34C2");
//!
//! // 带 SALT 的 CRC 校验
//! let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
//! assert!(aic.verify_checksum(REGISTRY_DEFAULT_SALT));
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{AipError, AipResult};

/// AIC 的 OID 前缀（由国家 OID 注册中心分配）
pub const AIC_OID_PREFIX: &str = "1.2.156.3088";

/// 规范示例使用的前缀，仅用于校验规范文档中的示例值
pub const AIC_SPEC_EXAMPLE_PREFIX: &str = "1.2.156.1234";

/// AIC 的段数
pub const AIC_SEGMENTS: usize = 10;

/// 第 5~10 级的长度约束 `(min, max)`
pub const AIC_SEGMENT_LENGTHS: [(usize, usize); 6] = [
    (1, 1), // 5: 版本号
    (1, 6), // 6: ARSP
    (1, 6), // 7: Vendor
    (1, 9), // 8: 本体序列号
    (1, 9), // 9: 实例序列号
    (4, 4), // 10: CRC
];

/// 规范化 AIC 的最大长度（含最长的各级）
pub const AIC_MAX_LEN: usize = 56;

/// 本体码（1~9 级）的最大长度
pub const AIC_BODY_MAX_LEN: usize = 48;

/// 本体前缀（1~8 级）的最大长度
pub const AIC_ONTOLOGY_PREFIX_MAX_LEN: usize = 38;

/// Base36 字母表
pub const BASE36_ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// 注册中心默认的 CRC SALT（十六进制 `00494F41`）
///
/// 生产环境中各 ARSP 使用自己的 SALT，本常量仅对应默认注册中心配置与规范示例。
pub const REGISTRY_DEFAULT_SALT: &[u8] = b"\x00\x49\x4F\x41";

/// CRC-16/CCITT-FALSE 的增量计算器
///
/// 便于对「本体码 + SALT」这类拼接输入做流式计算，避免额外的临时缓冲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc16CcittFalse {
    state: u16,
}

impl Crc16CcittFalse {
    /// 创建计算器（初值 `0xFFFF`）
    pub const fn new() -> Self {
        Self { state: 0xFFFF }
    }

    /// 追加一段数据
    pub fn update(&mut self, data: &[u8]) {
        let mut crc = self.state;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                crc = if crc & 0x8000 != 0 {
                    (crc << 1) ^ 0x1021
                } else {
                    crc << 1
                };
            }
        }
        self.state = crc;
    }

    /// 返回当前校验值（`xorout = 0`）
    pub const fn finish(self) -> u16 {
        self.state
    }
}

impl Default for Crc16CcittFalse {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算 CRC-16/CCITT-FALSE
///
/// ```rust
/// use basic::aic::crc16_ccitt_false;
///
/// // 标准测试向量：CRC-16/CCITT-FALSE("123456789") == 0x29B1
/// assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
/// ```
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc = Crc16CcittFalse::new();
    crc.update(data);
    crc.finish()
}

/// 判断字节是否为 Base36 字符（`0-9A-Z`，大写）
pub const fn is_base36_byte(b: u8) -> bool {
    (b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'Z')
}

/// 判断字符串是否为纯 Base36（大写，非空）
pub fn is_base36(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(is_base36_byte)
}

/// 将 Base36 字符转换为数值
pub const fn base36_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'Z' => Some(b - b'A' + 10),
        b'a'..=b'z' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// 将数值编码为固定长度的 Base36 字符串（左侧补 `0`），写入 `out`
///
/// 数值超出 `out` 可表示的范围时取低位（与上游实现一致）。
///
/// # 返回
/// 写入的字节数，等于 `out.len()`
pub fn base36_encode_into(mut num: u64, out: &mut [u8]) -> usize {
    let modulus = BASE36_ALPHABET.len() as u64;
    for slot in out.iter_mut().rev() {
        *slot = BASE36_ALPHABET[(num % modulus) as usize];
        num /= modulus;
    }
    out.len()
}

/// 将数值编码为定长 Base36 字符串
///
/// # 错误
/// - [`AipError::BufferOverflow`] - `num` 超出 `OUT` 位所能表示的范围
pub fn base36_encode<const OUT: usize>(num: u64) -> AipResult<heapless::String<OUT>> {
    // 检查是否会被截断
    let mut limit: u64 = 1;
    for _ in 0..OUT {
        limit = limit.saturating_mul(BASE36_ALPHABET.len() as u64);
    }
    if num >= limit {
        return Err(AipError::BufferOverflow);
    }
    let mut buf = [0u8; OUT];
    base36_encode_into(num, &mut buf);
    heapless::String::try_from(core::str::from_utf8(&buf).map_err(|_| AipError::InternalError)?)
        .map_err(|_| AipError::BufferOverflow)
}

/// 将 Base36 字符串解码为整数（大小写不敏感）
///
/// # 错误
/// - [`AipError::InvalidMessage`] - 存在非 Base36 字符
/// - [`AipError::BufferOverflow`] - 数值超出 `u64`
pub fn base36_decode(s: &str) -> AipResult<u64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    let mut value: u64 = 0;
    for b in trimmed.bytes() {
        let digit = base36_value(b).ok_or(AipError::InvalidMessage)? as u64;
        value = value
            .checked_mul(BASE36_ALPHABET.len() as u64)
            .and_then(|v| v.checked_add(digit))
            .ok_or(AipError::BufferOverflow)?;
    }
    Ok(value)
}

/// 去除空白并转为大写
///
/// # 错误
/// - [`AipError::BufferOverflow`] - 规范化结果超过 `OUT` 字节
pub fn normalize<const OUT: usize>(s: &str) -> AipResult<heapless::String<OUT>> {
    let mut out: heapless::String<OUT> = heapless::String::new();
    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
        let upper = ch.to_ascii_uppercase();
        out.push(upper).map_err(|_| AipError::BufferOverflow)?;
    }
    Ok(out)
}

/// 计算本体码（1~9 级）对应的 4 位 Base36 校验码
///
/// # 参数
/// - `body_1_9`: AIC 的 1~9 级（可含空白，内部会规范化）
/// - `salt`: ARSP 私有 SALT
///
/// # 错误
/// - [`AipError::BufferOverflow`] - 规范化后的本体码超过 [`AIC_BODY_MAX_LEN`]
pub fn checksum_for<S: AsRef<[u8]>>(body_1_9: &str, salt: S) -> AipResult<heapless::String<4>> {
    let normalized: heapless::String<AIC_BODY_MAX_LEN> = normalize(body_1_9)?;
    let mut crc = Crc16CcittFalse::new();
    crc.update(normalized.as_bytes());
    crc.update(salt.as_ref());
    base36_encode::<4>(crc.finish() as u64)
}

/// 智能体身份码（AIC）
///
/// 内部保存**规范化**后的字符串（大写、无空白），因此 `PartialEq` 与
/// `Serialize` 都基于规范形式：`"1.2.156.3088..." == " 1.2.156.3088... "`。
///
/// 构造即校验：只能通过 [`Aic::parse`] / [`Aic::parse_with_prefix`] 或
/// `Deserialize` 得到实例，因此持有 `Aic` 就意味着格式合法。
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Aic {
    /// 规范化后的完整 AIC
    normalized: heapless::String<AIC_MAX_LEN>,
}

impl Aic {
    /// 解析并校验 AIC，要求前缀为 [`AIC_OID_PREFIX`]
    ///
    /// # 错误
    /// - [`AipError::InvalidAic`] - 段数/字符集/长度/前缀不符合规范
    pub fn parse(aic: &str) -> AipResult<Self> {
        Self::parse_with_prefix(aic, AIC_OID_PREFIX)
    }

    /// 解析并校验 AIC，指定期望的 OID 前缀
    ///
    /// # 参数
    /// - `aic`: 待解析的 AIC（允许含空白与小写）
    /// - `expected_prefix`: 期望的前 4 级；传空字符串表示不校验前缀
    ///
    /// # 错误
    /// - [`AipError::InvalidAic`] - 格式不符合规范
    pub fn parse_with_prefix(aic: &str, expected_prefix: &str) -> AipResult<Self> {
        let normalized: heapless::String<AIC_MAX_LEN> = normalize(aic)?;

        // 段数与空段
        let mut count = 0usize;
        for part in normalized.split('.') {
            if part.is_empty() {
                return Err(AipError::InvalidAic);
            }
            count += 1;
        }
        if count != AIC_SEGMENTS {
            return Err(AipError::InvalidAic);
        }

        // 1~4 级：纯数字，且与期望前缀一致
        for part in normalized.split('.').take(4) {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(AipError::InvalidAic);
            }
        }
        if !expected_prefix.is_empty() {
            let mut parts = normalized.split('.');
            for expected in expected_prefix.split('.') {
                match parts.next() {
                    // 不分配内存的大小写无关比较（`to_ascii_uppercase` 需要 alloc）
                    Some(actual) if actual.eq_ignore_ascii_case(expected) => {}
                    _ => return Err(AipError::InvalidAic),
                }
            }
        }

        // 5~10 级：Base36 + 长度约束
        for (offset, (min, max)) in AIC_SEGMENT_LENGTHS.iter().enumerate() {
            let part = Self::nth_segment(&normalized, offset + 5).ok_or(AipError::InvalidAic)?;
            if !is_base36(part) || part.len() < *min || part.len() > *max {
                return Err(AipError::InvalidAic);
            }
        }

        Ok(Self { normalized })
    }

    /// 返回第 `level` 级（1~10）的内容
    fn nth_segment(s: &str, level: usize) -> Option<&str> {
        if level == 0 {
            return None;
        }
        s.split('.').nth(level - 1)
    }

    /// 规范化后的完整 AIC 字符串
    pub fn as_str(&self) -> &str {
        self.normalized.as_str()
    }

    /// 返回第 `level` 级（1~10）的内容
    pub fn segment(&self, level: usize) -> Option<&str> {
        Self::nth_segment(&self.normalized, level)
    }

    /// 1~4 级：OID 前缀
    pub fn oid_prefix(&self) -> &str {
        self.ontology_prefix_inner(4)
    }

    /// 第 5 级：版本号
    pub fn version(&self) -> &str {
        self.segment(5).unwrap_or("")
    }

    /// 第 6 级：ARSP 注册服务商序号
    pub fn arsp(&self) -> &str {
        self.segment(6).unwrap_or("")
    }

    /// 第 7 级：Vendor 供应商序号
    pub fn vendor(&self) -> &str {
        self.segment(7).unwrap_or("")
    }

    /// 第 8 级：本体序列号
    pub fn ontology_serial(&self) -> &str {
        self.segment(8).unwrap_or("")
    }

    /// 第 9 级：实例序列号（全 `0` 表示本体 AIC）
    pub fn instance_serial(&self) -> &str {
        self.segment(9).unwrap_or("")
    }

    /// 第 10 级：CRC 校验码
    pub fn checksum(&self) -> &str {
        self.segment(10).unwrap_or("")
    }

    /// 前 `levels` 级拼接结果
    fn ontology_prefix_inner(&self, levels: usize) -> &str {
        // 找到第 levels 个 '.' 的位置
        let mut seen = 0usize;
        for (idx, b) in self.normalized.bytes().enumerate() {
            if b == b'.' {
                seen += 1;
                if seen == levels {
                    return &self.normalized[..idx];
                }
            }
        }
        self.normalized.as_str()
    }

    /// 本体码（1~9 级），用于 CRC 计算
    ///
    /// # 错误
    /// - [`AipError::BufferOverflow`] - 理论不可达（长度受 [`AIC_BODY_MAX_LEN`] 约束）
    pub fn body(&self) -> AipResult<heapless::String<AIC_BODY_MAX_LEN>> {
        let len = self
            .normalized
            .len()
            .saturating_sub(self.checksum().len() + 1);
        let body = &self.normalized[..len];
        heapless::String::try_from(body).map_err(|_| AipError::BufferOverflow)
    }

    /// 本体前缀（1~8 级），可用于匹配同一本体下的所有实体
    pub fn ontology_prefix(&self) -> &str {
        self.ontology_prefix_inner(8)
    }

    /// 是否为本体 AIC（实例序列号全为 `0`）
    pub fn is_ontology(&self) -> bool {
        self.instance_serial().bytes().all(|b| b == b'0')
    }

    /// 是否为实体 AIC
    pub fn is_entity(&self) -> bool {
        !self.is_ontology()
    }

    /// 用给定 SALT 校验第 10 级校验码
    ///
    /// SALT 不匹配时返回 `false`（ASRP 私有 SALT 未配置的情况下不应把它当作错误）。
    pub fn verify_checksum<S: AsRef<[u8]>>(&self, salt: S) -> bool {
        match self
            .body()
            .and_then(|body| checksum_for(body.as_str(), salt))
        {
            Ok(expected) => expected.as_str() == self.checksum(),
            Err(_) => false,
        }
    }
}

impl fmt::Display for Aic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Aic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Aic({})", self.as_str())
    }
}

impl core::str::FromStr for Aic {
    type Err = AipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Aic {
    type Error = AipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl Serialize for Aic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Aic {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AicVisitor;

        impl<'a> serde::de::Visitor<'a> for AicVisitor {
            type Value = Aic;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a 10-segment dot-separated Base36 AIC string")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Aic, E> {
                Aic::parse(value).map_err(|_| E::custom("invalid AIC"))
            }
        }

        deserializer.deserialize_str(AicVisitor)
    }
}
