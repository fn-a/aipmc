# 平台示例

两个**固件骨架**示例，演示如何在 MCU 上组织 AIPMC 智能体节点：

| 目录     | 平台    | 目标三元组                | 工具链要求                              |
|----------|---------|---------------------------|-----------------------------------------|
| `esp32/` | ESP-32  | `xtensa-esp32-none-elf`   | esp-rs 工具链（`espup install`）        |
| `stm32/` | STM-32  | `thumbv7em-none-eabihf`   | `rustup target add thumbv7em-none-eabihf` |

## 构建

### STM-32

```bash
cd examples/stm32
rustup target add thumbv7em-none-eabihf
cargo check --target thumbv7em-none-eabihf      # 只做类型检查
cargo build --release --target thumbv7em-none-eabihf
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/aipmc-stm32-example
```

### ESP-32

```bash
# 一次性安装 esp-rs 工具链
cargo install espup
espup install

cd examples/esp32
cargo build --release --target xtensa-esp32-none-elf
espflash flash target/xtensa-esp32-none-elf/release/aipmc-esp32-example
```

## 当前限制

示例只提供裸 `main` 符号，**不包含**平台运行时：

- 链接/烧录需要在真实工程中引入 `esp-hal`（`#[esp_hal::main]`）或
  `cortex-m-rt`（`#[cortex_m_rt::entry]`）来生成启动代码与向量表；
- 示例中的中文注释标注了 `transport.send/recv` 与 `spec.find_skill` 的接线位置，
  需要接入真实的 UART/BLE 外设句柄；
- 示例中的 AIC 常量是**占位值**（格式合法、校验码为示例值），实际部署需替换为
  注册中心签发的 AIC；
