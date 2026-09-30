//! ACS（智能体能力描述）与技能测试

use aipmc::basic::AipError;
use aipmc::basic::acs::{
    ACPS_PROTOCOL_VERSION, AgentCapabilities, AgentCapability, AgentCapabilitySpec, AgentEndPoint,
    AgentProvider, MqProtocolVersion, Transport,
};
use aipmc::basic::aic::{Aic, REGISTRY_DEFAULT_SALT};
use aipmc::basic::skill::{AgentSkill, MimeMode};

/// 测试用 AIC（注册中心默认 SALT 下的实体 AIC）
const TEST_AIC: &str = "1.2.156.3088.1.0001.00001.ABC123.000001.176F";

fn test_aic() -> Aic {
    Aic::parse(TEST_AIC).unwrap()
}

/// 构造一个技能
fn test_skill() -> AgentSkill {
    let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
    skill.add_tag("sensor").unwrap();
    skill.add_example(r#"{"unit":"C"}"#).unwrap();
    skill.set_modes(MimeMode::json(), MimeMode::json()).unwrap();
    skill
        .set_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
        .unwrap();
    skill
}

/// 构造一个尽量完整的 ACS
fn full_spec() -> AgentCapabilitySpec {
    let mut provider = AgentProvider::cn().unwrap();
    provider.set_organization("ACME").unwrap();
    provider.set_name("ACME Robotics").unwrap();
    provider.set_email("ops@acme.example").unwrap();

    let mut capabilities = AgentCapabilities::none();
    capabilities.set_streaming(true);
    capabilities.set_notification(false);
    capabilities
        .add_message_queue(MqProtocolVersion::new("amqp:4.2").unwrap())
        .unwrap();

    let mut spec =
        AgentCapabilitySpec::new(test_aic(), "ESP32-Sensor", "Temperature sensor agent").unwrap();
    spec.set_version("1.2.3").unwrap();
    spec.set_last_modified_time("2026-07-03T09:26:02+08:00")
        .unwrap();
    spec.set_provider(provider);
    spec.set_capabilities(capabilities);
    spec.add_skill(test_skill()).unwrap();
    spec.add_hardware_capability(AgentCapability::SensorRead)
        .unwrap();
    spec.add_end_point(
        AgentEndPoint::new("aip://gateway.local/agents/esp32-01", Transport::HttpJson).unwrap(),
    )
    .unwrap();
    spec
}

// ---- 构造与默认值 ----

/// 构造后的默认字段符合 ACS 约束
#[test]
fn test_spec_defaults() {
    let spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    assert_eq!(spec.aic.as_str(), TEST_AIC);
    assert!(spec.active);
    assert_eq!(spec.version.as_str(), "0.1.0");
    assert_eq!(spec.protocol_version.as_str(), ACPS_PROTOCOL_VERSION);
    assert_eq!(spec.protocol_version.as_str(), "02.01");
    assert_eq!(spec.provider.country_code.as_ref().unwrap().as_str(), "CN");
    assert_eq!(spec.default_input_modes.len(), 1);
    assert_eq!(spec.default_input_modes[0].as_str(), "application/json");
    assert_eq!(spec.default_output_modes.len(), 1);
    assert!(spec.end_points.is_empty());
    assert!(spec.skills.is_empty());
    assert!(spec.mcu.is_empty());
    assert!(spec.validate().is_ok(), "默认构造应通过校验");
}

/// 标量字段的设置接口（`&mut self`，不复制整个结构体）
#[test]
fn test_spec_scalar_setters() {
    let mut spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    spec.set_name("n2").unwrap();
    spec.set_description("d2").unwrap();
    spec.set_version("9.9.9").unwrap();
    spec.set_protocol_version("02.01").unwrap();
    spec.set_last_modified_time("2026-01-02T03:04:05Z").unwrap();
    spec.set_active(false);

    assert_eq!(spec.name.as_str(), "n2");
    assert_eq!(spec.description.as_str(), "d2");
    assert_eq!(spec.version.as_str(), "9.9.9");
    assert!(!spec.active);
    assert!(spec.validate().is_ok());
}

/// 字符串容量上限
#[test]
fn test_spec_capacity_limits() {
    assert!(matches!(
        AgentCapabilitySpec::new(test_aic(), &"n".repeat(65), "d"),
        Err(AipError::BufferOverflow)
    ));
    assert!(matches!(
        AgentCapabilitySpec::new(test_aic(), "n", &"d".repeat(257)),
        Err(AipError::BufferOverflow)
    ));

    let mut spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    assert!(matches!(
        spec.set_version(&"v".repeat(17)),
        Err(AipError::BufferOverflow)
    ));
    assert!(matches!(
        spec.set_name(&"n".repeat(65)),
        Err(AipError::BufferOverflow)
    ));
    assert!(matches!(
        spec.set_description(&"d".repeat(257)),
        Err(AipError::BufferOverflow)
    ));
}

/// 列表容量上限
#[test]
fn test_spec_list_capacity_limits() {
    let mut spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    for i in 0..8 {
        spec.add_skill(AgentSkill::new(&format!("s{i}"), "n", "d").unwrap())
            .unwrap();
    }
    assert!(matches!(
        spec.add_skill(AgentSkill::new("s9", "n", "d").unwrap()),
        Err(AipError::BufferOverflow)
    ));

    let mut spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    for i in 0..4 {
        spec.add_end_point(
            AgentEndPoint::new(&format!("aip://gw/{i}"), Transport::JsonRpc).unwrap(),
        )
        .unwrap();
    }
    assert!(matches!(
        spec.add_end_point(AgentEndPoint::new("aip://gw/x", Transport::JsonRpc).unwrap()),
        Err(AipError::BufferOverflow)
    ));

    let mut spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    for _ in 0..4 {
        spec.add_hardware_capability(AgentCapability::Compute)
            .unwrap();
    }
    assert!(matches!(
        spec.add_hardware_capability(AgentCapability::Storage),
        Err(AipError::BufferOverflow)
    ));
}

/// 查找技能与硬件能力
#[test]
fn test_spec_lookup_helpers() {
    let spec = full_spec();
    assert_eq!(
        spec.find_skill("temp.read").unwrap().name.as_str(),
        "ReadTemperature"
    );
    assert!(spec.find_skill("nope").is_none());
    assert!(spec.has_hardware_capability(&AgentCapability::SensorRead));
    assert!(!spec.has_hardware_capability(&AgentCapability::Storage));
}

// ---- 校验 ----

/// 必填字段为空 / 时间格式不合法时校验失败
#[test]
fn test_validate_rejects_bad_fields() {
    let spec = full_spec();
    assert!(spec.validate().is_ok());

    let mut broken = spec.clone();
    broken.name = heapless::String::new();
    assert!(matches!(broken.validate(), Err(AipError::InvalidAcs)));

    let mut broken = spec.clone();
    broken.description = heapless::String::new();
    assert!(matches!(broken.validate(), Err(AipError::InvalidAcs)));

    let mut broken = spec.clone();
    broken.protocol_version = heapless::String::new();
    assert!(matches!(broken.validate(), Err(AipError::InvalidAcs)));

    let mut broken = spec.clone();
    broken.last_modified_time = heapless::String::try_from("2026/07/03 09:26").unwrap();
    assert!(matches!(broken.validate(), Err(AipError::InvalidAcs)));

    let mut broken = spec;
    broken.default_input_modes.clear();
    assert!(matches!(broken.validate(), Err(AipError::InvalidAcs)));
}

/// 时间戳格式检查（ISO-8601 结构）
#[test]
fn test_iso8601_structural_check() {
    let ok = {
        let mut spec = full_spec();
        spec.set_last_modified_time("2026-07-03T09:26:02Z").unwrap();
        spec
    };
    assert!(ok.validate().is_ok());

    let too_short = {
        let mut spec = full_spec();
        spec.set_last_modified_time("2026-07-03").unwrap();
        spec
    };
    assert!(matches!(too_short.validate(), Err(AipError::InvalidAcs)));

    let wrong_sep = {
        let mut spec = full_spec();
        spec.set_last_modified_time("2026-07-03 09:26:02").unwrap();
        spec
    };
    assert!(matches!(wrong_sep.validate(), Err(AipError::InvalidAcs)));
}

// ---- 消息队列协议版本 ----

/// `{protocol}:{versionExpr}` 校验
#[test]
fn test_mq_protocol_version() {
    for good in ["amqp:4.2", "mqtt:5", "kafka:*", "rabbitmq:4.2", "redis:7"] {
        let v = MqProtocolVersion::new(good).unwrap();
        assert_eq!(v.as_str(), good);
        assert!(!v.protocol().is_empty());
        assert!(!v.version_expression().is_empty());
    }
    assert_eq!(
        MqProtocolVersion::new("amqp:4.2").unwrap().protocol(),
        "amqp"
    );
    assert_eq!(
        MqProtocolVersion::new("amqp:4.2")
            .unwrap()
            .version_expression(),
        "4.2"
    );

    for bad in ["amqp", "amqp:", ":4.2", "nats:1", ""] {
        assert!(
            matches!(MqProtocolVersion::new(bad), Err(AipError::InvalidAcs)),
            "{bad}"
        );
    }
    assert!(matches!(
        MqProtocolVersion::new(&"a".repeat(30)),
        Err(AipError::InvalidAcs)
    ));
}

/// MqProtocolVersion 反序列化时同样校验
#[test]
fn test_mq_protocol_version_deserialization_validates() {
    let ok: MqProtocolVersion = serde_json::from_str("\"amqp:4.2\"").unwrap();
    assert_eq!(ok.as_str(), "amqp:4.2");
    assert!(serde_json::from_str::<MqProtocolVersion>("\"nats:1\"").is_err());
    assert!(serde_json::from_str::<MqProtocolVersion>("\"amqp\"").is_err());
}

/// 能力列表容量上限
#[test]
fn test_capabilities_capacity_limit() {
    let mut caps = AgentCapabilities::none();
    caps.set_streaming(true);
    caps.set_notification(true);
    for v in ["mqtt:5", "amqp:4.2", "kafka:3", "redis:7"] {
        caps.add_message_queue(MqProtocolVersion::new(v).unwrap())
            .unwrap();
    }
    assert!(matches!(
        caps.add_message_queue(MqProtocolVersion::new("rabbitmq:4.2").unwrap()),
        Err(AipError::BufferOverflow)
    ));
}

// ---- MIME 类型 ----

/// MIME 类型构造与校验
#[test]
fn test_mime_mode() {
    assert_eq!(MimeMode::json().as_str(), "application/json");
    assert_eq!(MimeMode::text().as_str(), "text/plain");
    assert_eq!(
        MimeMode::octet_stream().as_str(),
        "application/octet-stream"
    );
    assert_eq!(
        MimeMode::new("application/cbor").unwrap().as_str(),
        "application/cbor"
    );

    for bad in ["json", "/json", "application/", ""] {
        assert!(
            matches!(MimeMode::new(bad), Err(AipError::InvalidAcs)),
            "{bad}"
        );
    }
    assert!(matches!(
        MimeMode::new(&format!("a/{}", "b".repeat(60))),
        Err(AipError::BufferOverflow)
    ));
}

// ---- 传输方式 ----

/// Transport 的规范取值与 MCU 扩展取值
#[test]
fn test_transport_variants() {
    assert!(Transport::JsonRpc.is_acps_standard());
    assert!(Transport::HttpJson.is_acps_standard());
    assert!(Transport::Amqp.is_acps_standard());
    assert!(!Transport::McuUart.is_acps_standard());
    assert!(!Transport::McuBle.is_acps_standard());

    assert_eq!(
        serde_json::to_string(&Transport::JsonRpc).unwrap(),
        "\"JSONRPC\""
    );
    assert_eq!(
        serde_json::to_string(&Transport::HttpJson).unwrap(),
        "\"HTTP_JSON\""
    );
    assert_eq!(serde_json::to_string(&Transport::Amqp).unwrap(), "\"AMQP\"");
    assert_eq!(
        serde_json::to_string(&Transport::McuUart).unwrap(),
        "\"MCU_UART\""
    );
    assert_eq!(
        serde_json::to_string(&Transport::McuWifi).unwrap(),
        "\"MCU_WIFI\""
    );
}

// ---- 技能 ----

/// 技能构建
#[test]
fn test_skill_build() {
    let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
    skill.set_version("0.2.0").unwrap();
    skill.add_tag("sensor").unwrap();
    skill.add_tag("dht22").unwrap();
    skill.add_example(r#"{"unit":"C"}"#).unwrap();
    skill.set_modes(MimeMode::json(), MimeMode::text()).unwrap();
    skill
        .set_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
        .unwrap();

    assert_eq!(skill.id.as_str(), "temp.read");
    assert_eq!(skill.version.as_str(), "0.2.0");
    assert_eq!(skill.tags.len(), 2);
    assert!(skill.has_tag("sensor"));
    assert!(!skill.has_tag("actuator"));
    assert_eq!(skill.examples.len(), 1);
    assert_eq!(skill.input_modes[0].as_str(), "application/json");
    assert_eq!(skill.output_modes[0].as_str(), "text/plain");
    assert_eq!(skill.mcu.input_schema.as_str(), r#"{"type":"number"}"#);
}

/// `set_modes` 会替换而不是追加
#[test]
fn test_skill_set_modes_replaces() {
    let mut skill = AgentSkill::new("t", "T", "d").unwrap();
    skill.set_modes(MimeMode::json(), MimeMode::json()).unwrap();
    skill.set_modes(MimeMode::text(), MimeMode::text()).unwrap();
    assert_eq!(skill.input_modes.len(), 1);
    assert_eq!(skill.output_modes.len(), 1);
    assert_eq!(skill.input_modes[0].as_str(), "text/plain");
}

/// 技能容量上限
#[test]
fn test_skill_capacity_limits() {
    let mut skill = AgentSkill::new("t", "T", "d").unwrap();
    for i in 0..4 {
        skill.add_tag(&format!("tag{i}")).unwrap();
    }
    assert!(matches!(
        skill.add_tag("overflow"),
        Err(AipError::BufferOverflow)
    ));

    let mut skill = AgentSkill::new("t", "T", "d").unwrap();
    for i in 0..2 {
        skill.add_example(&format!("e{i}")).unwrap();
    }
    assert!(matches!(
        skill.add_example("e3"),
        Err(AipError::BufferOverflow)
    ));

    let mut skill = AgentSkill::new("t", "T", "d").unwrap();
    assert!(matches!(
        skill.set_schemas(&"s".repeat(129), "{}"),
        Err(AipError::BufferOverflow)
    ));
}

// ---- JSON 序列化（camelCase + x-mcu 命名空间） ----

/// ACS JSON 字段名必须是 camelCase，且与规范同名
#[test]
fn test_spec_json_field_names() {
    let json = full_spec().to_json().unwrap();

    for expected in [
        "\"aic\":",
        "\"active\":",
        "\"lastModifiedTime\":",
        "\"protocolVersion\":",
        "\"defaultInputModes\":",
        "\"defaultOutputModes\":",
        "\"endPoints\":",
        "\"messageQueue\":",
        "\"countryCode\":",
        "\"x-mcu\":",
        "\"capabilities\":",
    ] {
        assert!(json.contains(expected), "缺少字段 {expected}：{json}");
    }
    // 不应出现 snake_case 残留
    for forbidden in [
        "\"last_modified_time\"",
        "\"protocol_version\"",
        "\"default_input_modes\"",
        "\"end_points\"",
        "\"message_queue\"",
        "\"country_code\"",
    ] {
        assert!(
            !json.contains(forbidden),
            "出现 snake_case 字段 {forbidden}"
        );
    }
}

/// 技能 JSON：ACS 字段 + x-mcu 扩展
#[test]
fn test_skill_json_shape() {
    let mut skill = AgentSkill::new("temp.read", "ReadTemperature", "Read temperature").unwrap();
    skill.set_version("0.1.0").unwrap();
    skill.add_tag("sensor").unwrap();
    skill
        .set_schemas(r#"{"type":"number"}"#, r#"{"type":"number"}"#)
        .unwrap();
    let json = skill.to_json().unwrap();
    assert!(json.contains("\"id\":\"temp.read\""));
    assert!(json.contains("\"version\":\"0.1.0\""));
    assert!(json.contains("\"tags\":[\"sensor\"]"));
    assert!(json.contains("\"x-mcu\":{"));
    assert!(json.contains("\"inputSchema\""));
    // 空的 examples / inputModes 不应出现
    assert!(!json.contains("\"examples\""));
    assert!(!json.contains("\"inputModes\""));

    // 无扩展时不应出现 x-mcu
    let plain = AgentSkill::new("a.b", "AB", "d")
        .unwrap()
        .to_json()
        .unwrap();
    assert!(!plain.contains("x-mcu"));
}

/// 无 MCU 扩展时根级 JSON 不含 x-mcu
#[test]
fn test_spec_json_omits_empty_extensions() {
    let spec = AgentCapabilitySpec::new(test_aic(), "n", "d").unwrap();
    let json = spec.to_json().unwrap();
    assert!(!json.contains("x-mcu"));
    assert!(json.contains("\"endPoints\":[]"));
    assert!(json.contains("\"skills\":[]"));
}

/// JSON 往返
#[test]
fn test_spec_json_roundtrip() {
    let spec = full_spec();
    let json = spec.to_json().unwrap();
    let parsed = AgentCapabilitySpec::from_json(&json).unwrap();
    assert_eq!(parsed, spec);
    assert!(parsed.validate().is_ok());

    let skill_json = spec.skills[0].to_json().unwrap();
    assert_eq!(AgentSkill::from_json(&skill_json).unwrap(), spec.skills[0]);
}

/// JSON 反序列化会校验 aic 与其他受约束字段
#[test]
fn test_spec_json_rejects_invalid_values() {
    let spec = full_spec();
    let json = spec.to_json().unwrap();

    // 非法 AIC
    let bad_aic = json.replace(
        TEST_AIC,
        "1.2.156.3089.1.K9.ABCDEF.123456789.000000000.ABCD",
    );
    assert!(matches!(
        AgentCapabilitySpec::from_json(&bad_aic),
        Err(AipError::CodecError)
    ));

    // 非法消息队列条目
    let bad_mq = json.replace("amqp:4.2", "nats:1");
    assert!(matches!(
        AgentCapabilitySpec::from_json(&bad_mq),
        Err(AipError::CodecError)
    ));

    // 非 JSON
    assert!(matches!(
        AgentCapabilitySpec::from_json("not json"),
        Err(AipError::CodecError)
    ));
    assert!(matches!(
        AgentCapabilitySpec::from_json("{}"),
        Err(AipError::CodecError)
    ));
}

/// 校验码可被独立复核：JSON 中的 aic 仍是可校验的身份码
#[test]
fn test_spec_aic_verifiable() {
    let spec = full_spec();
    assert!(spec.aic.verify_checksum(REGISTRY_DEFAULT_SALT));
}
