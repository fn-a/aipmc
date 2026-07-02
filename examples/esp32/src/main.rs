//! AIP 智能体互联协议 ESP-32 示例
//!
//! 演示如何在 ESP-32 上创建一个温度传感器智能体节点，
//! 包含 DHT22 温度读取技能和心跳保活功能。
//!
//! ## 运行方式
//!
//! ```bash
//! cargo build --target xtensa-esp32-none-elf --bin aipmc-esp32-example
//! espflash flash target/xtensa-esp32-none-elf/debug/aipmc-esp32-example
//! ```

#![no_std]
#![no_main]

use aipmc::basic::agent::{AgentCard, AgentCapability};
use aipmc::basic::protocol::AcpMethod;
use aipmc::basic::skill::AgentSkill;
use aipmc::basic::AcpMessage;

/// 入口函数，构建智能体卡片后进入主循环
fn main_entry() -> ! {
    // 1. 构建 AgentCard：描述智能体身份、能力与技能
    let agent = AgentCard::new("esp32-sensor-01", "ESP32-Sensor", "ESP-32 sensor agent with temperature and humidity")
        .unwrap()
        .with_capability(AgentCapability::SensorRead)
        .unwrap()
        .with_skill(
            AgentSkill::new("temp-read", "ReadTemperature", "Read current temperature from DHT22 sensor")
                .unwrap()
                .with_schemas(
                    r#"{"type":"object","properties":{"unit":{"type":"string","enum":["C","F"]}}}"#,
                    r#"{"type":"object","properties":{"value":{"type":"number"},"unit":{"type":"string"}}"#,
                )
                .unwrap(),
        )
        .unwrap()
        .with_endpoint("aip://esp32-sensor-01.local")
        .unwrap();

    // 2. 构建心跳消息
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 0);

    // 3. 主循环：周期性发送心跳，处理入站消息
    loop {
        let _ = &agent;
        let _ = &msg;
        // TODO: 调用 transport.recv() 处理入站请求
        // TODO: 调用 transport.send() 发送心跳
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
