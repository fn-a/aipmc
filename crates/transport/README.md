# aipmc-transport

AIP 智能体互联协议传输层 — 支持 UART/WiFi/BLE 等通信方式。

## 概述

`aipmc-transport` 提供了 ACPs 协议消息在不同物理通道上的传输抽象，基于 `embedded-io` trait 设计，实现了 MCU 间通信的底层收发。

## 传输方式

| 模块 | 传输方式 | 适用场景 | 典型波特率 | 状态 |
|------|----------|----------|------------|------|
| [`uart`] | UART | MCU 间有线通信 | 9600~921600 bps | 已实现 |
| [`wifi`] | WiFi | ESP-32 无线联网 | 1~54 Mbps | 骨架 |
| [`ble`] | BLE | 低功耗无线 | 1~2 Mbps | 骨架 |

## 使用示例（UART）

```rust,ignore
use aipmc::basic::{AcpMessage, AcpMethod};
use aipmc::transport::UartTransport;

// uart 需实现 embedded_io::Read + embedded_io::Write
let mut transport = UartTransport::new(uart);

// 发送消息
let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
transport.send(&msg).unwrap();

// 接收消息
let received = transport.recv().unwrap();
```

## 依赖关系

```
aipmc-transport
├── aipmc-basic    (协议消息编解码)
└── embedded-io   (IO trait 抽象)
```

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `async` | 否 | 启用异步传输支持（`embedded-io-async`） |
| `defmt` | 否 | 启用 `defmt` 格式化输出 |
