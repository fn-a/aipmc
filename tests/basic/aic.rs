//! AIC（智能体身份码）解析与校验测试
//!
//! （`1.2.156.1234…` 为 ACPs 规范示例，`…176F` / `…0M9Y` 为注册中心默认 SALT
//! `0x00494F41` 下的独立计算值）。

use aipmc::basic::AipError;
use aipmc::basic::aic::{
    AIC_OID_PREFIX, AIC_SEGMENTS, Aic, REGISTRY_DEFAULT_SALT, base36_decode, base36_encode,
    checksum_for, crc16_ccitt_false, is_base36, normalize,
};

/// 规范示例 AIC（前缀 1.2.156.1234，空 SALT）
const SPEC_EXAMPLE: &str = "1.2.156.1234.1.1.34C2.478BDF.3GF546.0H9T";

/// 注册中心默认 SALT 下的实体 AIC
const ENTITY_AIC: &str = "1.2.156.3088.1.0001.00001.ABC123.000001.176F";

/// 注册中心默认 SALT 下的本体 AIC（第 9 级全 0）
const ONTOLOGY_AIC: &str = "1.2.156.3088.1.0001.00001.ABC123.000000.0M9Y";

// ---- 基础算法 ----

/// CRC-16/CCITT-FALSE 标准测试向量
#[test]
fn test_crc16_ccitt_false_standard_vector() {
    assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
    assert_eq!(crc16_ccitt_false(b""), 0xFFFF);
}

/// 增量 CRC 与一次性计算等价
#[test]
fn test_crc_incremental_matches_oneshot() {
    let mut crc = aipmc::basic::aic::Crc16CcittFalse::new();
    crc.update(b"1234");
    crc.update(b"56789");
    assert_eq!(crc.finish(), crc16_ccitt_false(b"123456789"));
}

/// Base36 编码：定长、左补 0、大写
#[test]
fn test_base36_encode() {
    // CRC-16/CCITT-FALSE("123456789") = 0x29B1 = 10673 → 4 位 Base36 为 "088H"
    assert_eq!(base36_encode::<4>(0x29B1).unwrap().as_str(), "088H");
    // 0 → "0000"
    assert_eq!(base36_encode::<4>(0).unwrap().as_str(), "0000");
    // 35 → "000Z"
    assert_eq!(base36_encode::<4>(35).unwrap().as_str(), "000Z");
    // 36 → "0010"
    assert_eq!(base36_encode::<4>(36).unwrap().as_str(), "0010");
    // 超出位宽必须报错而不是静默截断
    assert!(matches!(
        base36_encode::<1>(36),
        Err(AipError::BufferOverflow)
    ));
}

/// Base36 解码
#[test]
fn test_base36_decode() {
    assert_eq!(base36_decode("0").unwrap(), 0);
    assert_eq!(base36_decode("Z").unwrap(), 35);
    assert_eq!(base36_decode("10").unwrap(), 36);
    assert_eq!(
        base36_decode("  1y41  ").unwrap(),
        base36_decode("1Y41").unwrap()
    );
    assert!(matches!(base36_decode("!"), Err(AipError::InvalidMessage)));
    assert!(matches!(
        base36_decode("ABC!"),
        Err(AipError::InvalidMessage)
    ));
}

/// Base36 字符判定只接受大写
#[test]
fn test_is_base36() {
    assert!(is_base36("0A9Z"));
    assert!(!is_base36("abc"));
    assert!(!is_base36(""));
    assert!(!is_base36("A-1"));
}

/// 规范化：去空白 + 转大写
#[test]
fn test_normalize() {
    let n: heapless::String<64> = normalize(" 1.2.156.3088.1.k9 \n").unwrap();
    assert_eq!(n.as_str(), "1.2.156.3088.1.K9");
}

// ---- 校验码 ----

/// 规范示例：空 SALT 下的黄金值
#[test]
fn test_checksum_spec_example_without_salt() {
    assert_eq!(
        checksum_for("1.2.156.1234.1.1.34C2.478BDF.3GF546", b"")
            .unwrap()
            .as_str(),
        "0H9T"
    );
}

