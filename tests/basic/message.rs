//! 消息编解码测试

use aipmc::basic::{
    ACP_HEADER_LEN, ACP_MAGIC, ACP_MAX_FRAME_LEN, ACP_MAX_PAYLOAD_LEN, ACP_SESSION_ID_LEN,
    ACP_VERSION, AcpCodec, AcpHeader, AcpMessage, AcpMethod, AcpStatusCode, AipError,
};

/// 构造一个原始帧字节流，用于覆盖各种畸形输入
fn raw_frame(method: u8, flags: u8, payload_len: u16, body: &[u8]) -> Vec<u8> {
    let mut buf = vec![
        (ACP_MAGIC >> 8) as u8,
        (ACP_MAGIC & 0xFF) as u8,
        ACP_VERSION,
        method,
        flags,
        (payload_len >> 8) as u8,
        (payload_len & 0xFF) as u8,
        0, // checksum 占位
    ];
    buf.extend_from_slice(body);
    buf
}

// ---- 基本往返 ----

/// 带负载消息的编码 / 解码往返
#[test]
fn test_message_roundtrip() {
    let payload = b"hello aipmc";
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 42, payload).unwrap();
    assert_eq!(
        msg.frame_len(),
        ACP_HEADER_LEN + ACP_SESSION_ID_LEN + payload.len()
    );
    assert_eq!(
        msg.header.payload_len as usize,
        ACP_SESSION_ID_LEN + payload.len()
    );

    let encoded = msg.encode().unwrap();
    assert_eq!(encoded.len(), msg.frame_len());

    let decoded = AcpMessage::decode(&encoded).unwrap();
    assert_eq!(decoded.header.method, AcpMethod::Stream);
    assert_eq!(decoded.session_id, 42);
    assert_eq!(&decoded.payload[..], payload);
    assert!(decoded.validate_checksum());
}

/// 回归：空负载消息必须能自洽往返
#[test]
fn test_empty_payload_message_roundtrip() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 999);
    assert_eq!(msg.payload.len(), 0);
    assert_eq!(msg.header.payload_len as usize, ACP_SESSION_ID_LEN);
    assert_eq!(msg.frame_len(), ACP_HEADER_LEN + ACP_SESSION_ID_LEN);

    let encoded = msg.encode().unwrap();
    assert_eq!(encoded.len(), ACP_HEADER_LEN + ACP_SESSION_ID_LEN);

    let decoded = AcpMessage::decode(&encoded).unwrap();
    assert_eq!(decoded.header.method, AcpMethod::Heartbeat);
    assert_eq!(decoded.session_id, 999);
    assert!(decoded.payload.is_empty());
}

/// 编码结果与 header 字段一致性：即使直接改坏 header，编码仍以实际负载为准
#[test]
fn test_encode_is_self_consistent_after_header_tampering() {
    let mut msg = AcpMessage::with_payload(AcpMethod::Initialize, 7, b"abc").unwrap();
    msg.header.payload_len = 0; // 故意写坏
    msg.header.version = 0xEE;
    let encoded = msg.encode().unwrap();
    let decoded = AcpMessage::decode(&encoded).unwrap();
    assert_eq!(&decoded.payload[..], b"abc");
    assert_eq!(decoded.session_id, 7);
}

// ---- 校验和 ----

/// 校验和验证
#[test]
fn test_checksum_validation() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    assert!(msg.validate_checksum());
}

/// 回归：篡改负载后 decode 必须返回 ChecksumMismatch
#[test]
fn test_decode_rejects_tampered_payload() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    let mut encoded = msg.encode().unwrap();
    let last = encoded.len() - 1;
    encoded[last] ^= 0xFF;

    assert!(matches!(
        AcpMessage::decode(&encoded),
        Err(AipError::ChecksumMismatch)
    ));
}

/// 篡改帧头字段同样会被校验和拦住
#[test]
fn test_decode_rejects_tampered_header() {
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 5, b"payload").unwrap();
    let mut encoded = msg.encode().unwrap();
    encoded[3] = AcpMethod::TaskCancel.code(); // 篡改方法码
    assert!(matches!(
        AcpMessage::decode(&encoded),
        Err(AipError::ChecksumMismatch)
    ));
}

/// decode_unverified 跳过校验和校验，但其余字段仍然校验
#[test]
fn test_decode_unverified_skips_checksum_only() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    let mut encoded = msg.encode().unwrap();
    let last = encoded.len() - 1;
    encoded[last] ^= 0xFF;

    let decoded = AcpMessage::decode_unverified(&encoded).unwrap();
    assert!(!decoded.validate_checksum());

    // 帧头错误仍然被拒绝
    let mut bad_magic = encoded.clone();
    bad_magic[0] = 0x00;
    assert!(matches!(
        AcpMessage::decode_unverified(&bad_magic),
        Err(AipError::InvalidHeader)
    ));
}

// ---- 畸形输入（回归：不得 panic）----

