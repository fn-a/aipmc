//! 字节流传输通道测试（UART / SPI 别名、WiFi / BLE 占位实现）

use aipmc::basic::{AcpMessage, AcpMethod, AcpStatusCode, AipError};
use aipmc::transport::{BleTransport, SpiTransport, StreamTransport, UartTransport, WifiTransport};

use super::{FailingDevice, MockDuplex};

/// 发送时写出的字节恰好是一帧，且内容与 `encode()` 一致
#[test]
fn test_send_writes_exact_frame() {
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 7, b"ping").unwrap();
    let expected = msg.encode().unwrap();

    let mut transport = UartTransport::new(MockDuplex::default());
    transport.send(&msg).unwrap();

    let device = transport.into_inner();
    assert_eq!(device.tx.len(), msg.frame_len());
    assert_eq!(device.tx, expected.as_slice());
    assert!(device.rx.is_empty());
}

/// 接收时按帧读取，多余字节保留在底层设备中（不越界消费）
#[test]
fn test_recv_reads_exactly_one_frame() {
    let first = AcpMessage::with_payload(AcpMethod::Initialize, 1, b"aaa").unwrap();
    let second = AcpMessage::new(AcpMethod::Heartbeat, 2);

    let mut stream = first.encode().unwrap().as_slice().to_vec();
    stream.extend_from_slice(&second.encode().unwrap());

    let mut transport = SpiTransport::new(MockDuplex::with_input(&stream));
    assert_eq!(transport.recv().unwrap(), first);
    assert_eq!(transport.inner_mut().rx.len(), second.frame_len());
    assert_eq!(transport.recv().unwrap(), second);
    assert!(transport.inner_mut().rx.is_empty());
}

/// 同一句柄上先发后收（全双工），验证读写复用同一设备引用
#[test]
fn test_send_then_recv_on_same_handle() {
    let request = AcpMessage::with_payload(AcpMethod::Stream, 3, b"req").unwrap();
    let reply = request.response(AcpStatusCode::Ok, b"res").unwrap();
    let reply_bytes = reply.encode().unwrap();
    let request_bytes = request.encode().unwrap();

    let mut device = MockDuplex::with_input(&reply_bytes);
    let mut transport = StreamTransport::new(&mut device);
    transport.send(&request).unwrap();
    assert_eq!(transport.recv().unwrap(), reply);
    assert_eq!(transport.inner_mut().tx, request_bytes.as_slice());
}

/// 空设备读取 → UnexpectedEof
#[test]
fn test_recv_on_empty_device_is_eof() {
    let mut transport = UartTransport::new(MockDuplex::default());
    assert!(matches!(transport.recv(), Err(AipError::UnexpectedEof)));
}

/// 底层写失败 → 按错误类型映射（TimedOut → Timeout）
#[test]
fn test_send_maps_io_error() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 1);
    let mut transport = UartTransport::new(FailingDevice {
        kind: embedded_io::ErrorKind::TimedOut,
    });
    assert!(matches!(transport.send(&msg), Err(AipError::Timeout)));
}

/// 底层不支持的操作 → NotImplemented
#[test]
fn test_send_maps_unsupported() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 1);
    let mut transport = UartTransport::new(FailingDevice {
        kind: embedded_io::ErrorKind::Unsupported,
    });
    assert!(matches!(
        transport.send(&msg),
        Err(AipError::NotImplemented)
    ));
}

/// 回归：WiFi / BLE 占位实现必须返回 NotImplemented 而不是 panic
#[test]
fn test_wifi_and_ble_return_not_implemented() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 0);

    let mut wifi = WifiTransport::new();
    assert!(matches!(wifi.send(&msg), Err(AipError::NotImplemented)));
    assert!(matches!(wifi.recv(), Err(AipError::NotImplemented)));

    let mut ble = BleTransport::new();
    assert!(matches!(ble.send(&msg), Err(AipError::NotImplemented)));
    assert!(matches!(ble.recv(), Err(AipError::NotImplemented)));
}

/// `new()` 类型同时具备 `Default` 实现（符合 clippy::new_without_default）
#[test]
// 这里刻意调用 `Default::default()` 以验证该实现存在
#[allow(clippy::default_constructed_unit_structs)]
fn test_transport_default_impls() {
    let mut wifi = WifiTransport::default();
    let mut ble = BleTransport::default();
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 0);
    assert!(wifi.send(&msg).is_err());
    assert!(ble.send(&msg).is_err());

    let mut stream: StreamTransport<MockDuplex> = StreamTransport::new(MockDuplex::default());
    assert!(stream.recv().is_err());
    assert!(stream.inner_mut().tx.is_empty());
}
