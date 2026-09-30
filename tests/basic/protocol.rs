//! 帧头、方法码、状态码与协议常量测试

use aipmc::basic::{
    ACP_HEADER_LEN, ACP_MAGIC, ACP_MAX_FRAME_LEN, ACP_MAX_PAYLOAD_LEN, ACP_MAX_PAYLOAD_LEN_WIRE,
    ACP_MIN_PAYLOAD_LEN, ACP_SESSION_ID_LEN, ACP_VERSION, AcpHeader, AcpMethod, AcpProtocol,
    AcpStatusCode, AipError,
};

/// 协议常量之间的关系必须自洽，否则帧长计算会出现不一致
#[test]
fn test_protocol_constants_are_consistent() {
    assert_eq!(ACP_MAGIC, 0xA1A2);
    assert_eq!(ACP_VERSION, 1);
    assert_eq!(ACP_HEADER_LEN, 8);
    assert_eq!(ACP_SESSION_ID_LEN, 4);
    assert_eq!(ACP_MIN_PAYLOAD_LEN as usize, ACP_SESSION_ID_LEN);
    assert_eq!(
        ACP_MAX_PAYLOAD_LEN_WIRE as usize,
        ACP_SESSION_ID_LEN + ACP_MAX_PAYLOAD_LEN
    );
    assert_eq!(
        ACP_MAX_FRAME_LEN,
        ACP_HEADER_LEN + ACP_SESSION_ID_LEN + ACP_MAX_PAYLOAD_LEN
    );
}

/// 帧头序列化 / 反序列化往返
#[test]
fn test_header_roundtrip() {
    let header = AcpHeader::new(AcpMethod::Initialize, 100);
    let bytes = header.to_bytes();
    let decoded = AcpHeader::from_bytes(&bytes).unwrap();
    assert_eq!(header, decoded);
    assert_eq!(header.method, decoded.method);
    assert_eq!(header.payload_len, decoded.payload_len);
    assert_eq!(header.version, decoded.version);
    assert_eq!(header.magic, decoded.magic);
}

/// 帧头长度/宽度字段辅助方法
#[test]
fn test_header_len_helpers() {
    let header = AcpHeader::new(AcpMethod::Heartbeat, 4 + 10);
    assert_eq!(header.frame_len(), ACP_HEADER_LEN + 14);
    assert_eq!(header.body_len(), 10);

    // 即使 payload_len 被构造成非法值也不会 panic（饱和减法）
    let mut bogus = AcpHeader::new(AcpMethod::Heartbeat, 4);
    bogus.payload_len = 0;
    assert_eq!(bogus.body_len(), 0);
}

/// try_new 拒绝越界的 payload_len
#[test]
fn test_header_try_new_validates_length() {
    assert!(AcpHeader::try_new(AcpMethod::Heartbeat, ACP_MIN_PAYLOAD_LEN).is_ok());
    assert!(AcpHeader::try_new(AcpMethod::Heartbeat, ACP_MAX_PAYLOAD_LEN_WIRE).is_ok());
    assert!(matches!(
        AcpHeader::try_new(AcpMethod::Heartbeat, 0),
        Err(AipError::InvalidPayload)
    ));
    assert!(matches!(
        AcpHeader::try_new(AcpMethod::Heartbeat, 3),
        Err(AipError::InvalidPayload)
    ));
    assert!(matches!(
        AcpHeader::try_new(AcpMethod::Heartbeat, ACP_MAX_PAYLOAD_LEN_WIRE + 1),
        Err(AipError::InvalidPayload)
    ));
}

// ---- 帧头非法输入 ----

/// 构造原始帧头字节，便于测试各种畸形输入
fn raw_header(version: u8, method: u8, flags: u8, payload_len: u16, checksum: u8) -> [u8; 8] {
    let mut buf = [0u8; ACP_HEADER_LEN];
    buf[0] = (ACP_MAGIC >> 8) as u8;
    buf[1] = (ACP_MAGIC & 0xFF) as u8;
    buf[2] = version;
    buf[3] = method;
    buf[4] = flags;
    buf[5] = (payload_len >> 8) as u8;
    buf[6] = (payload_len & 0xFF) as u8;
    buf[7] = checksum;
    buf
}

