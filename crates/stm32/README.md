# aipmc-stm32

AIP 智能体互联协议 STM-32 平台适配层。

## 概述

`aipmc-stm32` 提供 STM-32 系列芯片的硬件抽象层（HAL），封装 UART/SPI 初始化、GPIO 控制等平台特定功能，使上层协议栈无需关注硬件细节。

## 支持的芯片系列

| Feature | 芯片系列 |
|---------|----------|
| `stm32f4` | STM32F4xx |
| `stm32h7` | STM32H7xx |
| `stm32l4` | STM32L4xx |
| `stm32g4` | STM32G4xx |

## 依赖关系

```
aipmc-stm32
├── aipmc-base       (协议核心)
├── aipmc-transport   (传输层)
└── stm32-hal         (STM-32 HAL，待对接)
```

## 使用示例

```rust,ignore
use aipmc::stm32::Stm32Hal;

let mut hal = Stm32Hal::new();
hal.init_uart().unwrap();

let agent_card = hal.get_device_info().unwrap();
```

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `stm32f4` | 否 | 启用 STM32F4 系列支持 |
| `stm32h7` | 否 | 启用 STM32H7 系列支持 |
| `defmt` | 否 | 启用 `defmt` 格式化输出 |

## 当前状态

骨架实现，UART/SPI 初始化待对接 `stm32-hal` crate。
