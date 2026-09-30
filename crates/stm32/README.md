# aipmc-stm32

AIP 智能体互联协议 STM-32 平台适配层。

## 概述

`aipmc-stm32` 提供 STM-32 系列芯片的硬件抽象层（HAL）。它是 `no_std` 的，
**不依赖任何 `stm32-*` crate**：芯片系列通过 Cargo feature 在编译期选定。

## 支持的芯片系列

| Feature | 芯片系列 |
|---------|----------|
| `stm32f4`（默认） | STM32F4xx |
| `stm32h7` | STM32H7xx |
| `stm32l4` | STM32L4xx |
| `stm32g4` | STM32G4xx |

**系列 feature 互斥**：同时启用多个会触发 `compile_error!`，不会静默选择其中一个。
因此本 crate 不支持 `--all-features`，CI 中按系列分别构建。

未启用任何系列 feature 时按 `stm32f4` 处理。

## 当前状态

| 能力 | 状态 |
|------|------|
| 系列识别（`Series` / `series()`） | 已实现 |
| ACS 能力描述生成（`capability_spec(aic)`，`name` 默认取系列展示名如 `"STM32H7"`） | 已实现 |
| `init_uart()` | 占位，返回 `AipError::NotImplemented` |
| `stm32-hal` / `embassy-stm32` 对接 | 未开始 |

## 使用示例

```rust,ignore
use aipmc::basic::aic::Aic;
use aipmc::stm32::{series, Stm32Hal};

let hal = Stm32Hal::new();
// AIC 由注册中心签发并预置在设备上；设备侧只做校验，不生成
let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000000.0M9Y").unwrap();
let spec = hal.capability_spec(aic).unwrap();   // name 默认 = 系列展示名，如 "STM32H7"

hal.init_uart().unwrap();                     // 当前返回 NotImplemented
let _ = series();
```

## 依赖关系

```
aipmc-stm32
├── aipmc-basic       (协议核心)
└── aipmc-transport   (传输层)
```

`stm32-hal` / `embassy-stm32` 尚未加入依赖；对接后 `init_uart` 才会真正实现。

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `stm32f4` / `stm32h7` / `stm32l4` / `stm32g4` | 否 | 芯片系列，最多启用一个 |
| `defmt` | 否 | 启用 `defmt` 格式化输出（转发到 basic / transport） |