/// 回归：payload_len < 4 必须返回错误而不是 panic
#[test]
fn test_decode_does_not_panic_on_short_payload_len() {
    for payload_len in [0u16, 1, 2, 3] {
        // body 补足到 4 字节，使缓冲区长度超过最小帧长，从而走到 payload_len 校验
        let frame = raw_frame(AcpMethod::Heartbeat.code(), 0, payload_len, &[0, 0, 0, 0]);
        let result = AcpMessage::decode(&frame);
        assert!(
            matches!(result, Err(AipError::InvalidPayload)),
            "payload_len={payload_len} 应返回 InvalidPayload，实际 {result:?}"
        );
    }
}

/// 声明长度超过缓冲区实际内容时返回 InvalidPayload
#[test]
fn test_decode_rejects_truncated_body() {
    // 声明 body 长 10，但只给了 4 字节
    let frame = raw_frame(AcpMethod::Heartbeat.code(), 0, 4 + 10, &[0u8; 4]);
    assert!(matches!(
        AcpMessage::decode(&frame),
        Err(AipError::InvalidPayload)
    ));
}

/// 声明长度超过单帧上限时返回 InvalidPayload
#[test]
fn test_decode_rejects_oversize_declared_payload() {
    let frame = raw_frame(AcpMethod::Heartbeat.code(), 0, u16::MAX, &[0u8; 8]);
    assert!(matches!(
        AcpMessage::decode(&frame),
        Err(AipError::InvalidPayload)
    ));
}

/// 数据不足一个最小帧
#[test]
fn test_message_too_short() {
    for len in 0..(ACP_HEADER_LEN + ACP_SESSION_ID_LEN) {
        let buf = vec![0u8; len];
        assert!(
            matches!(AcpMessage::decode(&buf), Err(AipError::InvalidMessage)),
            "len={len}"
        );
    }
}

/// 非法 Magic
#[test]
fn test_decode_rejects_bad_magic() {
    let mut frame = raw_frame(AcpMethod::Heartbeat.code(), 0, 4, &[0, 0, 0, 1]);
    frame[0] = 0x00;
    frame[1] = 0x00;
    assert!(matches!(
        AcpMessage::decode(&frame),
        Err(AipError::InvalidHeader)
    ));
}

/// 回归：非法协议版本必须被拒绝
#[test]
fn test_decode_rejects_unsupported_version() {
    let mut frame = raw_frame(AcpMethod::Heartbeat.code(), 0, 4, &[0, 0, 0, 1]);
    frame[2] = 0x7F;
    assert!(matches!(
        AcpMessage::decode(&frame),
        Err(AipError::UnsupportedVersion)
    ));
}

/// 未知方法码
#[test]
fn test_decode_rejects_unknown_method() {
    let frame = raw_frame(0x7F, 0, 4, &[0, 0, 0, 1]);
    assert!(matches!(
        AcpMessage::decode(&frame),
        Err(AipError::UnsupportedMethod)
    ));
}

// ---- 流式分帧 ----

/// decode_frame 返回消耗的字节数，且忽略其后的数据
#[test]
fn test_decode_frame_reports_consumed_bytes() {
    let first = AcpMessage::with_payload(AcpMethod::Initialize, 1, b"aaa").unwrap();
    let second = AcpMessage::with_payload(AcpMethod::Heartbeat, 2, b"bb").unwrap();

    let mut stream = first.encode().unwrap().as_slice().to_vec();
    let first_len = stream.len();
    stream.extend_from_slice(&second.encode().unwrap());

    let (msg, consumed) = AcpMessage::decode_frame(&stream).unwrap();
    assert_eq!(consumed, first_len);
    assert_eq!(msg.session_id, 1);
    assert_eq!(&msg.payload[..], b"aaa");

    // 从剩余字节继续解码第二帧
    let (msg2, consumed2) = AcpMessage::decode_frame(&stream[consumed..]).unwrap();
    assert_eq!(consumed2, msg2.frame_len());
    assert_eq!(msg2.session_id, 2);
    assert_eq!(&msg2.payload[..], b"bb");
}

/// AcpCodec 的便捷接口与 AcpMessage 等价
#[test]
fn test_codec_roundtrip() {
    let msg = AcpMessage::with_payload(AcpMethod::Initialize, 123, b"init-request").unwrap();
    let encoded = AcpCodec::encode(&msg).unwrap();
    let decoded = AcpCodec::decode(&encoded).unwrap();
    assert_eq!(decoded.header.method, AcpMethod::Initialize);
    assert_eq!(decoded.session_id, 123);
    assert_eq!(&decoded.payload[..], b"init-request");

    let (decoded2, consumed) = AcpCodec::decode_frame(&encoded).unwrap();
    assert_eq!(consumed, encoded.len());
    assert_eq!(decoded2, decoded);

    let mut out = [0u8; ACP_MAX_FRAME_LEN];
    let n = AcpCodec::encode_to(&msg, &mut out).unwrap();
    assert_eq!(&out[..n], &encoded[..]);
}

// ---- encode_to ----