/// Magic 不匹配必须被拒绝
#[test]
fn test_header_rejects_bad_magic() {
    let mut buf = raw_header(ACP_VERSION, AcpMethod::Heartbeat.code(), 0, 4, 0);
    buf[0] = 0x00;
    buf[1] = 0x00;
    assert!(matches!(
        AcpHeader::from_bytes(&buf),
        Err(AipError::InvalidHeader)
    ));
}

/// 不支持的协议版本必须被拒绝
#[test]
fn test_header_rejects_unsupported_version() {
    for version in [0u8, 2, 0x7F, 0xFF] {
        let buf = raw_header(version, AcpMethod::Heartbeat.code(), 0, 4, 0);
        assert!(
            matches!(
                AcpHeader::from_bytes(&buf),
                Err(AipError::UnsupportedVersion)
            ),
            "version={version} 应被拒绝"
        );
    }
}

/// payload_len 越界须被拒绝
#[test]
fn test_header_rejects_out_of_range_payload_len() {
    for len in [0u16, 1, 2, 3, ACP_MAX_PAYLOAD_LEN_WIRE + 1, u16::MAX] {
        let buf = raw_header(ACP_VERSION, AcpMethod::Heartbeat.code(), 0, len, 0);
        assert!(
            matches!(AcpHeader::from_bytes(&buf), Err(AipError::InvalidPayload)),
            "payload_len={len} 应被拒绝"
        );
    }
}

/// 未知方法码必须被拒绝
#[test]
fn test_header_rejects_unknown_method() {
    let buf = raw_header(ACP_VERSION, 0x7F, 0, 4, 0);
    assert!(matches!(
        AcpHeader::from_bytes(&buf),
        Err(AipError::UnsupportedMethod)
    ));
}

// ---- 标志位 ----

/// FIN / COMP / ENC 标志位的设置与读取
#[test]
fn test_header_flag_accessors() {
    let mut header = AcpHeader::new(AcpMethod::Stream, 4);
    assert!(!header.is_final());
    assert!(!header.is_compressed());
    assert!(!header.is_encrypted());
    assert!(!header.has_reserved_bits());

    header.set_final(true);
    header.set_compressed(true);
    header.set_encrypted(true);
    assert!(header.is_final());
    assert!(header.is_compressed());
    assert!(header.is_encrypted());
    assert_eq!(header.flags, 0b0000_0111);

    header.set_final(false);
    assert!(!header.is_final());
    assert_eq!(header.flags, 0b0000_0110);

    // 保留位必须能被识别出来（协议要求发送方置 0、接收方忽略）
    header.flags |= 0b1000_0000;
    assert!(header.has_reserved_bits());
}

// ---- 方法码 ----

/// 全部方法码的 TryFrom 映射，包含 MCU 扩展的 CapabilitySpec / HeartbeatAck
#[test]
fn test_method_try_from() {
    let cases: &[(u8, AcpMethod)] = &[
        (0x01, AcpMethod::Initialize),
        (0x02, AcpMethod::Initialized),
        (0x03, AcpMethod::Stream),
        (0x04, AcpMethod::TaskCancel),
        (0x05, AcpMethod::ClearContext),
        (0x06, AcpMethod::Authorize),
        (0x07, AcpMethod::Deauthorize),
        (0x08, AcpMethod::Push),
        (0x10, AcpMethod::Discover),
        (0x11, AcpMethod::CapabilitySpec),
        (0x20, AcpMethod::Heartbeat),
        (0x21, AcpMethod::HeartbeatAck),
    ];
    for &(code, method) in cases {
        assert_eq!(
            AcpMethod::try_from(code).unwrap(),
            method,
            "code={code:#04x}"
        );
        assert_eq!(method.code(), code);
    }
    assert!(matches!(
        AcpMethod::try_from(0xFF),
        Err(AipError::UnsupportedMethod)
    ));
}

/// 标准方法与 MCU 扩展方法的划分，以及响应方法码映射
#[test]
fn test_method_classification_and_response() {
    assert!(AcpMethod::Initialize.is_standard());
    assert!(!AcpMethod::Initialize.is_mcu_extension());
    assert!(AcpMethod::Discover.is_mcu_extension());
    assert!(AcpMethod::HeartbeatAck.is_mcu_extension());

    assert_eq!(
        AcpMethod::Initialize.response_method(),
        AcpMethod::Initialized
    );
    assert_eq!(
        AcpMethod::Discover.response_method(),
        AcpMethod::CapabilitySpec
    );
    assert_eq!(
        AcpMethod::Heartbeat.response_method(),
        AcpMethod::HeartbeatAck
    );
    // 其余方法保持不变
    assert_eq!(AcpMethod::Stream.response_method(), AcpMethod::Stream);
    assert_eq!(
        AcpMethod::TaskCancel.response_method(),
        AcpMethod::TaskCancel
    );
}