/// 注册中心默认 SALT 下的黄金值
#[test]
fn test_checksum_registry_salt_golden_values() {
    assert_eq!(
        checksum_for(
            "1.2.156.3088.1.0001.00001.ABC123.000001",
            REGISTRY_DEFAULT_SALT
        )
        .unwrap()
        .as_str(),
        "176F"
    );
    assert_eq!(
        checksum_for(
            "1.2.156.3088.1.0001.00001.ABC123.000000",
            REGISTRY_DEFAULT_SALT
        )
        .unwrap()
        .as_str(),
        "0M9Y"
    );
}

/// 校验码对空白/大小写不敏感
#[test]
fn test_checksum_normalizes_input() {
    assert_eq!(
        checksum_for(
            " 1.2.156.3088.1.0001.00001.abc123.000001 ",
            REGISTRY_DEFAULT_SALT
        )
        .unwrap()
        .as_str(),
        "176F"
    );
}

// ---- 解析与格式校验 ----

/// 规范示例解析成功（非默认前缀需显式指定）
#[test]
fn test_parse_spec_example_with_explicit_prefix() {
    assert!(Aic::parse(SPEC_EXAMPLE).is_err(), "默认前缀下应拒绝");
    let aic = Aic::parse_with_prefix(SPEC_EXAMPLE, "1.2.156.1234").unwrap();
    assert_eq!(aic.oid_prefix(), "1.2.156.1234");
    assert_eq!(aic.version(), "1");
    assert_eq!(aic.arsp(), "1");
    assert_eq!(aic.vendor(), "34C2");
    assert_eq!(aic.ontology_serial(), "478BDF");
    assert_eq!(aic.instance_serial(), "3GF546");
    assert_eq!(aic.checksum(), "0H9T");
    assert!(aic.is_entity());
    assert_eq!(aic.to_string(), SPEC_EXAMPLE);
}

/// 默认前缀下的解析与各级访问
#[test]
fn test_parse_entity_and_ontology() {
    let entity = Aic::parse(ENTITY_AIC).unwrap();
    assert_eq!(entity.oid_prefix(), AIC_OID_PREFIX);
    assert_eq!(entity.as_str(), ENTITY_AIC);
    assert!(entity.is_entity());
    assert!(!entity.is_ontology());
    assert_eq!(entity.ontology_prefix(), "1.2.156.3088.1.0001.00001.ABC123");
    assert_eq!(
        entity.body().unwrap().as_str(),
        "1.2.156.3088.1.0001.00001.ABC123.000001"
    );

    let ontology = Aic::parse(ONTOLOGY_AIC).unwrap();
    assert!(ontology.is_ontology());
    assert!(!ontology.is_entity());
}

/// 规范化：大小写与空白都会被接受并归一
#[test]
fn test_parse_normalizes_case_and_whitespace() {
    let aic = Aic::parse(" 1.2.156.3088.1.0001.00001.abc123.000001.176f ").unwrap();
    assert_eq!(aic.as_str(), ENTITY_AIC);
    assert_eq!(aic, Aic::parse(ENTITY_AIC).unwrap());
}

/// 段数必须为 10
#[test]
fn test_parse_rejects_wrong_segment_count() {
    for bad in [
        "1.2.156.3088.1.K9",
        "1.2.156.3088.1.K9.ABCDEF.123456789.000000000",
        "1.2.156.3088.1.K9.ABCDEF.123456789.000000000.ABCD.EXTRA",
    ] {
        assert!(
            matches!(Aic::parse(bad), Err(AipError::InvalidAic)),
            "{bad}"
        );
    }
}