/// encode_to 写入精确长度，缓冲区不足时报 BufferOverflow
#[test]
fn test_encode_to_buffer_sizing() {
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 3, b"abcdef").unwrap();
    let expected = msg.encode().unwrap();

    let mut exact = [0u8; ACP_MAX_FRAME_LEN];
    let n = msg.encode_to(&mut exact).unwrap();
    assert_eq!(n, expected.len());
    assert_eq!(&exact[..n], &expected[..]);

    let mut too_small = [0u8; 4];
    assert!(matches!(
        msg.encode_to(&mut too_small),
        Err(AipError::BufferOverflow)
    ));
}

// ---- 容量边界 ----

/// 负载容量边界：512 字节可用，513 字节溢出
#[test]
fn test_payload_capacity_boundary() {
    let max = vec![0xABu8; ACP_MAX_PAYLOAD_LEN];
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 1, &max).unwrap();
    assert_eq!(msg.frame_len(), ACP_MAX_FRAME_LEN);
    assert_eq!(msg.encode().unwrap().len(), ACP_MAX_FRAME_LEN);
    assert_eq!(AcpMessage::decode(&msg.encode().unwrap()).unwrap(), msg);

    let over = vec![0xABu8; ACP_MAX_PAYLOAD_LEN + 1];
    assert!(matches!(
        AcpMessage::with_payload(AcpMethod::Stream, 1, &over),
        Err(AipError::BufferOverflow)
    ));
}

// ---- 响应构造与状态码 ----

/// 响应方法码映射 + 状态码回读
#[test]
fn test_response_build_and_status_readback() {
    let request = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"init").unwrap();
    let response = request.response(AcpStatusCode::Ok, b"ok").unwrap();
    assert_eq!(response.header.method, AcpMethod::Initialized);
    assert_eq!(response.session_id, 0);
    assert_eq!(response.payload[0], AcpStatusCode::Ok as u8);
    assert_eq!(response.status().unwrap(), AcpStatusCode::Ok);
    assert_eq!(&response.payload[1..], b"ok");
    assert!(response.validate_checksum());

    // 非 Initialize 方法保持方法码不变
    let hb = AcpMessage::new(AcpMethod::Heartbeat, 9);
    assert_eq!(
        hb.response(AcpStatusCode::Accepted, &[])
            .unwrap()
            .header
            .method,
        AcpMethod::HeartbeatAck
    );

    // Discover 的响应是 CapabilitySpec（承载 ACS）
    let discover = AcpMessage::new(AcpMethod::Discover, 0);
    let spec = discover.response(AcpStatusCode::Ok, b"{}").unwrap();
    assert_eq!(spec.header.method, AcpMethod::CapabilitySpec);
    assert_eq!(spec.status().unwrap(), AcpStatusCode::Ok);
}

/// 响应负载需为状态码留出 1 字节；空负载响应读取状态码报错
#[test]
fn test_response_capacity_and_empty_status() {
    let request = AcpMessage::new(AcpMethod::Initialize, 0);
    let full = vec![0u8; ACP_MAX_PAYLOAD_LEN - 1];
    assert!(request.response(AcpStatusCode::Ok, &full).is_ok());

    let over = vec![0u8; ACP_MAX_PAYLOAD_LEN];
    assert!(matches!(
        request.response(AcpStatusCode::Ok, &over),
        Err(AipError::BufferOverflow)
    ));

    assert!(matches!(
        AcpMessage::new(AcpMethod::Heartbeat, 0).status(),
        Err(AipError::InvalidPayload)
    ));
    assert!(matches!(
        AcpMessage::with_payload(AcpMethod::Heartbeat, 0, &[0xEE])
            .unwrap()
            .status(),
        Err(AipError::UnsupportedStatusCode)
    ));
}

// ---- from_parts ----

/// from_parts 校验 payload_len 与实际负载长度一致
#[test]
fn test_from_parts_validates_length() {
    let header = AcpHeader::new(AcpMethod::Heartbeat, 4 + 3);
    let msg = AcpMessage::from_parts(header.clone(), 11, b"abc").unwrap();
    assert_eq!(msg.session_id, 11);
    assert_eq!(&msg.payload[..], b"abc");

    // payload_len 与实际负载不符
    assert!(matches!(
        AcpMessage::from_parts(header, 11, b"abcd"),
        Err(AipError::InvalidPayload)
    ));

    // 负载超过 512 字节
    let big = vec![0u8; ACP_MAX_PAYLOAD_LEN + 1];
    let header_big = AcpHeader::new(AcpMethod::Heartbeat, (4 + big.len()) as u16);
    assert!(matches!(
        AcpMessage::from_parts(header_big, 1, &big),
        Err(AipError::BufferOverflow)
    ));
}

/// refresh_checksum 在直接修改字段后恢复一致性
#[test]
fn test_refresh_checksum_after_direct_mutation() {
    let mut msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"aa").unwrap();
    msg.payload[0] = b'z';
    assert!(!msg.validate_checksum());

    msg.refresh_checksum();
    assert!(msg.validate_checksum());
    assert_eq!(
        &AcpMessage::decode(&msg.encode().unwrap()).unwrap().payload[..],
        b"za"
    );
}