/// 方法码与 ACPs 规范的对应关系必须与文档一致
#[test]
fn test_method_acps_binding() {
    use aipmc::basic::AcpsBinding;

    assert_eq!(
        AcpMethod::Stream.acps_binding(),
        AcpsBinding::Method("stream")
    );
    assert_eq!(
        AcpMethod::TaskCancel.acps_binding(),
        AcpsBinding::TaskCommand("cancel")
    );
    assert_eq!(
        AcpMethod::Push.acps_binding(),
        AcpsBinding::Method("notification/set")
    );
    assert_eq!(
        AcpMethod::Discover.acps_binding(),
        AcpsBinding::Document("/discover")
    );
    assert_eq!(
        AcpMethod::CapabilitySpec.acps_binding(),
        AcpsBinding::Document("AgentCapabilitySpec")
    );

    // 规范中没有对应概念的方法必须明确标记为 MCU 扩展
    for method in [
        AcpMethod::Initialize,
        AcpMethod::Initialized,
        AcpMethod::ClearContext,
        AcpMethod::Authorize,
        AcpMethod::Deauthorize,
        AcpMethod::Heartbeat,
        AcpMethod::HeartbeatAck,
    ] {
        assert_eq!(
            method.acps_binding(),
            AcpsBinding::McuExtension,
            "{method:?} 应标记为 MCU 扩展"
        );
    }

    // 每个方法码都必须有绑定（穷尽性由 match 保证，这里做一次抽样断言）
    let codes = [
        0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x10, 0x11, 0x20, 0x21,
    ];
    let standard_like = codes
        .iter()
        .filter(|&&c| AcpMethod::try_from(c).unwrap().acps_binding() != AcpsBinding::McuExtension)
        .count();
    assert_eq!(standard_like, 5, "当前有 5 个方法码对应规范概念");
}

// ---- 状态码 ----

/// 全部状态码的 TryFrom 映射
#[test]
fn test_status_code_try_from() {
    let cases: &[(u8, AcpStatusCode)] = &[
        (0x00, AcpStatusCode::Ok),
        (0x01, AcpStatusCode::Accepted),
        (0x02, AcpStatusCode::PartialContent),
        (0x40, AcpStatusCode::BadRequest),
        (0x41, AcpStatusCode::Unauthorized),
        (0x44, AcpStatusCode::NotFound),
        (0x45, AcpStatusCode::MethodNotSupported),
        (0x4E, AcpStatusCode::SessionExpired),
        (0x50, AcpStatusCode::InternalError),
        (0x53, AcpStatusCode::ServiceUnavailable),
    ];
    for &(code, status) in cases {
        assert_eq!(
            AcpStatusCode::try_from(code).unwrap(),
            status,
            "code={code:#04x}"
        );
        assert_eq!(status.code(), code);
    }
    assert!(matches!(
        AcpStatusCode::try_from(0xFF),
        Err(AipError::UnsupportedStatusCode)
    ));
}

/// 状态码分类辅助（成功 / 客户端错误 / 服务端错误）
#[test]
fn test_status_code_classification() {
    assert!(AcpStatusCode::Ok.is_success());
    assert!(AcpStatusCode::PartialContent.is_success());
    assert!(!AcpStatusCode::BadRequest.is_success());

    assert!(AcpStatusCode::BadRequest.is_client_error());
    assert!(AcpStatusCode::SessionExpired.is_client_error());
    assert!(!AcpStatusCode::Ok.is_client_error());

    assert!(AcpStatusCode::InternalError.is_server_error());
    assert!(AcpStatusCode::ServiceUnavailable.is_server_error());
    assert!(!AcpStatusCode::BadRequest.is_server_error());
}

// ---- 协议元数据 ----

/// 协议版本查询与兼容性判断
#[test]
fn test_protocol_version() {
    assert_eq!(AcpProtocol::version(), ACP_VERSION);
    assert!(AcpProtocol::is_compatible(ACP_VERSION));
    assert!(!AcpProtocol::is_compatible(ACP_VERSION + 1));
}
