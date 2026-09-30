# aipmc-transport

AIP 智能体互联协议传输层 — 在任意有序字节流上按帧收发协议消息。

## 概述

`aipmc-transport` 基于 `embedded-io` trait 提供协议消息的传输抽象。它是 `no_std` 的，
**不进行动态分配**：`FrameReader` 只用 8 + 516 字节的栈缓冲区。

## 分层

```
StreamTransport<T>   (UartTransport / SpiTransport 别名)   整帧发送 + 按帧接收
        │
FrameReader<T>                                             read_exact 分帧 + 校验
        │
embedded_io::Read + Write                                  厂商 HAL / 驱动
```

## 传输方式

| 模块 | 传输方式 | 适用场景 | 典型速率 | 状态 |
|------|----------|----------|----------|------|
| [`uart`] | UART | MCU 间有线通信 | 9600~921600 bps | 已实现 |
| [`spi`] | SPI | 高速板级通信 | 1~40 Mbps | 已实现（需板级把事务封装为字节流句柄） |
| [`wifi`] | WiFi | ESP-32 无线联网 | 1~54 Mbps | 占位，`send`/`recv` 返回 `AipError::NotImplemented` |
| [`ble`] | BLE | 低功耗无线 | 1~2 Mbps | 占位，`send`/`recv` 返回 `AipError::NotImplemented` |

I2C 不在列表中：它不是有序字节流（每次事务带从机地址与读写方向），无法复用
`StreamTransport`，需要基于目标平台 I2C HAL 单独实现。

## 分帧流程

`FrameReader::read_frame()` 分四步，任一步失败都返回错误而不是继续：

1. `read_exact` 读 8 字节帧头，校验 Magic / 版本 / 方法码 / `payload_len` 区间
2. `read_exact` 读 `payload_len` 字节（Session-ID + 业务负载）
3. 校验和比对（覆盖帧头前 7 字节 + 帧体）
4. 组装 `AcpMessage`

## 使用示例（UART）

```rust,ignore
use aipmc::basic::{AcpMessage, AcpMethod};
use aipmc::transport::UartTransport;

// uart 需实现 embedded_io::Read + embedded_io::Write
let mut transport = UartTransport::new(uart);

// 发送消息（write_all，整帧写出）
let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"ping").unwrap();
transport.send(&msg).unwrap();

// 接收消息（阻塞直到读满一帧）
let received = transport.recv().unwrap();
```

## 直接使用 FrameReader

只需要分帧、不需要写通道时可以直接用 `FrameReader`：

```rust,ignore
use aipmc::transport::FrameReader;

let mut reader = FrameReader::new(some_read_impl);
let msg = reader.read_frame()?;          // 一帧
let rest = reader.into_inner();          // 取回底层句柄
```

## 错误映射

| 底层情况 | `AipError` |
|----------|-----------|
| 流提前结束（`ReadExactError::UnexpectedEof`） | `UnexpectedEof` |
| `ErrorKind::TimedOut` | `Timeout` |
| `ErrorKind::InvalidInput` / `InvalidData` | `InvalidMessage` |
| `ErrorKind::Unsupported` | `NotImplemented` |
| 其他 | `TransportError` |
| 校验和不匹配 | `ChecksumMismatch` |

## 依赖关系

```
aipmc-transport
├── aipmc-basic    (协议常量与消息编解码)
└── embedded-io    (IO trait 抽象)
```

## Feature Flags

| Feature | 默认 | 说明 |
|---------|------|------|
| `defmt` | 否 | 启用 `defmt` 格式化输出（同时转发到 `aipmc-basic/defmt`） |
