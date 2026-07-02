//! AIP 智能体互联协议 STM-32 示例
//!
//! 演示如何在 STM-32 上创建一个执行器控制智能体节点，
//! 包含继电器控制技能和心跳保活功能。
//!
//! ## 运行方式
//!
//! ```bash
//! cargo build --target thumbv7em-none-eabihf --bin aipmc-stm32-example
//! probe-rs run target/thumbv7em-none-eabihf/debug/aipmc-stm32-example
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
    let agent = AgentCard::new("stm32-ctrl-01", "STM32-Controller", "STM-32 actuator control agent")
        .unwrap()
        .with_capability(AgentCapability::ActuatorControl)
        .unwrap()
        .with_skill(
            AgentSkill::new("relay-ctrl", "RelayControl", "Control relay on/off state")
                .unwrap()
                .with_schemas(
                    r#"{"type":"object","properties":{"channel":{"type":"integer"},"state":{"type":"boolean"}}"#,
                    r#"{"type":"object","properties":{"success":{"type":"boolean"}}"#,
                )
                .unwrap(),
        )
        .unwrap()
        .with_endpoint("aip://stm32-ctrl-01.uart0")
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
