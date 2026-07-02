//! AIPMC 集成测试
//!
//! 包含协议编解码、智能体卡片构建、消息校验等端到端测试用例，
//! 通过 `aipmc` 根 crate 统一调用，验证各子模块的协作正确性。

use aipmc::basic::{
    AcpCodec, AcpMessage, AcpMethod, AcpProtocol, AcpStatusCode,
    AgentCard, AgentCapability, AgentSkill, AipError, protocol::AcpHeader
};

/// 测试：帧头序列化与反序列化往返
///
/// 验证 AcpHeader 的 to_bytes / from_bytes 双向转换正确性，
/// 确保 Magic、Version、Method、PayloadLen 字段无损还原。
#[test]
fn test_header_roundtrip() {
    let header = AcpHeader::new(AcpMethod::Initialize, 100);
    let bytes = header.to_bytes();
    let decoded = AcpHeader::from_bytes(&bytes).unwrap();
    assert_eq!(header.method, decoded.method);
    assert_eq!(header.payload_len, decoded.payload_len);
    assert_eq!(header.version, decoded.version);
    assert_eq!(header.magic, decoded.magic);
}

/// 测试：消息序列化与反序列化往返
///
/// 验证 AcpMessage 的 encode / decode 双向转换正确性，
/// 确保方法码、会话ID、负载内容无损还原。
#[test]
fn test_message_roundtrip() {
    let payload = b"hello aipmc";
    let msg = AcpMessage::with_payload(AcpMethod::MessageStream, 42, payload).unwrap();
    let encoded = msg.encode().unwrap();
    let decoded = AcpMessage::decode(&encoded).unwrap();
    assert_eq!(decoded.header.method, AcpMethod::MessageStream);
    assert_eq!(decoded.session_id, 42);
    assert_eq!(&decoded.payload[..], payload);
}

/// 测试：消息校验和验证
///
/// 验证校验和计算与验证机制正确工作，
/// 确保传输中数据损坏可被检测。
#[test]
fn test_checksum_validation() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    assert!(msg.validate_checksum());
}

/// 测试：校验和损坏检测
///
/// 篡改编码后的消息字节，验证校验和校验能够检测到数据损坏。
#[test]
fn test_checksum_tamper_detection() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    let mut encoded = msg.encode().unwrap();
    // 篡改最后一个 payload 字节
    let last = encoded.len() - 1;
    encoded[last] ^= 0xFF;
    let decoded = AcpMessage::decode(&encoded).unwrap();
    assert!(!decoded.validate_checksum());
}

/// 测试：智能体卡片构建
///
/// 验证 AgentCard 的 Builder 模式构建正确性，
/// 确保各字段（agent_id、capabilities、endpoint）正确设置。
#[test]
fn test_agent_card_build() {
    let card = AgentCard::new("dev-01", "TestAgent", "A test agent")
        .unwrap()
        .with_capability(AgentCapability::SensorRead)
        .unwrap()
        .with_endpoint("aip://dev-01.local")
        .unwrap();
    assert_eq!(card.agent_id.as_str(), "dev-01");
    assert_eq!(card.name.as_str(), "TestAgent");
    assert_eq!(card.capabilities.len(), 1);
    assert_eq!(card.endpoint.as_str(), "aip://dev-01.local");
    assert_eq!(card.version.as_str(), "0.1.0");
}

/// 测试：智能体卡片自定义版本号
#[test]
fn test_agent_card_version() {
    let card = AgentCard::new("dev-02", "VersionedAgent", "Versioned")
        .unwrap()
        .with_version("2.0.0")
        .unwrap();
    assert_eq!(card.version.as_str(), "2.0.0");
}

/// 测试：智能体技能构建
///
/// 验证 AgentSkill 的 Builder 模式构建正确性，
/// 确保 id、tags、schemas 字段正确设置。
#[test]
fn test_agent_skill_build() {
    let skill = AgentSkill::new("temp", "ReadTemp", "Read temperature")
        .unwrap()
        .with_tag("sensor")
        .unwrap()
        .with_tag("dht22")
        .unwrap()
        .with_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
        .unwrap();
    assert_eq!(skill.id.as_str(), "temp");
    assert_eq!(skill.name.as_str(), "ReadTemp");
    assert_eq!(skill.tags.len(), 2);
    assert_eq!(skill.tags[0].as_str(), "sensor");
    assert_eq!(skill.tags[1].as_str(), "dht22");
}

/// 测试：方法码转换
///
/// 验证 AcpMethod 的 TryFrom<u8> 实现正确性，
/// 包括合法方法码和非法方法码。
#[test]
fn test_method_try_from() {
    assert_eq!(AcpMethod::try_from(0x01).unwrap(), AcpMethod::Initialize);
    assert_eq!(AcpMethod::try_from(0x02).unwrap(), AcpMethod::Initialized);
    assert_eq!(AcpMethod::try_from(0x03).unwrap(), AcpMethod::MessageStream);
    assert_eq!(AcpMethod::try_from(0x04).unwrap(), AcpMethod::TaskCancel);
    assert_eq!(AcpMethod::try_from(0x05).unwrap(), AcpMethod::ClearContext);
    assert_eq!(AcpMethod::try_from(0x06).unwrap(), AcpMethod::Authorize);
    assert_eq!(AcpMethod::try_from(0x07).unwrap(), AcpMethod::Deauthorize);
    assert_eq!(AcpMethod::try_from(0x08).unwrap(), AcpMethod::Push);
    assert_eq!(AcpMethod::try_from(0x10).unwrap(), AcpMethod::Discover);
    assert_eq!(AcpMethod::try_from(0x20).unwrap(), AcpMethod::Heartbeat);
    assert!(matches!(AcpMethod::try_from(0xFF), Err(AipError::UnsupportedMethod)));
}

