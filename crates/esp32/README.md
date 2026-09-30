# aipmc-esp32

AIP 智能体互联协议 ESP-32 平台适配层。

## 概述

`aipmc-esp32` 提供 ESP-32 系列芯片的硬件抽象层（HAL）。它是 `no_std` 的，
**不依赖任何 `esp-*` crate**：芯片型号通过 Cargo feature 在编译期选定。

## 支持的芯片

| Feature | 芯片型号 | BLE |
|---------|----------|-----|
| `esp32`（默认） | ESP32 | 有 |
| `esp32s2` | ESP32-S2 | 无 |
| `esp32s3` | ESP32-S3 | 有 |
| `esp32c3` | ESP32-C3 | 有 |
| `esp32c6` | ESP32-C6 | 有 |

**芯片 feature 互斥**：同时启用多个会触发 `compile_error!`，不会静默选择其中一个。
因此本 crate 不支持 `--all-features`，CI 中按芯片分别构建。

未启用任何芯片 feature 时按 `esp32` 处理。

## 当前状态

| 能力 | 状态 |
|------|------|
| 芯片识别（`Chip` / `chip()`） | 已实现 |
| ACS 能力描述生成（`capability_spec(aic)`，`name` 默认取芯片展示名如 `"ESP32-S3"`） | 已实现 |
| `init_wifi()`（需 `wifi` feature） | 占位，返回 `AipError::NotImplemented` |
| `init_ble()`（需 `ble` feature） | 占位，返回 `AipError::NotImplemented` |
| `esp-hal` / `esp-wifi` 对接 | 未开始 |

## 使用示例

```rust,ignore
use aipmc::basic::aic::Aic;
use aipmc::esp32::{chip, Esp32Hal};

let hal = Esp32Hal::new();
// AIC 由注册中心签发并预置在设备上；设备侧只做校验，不生成
let aic = Aic::parse("1.2.156.3088.1.0001.00001.ABC123.000001.176F").unwrap();
let spec = hal.capability_spec(aic).unwrap();   // name 默认 = 芯片展示名，如 "ESP32-S3"

// 需要启用相应的 feature
#[cfg(feature = "wifi")]
hal.init_wifi().unwrap();                     // 当前返回 NotImplemented
```

## 依赖关系

```
aipmc-esp32
├── aipmc-basic       (协议核心)
└── aipmc-transport   (传输层)
```

`esp-hal` / `esp-wifi` 尚未加入依赖；对接后 `init_wifi` / `init_ble` 才会真正实现。

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `esp32` / `esp32s2` / `esp32s3` / `esp32c3` / `esp32c6` | 否 | 芯片型号，最多启用一个 |
| `wifi` | 否 | 启用 `init_wifi` API 表面 |
| `ble` | 否 | 启用 `init_ble` API 表面 |
| `defmt` | 否 | 启用 `defmt` 格式化输出（转发到 basic / transport） |
