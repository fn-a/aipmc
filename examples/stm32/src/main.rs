//! AIP 智能体互联协议 STM-32 示例
//!
//! 演示如何在 STM-32 上创建一个执行器控制智能体节点骨架：
//! 校验预置的 AIC、生成 ACS 能力描述、构建心跳帧，并在主循环中处理收发。
//!
//! ## 资源占用要点
//!
//! - ACS 约 11 KB，因此放在 `static mut`（`.bss`）里，只把 `&mut` 传给构建函数
//! - 构建接口是 `&mut self`（`set_*` / `add_*`），**不要**用按值链式 builder
//! - 收发各用一个栈上帧缓冲（各 524 B），合计约 1 KB 栈
//!
//! ## 运行方式
//!
//! ```bash
//! rustup target add thumbv7em-none-eabihf
//! cargo build --release --target thumbv7em-none-eabihf
//! probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/aipmc-stm32-example
//! ```
//!
//! 详细说明与限制见 `examples/README.md`。

#![no_std]
#![no_main]

use aipmc::basic::acs::{AgentCapability, AgentCapabilitySpec, AgentEndPoint, Transport};
use aipmc::basic::aic::Aic;
use aipmc::basic::protocol::AcpMethod;
use aipmc::basic::skill::AgentSkill;
use aipmc::basic::{ACP_MAX_FRAME_LEN, AcpMessage, AipResult};
use aipmc::stm32::Stm32Hal;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

/// 由注册中心签发、预置在设备上的身份码。
///
/// **这是占位值**：格式合法，但第 10 级校验码是按默认 SALT 计算的示例值。
const PROVISIONED_AIC: &str = "1.2.156.3088.1.0001.00001.ABC123.000000.0M9Y";

/// 能力描述常驻 `.bss`（约 11 KB）
static mut SPEC: MaybeUninit<AgentCapabilitySpec> = MaybeUninit::uninit();

/// 固件入口符号
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    main_entry()
}

/// 在给定的存储上构建能力描述（不产生整体复制）
fn build_spec(aic: Aic, spec: &mut AgentCapabilitySpec) -> AipResult<()> {
    let hal = Stm32Hal::new();
    *spec = hal.capability_spec(aic)?;
    spec.set_name("STM32-Relay-Controller")?;

    let mut skill =
        AgentSkill::new("relay.control", "RelayControl", "Control relay on/off state")?;
    skill.add_tag("actuator")?;
    skill.set_schemas(
        r#"{"type":"object","properties":{"channel":{"type":"integer"}}}"#,
        r#"{"type":"object","properties":{"success":{"type":"boolean"}}}"#,
    )?;

    spec.add_skill(skill)?;
    spec.add_hardware_capability(AgentCapability::ActuatorControl)?;
    spec.add_end_point(AgentEndPoint::new(
        "aip://gateway.local/agents/stm32-ctrl-01",
        Transport::HttpJson,
    )?)?;
    Ok(())
}

/// 构建能力描述并进入主循环
fn main_entry() -> ! {
    // 1. 校验预置的 AIC（本体 AIC：第 9 级全 0）
    let aic = Aic::parse(PROVISIONED_AIC).expect("预置的 AIC 非法");

    // 2. 在 .bss 上原地构建 ACS
    //
    // SAFETY: 单线程裸机固件；写入发生在任何读取之前
    let spec: &mut AgentCapabilitySpec =
        unsafe { (*core::ptr::addr_of_mut!(SPEC)).assume_init_mut() };
    if build_spec(aic, spec).is_err() {
        panic!("构建 AgentCapabilitySpec 失败");
    }
    if spec.validate().is_err() {
        panic!("AgentCapabilitySpec 校验失败");
    }

    // 3. 预编码一个心跳帧，避免在主循环中做动态分配
    let heartbeat = AcpMessage::new(AcpMethod::Heartbeat, 0);
    let mut frame_buf = [0u8; ACP_MAX_FRAME_LEN];
    let frame_len = heartbeat.encode_to(&mut frame_buf).expect("编码失败");

    // 4. 主循环：周期性发送心跳，处理入站消息
    //
    // 实际使用时：
    //   let mut transport = aipmc::transport::UartTransport::new(uart);
    //   transport.send(&heartbeat)?;
    //   let req = transport.recv()?;                 // 入站请求
    //   spec.find_skill("relay.control")             // 技能分发
    loop {
        let _ = (spec.skills.len(), frame_len);
        core::hint::spin_loop();
    }
}

/// panic 处理：固件中通常记录日志后重启
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