/// 测试：状态码转换
#[test]
fn test_status_code_try_from() {
    assert_eq!(AcpStatusCode::try_from(0x00).unwrap(), AcpStatusCode::Ok);
    assert_eq!(AcpStatusCode::try_from(0x01).unwrap(), AcpStatusCode::Accepted);
    assert_eq!(AcpStatusCode::try_from(0x02).unwrap(), AcpStatusCode::PartialContent);
    assert_eq!(AcpStatusCode::try_from(0x40).unwrap(), AcpStatusCode::BadRequest);
    assert_eq!(AcpStatusCode::try_from(0x41).unwrap(), AcpStatusCode::Unauthorized);
    assert_eq!(AcpStatusCode::try_from(0x44).unwrap(), AcpStatusCode::NotFound);
    assert_eq!(AcpStatusCode::try_from(0x45).unwrap(), AcpStatusCode::MethodNotSupported);
    assert_eq!(AcpStatusCode::try_from(0x50).unwrap(), AcpStatusCode::InternalError);
    assert_eq!(AcpStatusCode::try_from(0x53).unwrap(), AcpStatusCode::ServiceUnavailable);
    assert_eq!(AcpStatusCode::try_from(0x4E).unwrap(), AcpStatusCode::SessionExpired);
    assert!(matches!(AcpStatusCode::try_from(0xFF), Err(AipError::InvalidMessage)));
}

/// 测试：AcpCodec 编解码
#[test]
fn test_codec_roundtrip() {
    let msg = AcpMessage::with_payload(AcpMethod::Initialize, 123, b"init-request").unwrap();
    let encoded = AcpCodec::encode(&msg).unwrap();
    let decoded = AcpCodec::decode(&encoded).unwrap();
    assert_eq!(decoded.header.method, AcpMethod::Initialize);
    assert_eq!(decoded.session_id, 123);
    assert_eq!(&decoded.payload[..], b"init-request");
}

/// 测试：空负载消息
#[test]
fn test_empty_payload_message() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 999);
    assert_eq!(msg.header.method, AcpMethod::Heartbeat);
    assert_eq!(msg.session_id, 999);
    assert!(msg.payload.is_empty());
}

/// 测试：消息响应构建
///
/// 验证 response() 方法正确构建响应消息，
/// 特别是 Initialize -> Initialized 的方法码映射。
#[test]
fn test_response_build() {
    let request = AcpMessage::with_payload(AcpMethod::Initialize, 0, b"init").unwrap();
    let response = request.response(AcpStatusCode::Ok, b"ok").unwrap();
    assert_eq!(response.header.method, AcpMethod::Initialized);
    assert_eq!(response.session_id, 0);
    // 响应负载首字节为状态码
    assert_eq!(response.payload[0], AcpStatusCode::Ok as u8);
}

/// 测试：无效帧头 Magic 检测
#[test]
fn test_invalid_header_magic() {
    let mut buf = [0u8; 8];
    buf[0] = 0x00; // 错误的 Magic
    buf[1] = 0x00;
    let result = AcpHeader::from_bytes(&buf);
    assert!(matches!(result, Err(AipError::InvalidHeader)));
}

/// 测试：消息过短检测
#[test]
fn test_message_too_short() {
    let result = AcpMessage::decode(&[0u8; 4]);
    assert!(matches!(result, Err(AipError::InvalidMessage)));
}

/// 测试：协议版本号查询
#[test]
fn test_protocol_version() {
    assert_eq!(AcpProtocol::version(), 1);
}

/// 测试：AgentCard 带技能构建
#[test]
fn test_agent_card_with_skills() {
    let skill1 = AgentSkill::new("temp-read", "ReadTemp", "Read temperature").unwrap();
    let skill2 = AgentSkill::new("humi-read", "ReadHumidity", "Read humidity").unwrap();
    let card = AgentCard::new("multi-sensor", "MultiSensor", "Temperature and humidity")
        .unwrap()
        .with_capability(AgentCapability::SensorRead)
        .unwrap()
        .with_skill(skill1)
        .unwrap()
        .with_skill(skill2)
        .unwrap();
    assert_eq!(card.skills.len(), 2);
    assert_eq!(card.skills[0].id.as_str(), "temp-read");
    assert_eq!(card.skills[1].id.as_str(), "humi-read");
}

/// 测试：AgentSkill 容量溢出
#[test]
fn test_skill_tag_overflow() {
    let mut skill = AgentSkill::new("test", "Test", "Test skill").unwrap();
    // 最多 4 个 tag
    for i in 0..4 {
        skill = skill.with_tag(&format!("tag{}", i)).unwrap();
    }
    // 第 5 个应该溢出
    let result = skill.with_tag("overflow");
    assert!(matches!(result, Err(AipError::BufferOverflow)));
}