/// 空段与空串
#[test]
fn test_parse_rejects_empty_segments() {
    assert!(matches!(Aic::parse(""), Err(AipError::InvalidAic)));
    assert!(matches!(
        Aic::parse("1..156.3088.1.K9.ABCDEF.123456789.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
    assert!(matches!(
        Aic::parse("1.2.156.3088.1.K9.ABCDEF.123456789.000000000."),
        Err(AipError::InvalidAic)
    ));
}

/// 前缀必须匹配
#[test]
fn test_parse_rejects_wrong_prefix() {
    assert!(matches!(
        Aic::parse("1.2.156.3089.1.K9.ABCDEF.123456789.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
}

/// 1~4 级必须为纯数字
#[test]
fn test_parse_rejects_non_numeric_prefix() {
    assert!(matches!(
        Aic::parse("1.2.15X.3088.1.K9.ABCDEF.123456789.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
}

/// 5~10 级必须是 Base36 且满足长度约束
#[test]
fn test_parse_rejects_bad_base36_and_lengths() {
    // 第 5 级（版本）只能是 1 位
    assert!(matches!(
        Aic::parse("1.2.156.3088.10.K9.ABCDEF.123456789.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
    // 第 6 级（ARSP）最多 6 位
    assert!(matches!(
        Aic::parse("1.2.156.3088.1.1234567.ABCDEF.123456789.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
    // 第 8 级（本体序列号）最多 9 位
    assert!(matches!(
        Aic::parse("1.2.156.3088.1.K9.ABCDEF.1234567890.000000000.ABCD"),
        Err(AipError::InvalidAic)
    ));
    // 第 10 级（校验码）必须恰好 4 位
    assert!(matches!(
        Aic::parse("1.2.156.3088.1.K9.ABCDEF.123456789.000000000.ABC"),
        Err(AipError::InvalidAic)
    ));
    // 非 Base36 字符
    assert!(matches!(
        Aic::parse("1.2.156.3088.1.K9.ABCDEF.123456789.000000000.AB-D"),
        Err(AipError::InvalidAic)
    ));
    // 小写不是错误：规范化会转为大写（与规范一致）
    assert_eq!(
        Aic::parse("1.2.156.3088.1.K9.ABCDEF.123456789.000000000.abcd")
            .unwrap()
            .as_str(),
        "1.2.156.3088.1.K9.ABCDEF.123456789.000000000.ABCD"
    );
}

/// 段数常量与结构一致
#[test]
fn test_segment_count_constant() {
    assert_eq!(AIC_SEGMENTS, 10);
    assert_eq!(Aic::parse(ENTITY_AIC).unwrap().segment(1).unwrap(), "1");
    assert_eq!(Aic::parse(ENTITY_AIC).unwrap().segment(10).unwrap(), "176F");
    assert!(Aic::parse(ENTITY_AIC).unwrap().segment(0).is_none());
    assert!(Aic::parse(ENTITY_AIC).unwrap().segment(11).is_none());
}

// ---- 校验码验证 ----

/// 校验码验证（正确 SALT 通过、错误 SALT 失败）
#[test]
fn test_verify_checksum() {
    let entity = Aic::parse(ENTITY_AIC).unwrap();
    assert!(entity.verify_checksum(REGISTRY_DEFAULT_SALT));
    assert!(!entity.verify_checksum(b"wrong-salt"));

    let ontology = Aic::parse(ONTOLOGY_AIC).unwrap();
    assert!(ontology.verify_checksum(REGISTRY_DEFAULT_SALT));

    // 校验码被篡改后应失败
    let tampered = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.0000").unwrap();
    assert!(!tampered.verify_checksum(REGISTRY_DEFAULT_SALT));
}

// ---- 其他 trait ----

/// FromStr / TryFrom
#[test]
fn test_from_str_and_try_from() {
    use core::str::FromStr;
    assert_eq!(
        Aic::from_str(ENTITY_AIC).unwrap(),
        Aic::parse(ENTITY_AIC).unwrap()
    );
    assert_eq!(
        Aic::try_from(ENTITY_AIC).unwrap(),
        Aic::parse(ENTITY_AIC).unwrap()
    );
    assert!(Aic::from_str("bogus").is_err());
}

/// JSON 往返：序列化为纯字符串，反序列化时校验
#[test]
fn test_json_roundtrip_and_validation() {
    let aic = Aic::parse(ENTITY_AIC).unwrap();
    let json = serde_json::to_string(&aic).unwrap();
    assert_eq!(json, format!("\"{ENTITY_AIC}\""));

    let back: Aic = serde_json::from_str(&json).unwrap();
    assert_eq!(back, aic);

    // 非法 AIC 必须被拒绝
    assert!(serde_json::from_str::<Aic>("\"not-an-aic\"").is_err());
    assert!(
        serde_json::from_str::<Aic>("\"1.2.156.3089.1.K9.ABCDEF.123456789.000000000.ABCD\"")
            .is_err()
    );
}

/// Debug 输出包含身份码内容
#[test]
fn test_debug_and_display() {
    let aic = Aic::parse(ENTITY_AIC).unwrap();
    assert_eq!(format!("{aic}"), ENTITY_AIC);
    assert!(format!("{aic:?}").contains(ENTITY_AIC));
}
