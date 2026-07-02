# aipmc-esp32

AIP 智能体互联协议 ESP-32 平台适配层。

## 概述

`aipmc-esp32` 提供 ESP-32 系列芯片的硬件抽象层（HAL），封装 WiFi/BLE 初始化、GPIO 控制等平台特定功能，使上层协议栈无需关注硬件细节。

## 支持的芯片

| Feature | 芯片型号 |
|---------|----------|
| `esp32` | ESP32 |
| `esp32s2` | ESP32-S2 |
| `esp32s3` | ESP32-S3 |
| `esp32c3` | ESP32-C3 |
| `esp32c6` | ESP32-C6 |

## 依赖关系

```
aipmc-esp32
├── aipmc-basic       (协议核心)
├── aipmc-transport   (传输层)
├── esp-hal           (ESP-32 HAL，待对接)
└── esp-wifi          (ESP-32 WiFi/BLE，待对接)
```

## 使用示例

```rust,ignore
use aipmc::esp32::Esp32Hal;

let mut hal = Esp32Hal::new();
hal.init_wifi().unwrap();

let agent_card = hal.get_device_info().unwrap();
```

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `esp32` | 否 | 启用 ESP32 芯片支持 |
| `esp32s3` | 否 | 启用 ESP32-S3 芯片支持 |
| `wifi` | 否 | 启用 WiFi 功能 |
| `ble` | 否 | 启用 BLE 功能 |
| `defmt` | 否 | 启用 `defmt` 格式化输出 |

## 当前状态

骨架实现，WiFi/BLE 初始化待对接 `esp-hal` / `esp-wifi` crate。
