//! 分帧读取测试

use aipmc::basic::{
    ACP_HEADER_LEN, ACP_MAGIC, ACP_SESSION_ID_LEN, ACP_VERSION, AcpMessage, AcpMethod, AipError,
};
use aipmc::transport::{FRAME_BODY_LEN, FrameReader, map_io_error, map_read_exact_error};

/// 构造原始帧字节流
fn raw_frame(method: u8, payload_len: u16, body: &[u8]) -> Vec<u8> {
    let mut buf = vec![
        (ACP_MAGIC >> 8) as u8,
        (ACP_MAGIC & 0xFF) as u8,
        ACP_VERSION,
        method,
        0,
        (payload_len >> 8) as u8,
        (payload_len & 0xFF) as u8,
        0,
    ];
    buf.extend_from_slice(body);
    buf
}

/// 从字节切片读取一帧
#[test]
fn test_frame_reader_reads_one_frame() {
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 77, b"hello").unwrap();
    let encoded = msg.encode().unwrap();

    let mut reader = FrameReader::new(encoded.as_slice());
    let decoded = reader.read_frame().unwrap();
    assert_eq!(decoded, msg);
    // 缓冲区内恰好一帧，读完后不应剩余任何字节
    assert_eq!(reader.into_inner().len(), 0);
}

/// 回归：一帧读完后必须恰好消耗 `frame_len` 字节，不得多读 4 字节
#[test]
fn test_frame_reader_consumes_exact_frame_boundary() {
    let first = AcpMessage::with_payload(AcpMethod::Initialize, 1, b"aaa").unwrap();
    let second = AcpMessage::with_payload(AcpMethod::Heartbeat, 2, b"bb").unwrap();

    let mut stream = first.encode().unwrap().as_slice().to_vec();
    stream.extend_from_slice(&second.encode().unwrap());

    let mut reader = FrameReader::new(stream.as_slice());
    assert_eq!(reader.read_frame().unwrap(), first);
    // 关键断言：恰好剩下第二帧的长度
    assert_eq!(reader.inner_mut().len(), second.frame_len());
    assert_eq!(reader.read_frame().unwrap(), second);
    assert_eq!(reader.into_inner().len(), 0);
}

/// 连续读取多帧后到达流末尾 → UnexpectedEof
#[test]
fn test_frame_reader_reports_eof_after_last_frame() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 5);
    let encoded = msg.encode().unwrap();
    let mut reader = FrameReader::new(encoded.as_slice());
    assert!(reader.read_frame().is_ok());
    assert!(matches!(reader.read_frame(), Err(AipError::UnexpectedEof)));
}

/// 帧体被截断 → UnexpectedEof（不会把半个帧当成功）
#[test]
fn test_frame_reader_rejects_truncated_body() {
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 1, b"0123456789").unwrap();
    let encoded = msg.encode().unwrap();
    let truncated = &encoded[..encoded.len() - 1];

    let mut reader = FrameReader::new(truncated);
    assert!(matches!(reader.read_frame(), Err(AipError::UnexpectedEof)));
}

/// 帧头被截断 → UnexpectedEof
#[test]
fn test_frame_reader_rejects_truncated_header() {
    let msg = AcpMessage::new(AcpMethod::Heartbeat, 1);
    let encoded = msg.encode().unwrap();
    let mut reader = FrameReader::new(&encoded[..ACP_HEADER_LEN - 1]);
    assert!(matches!(reader.read_frame(), Err(AipError::UnexpectedEof)));
}

/// 回归：`payload_len < 4` 的畸形帧返回 InvalidPayload，不 panic
#[test]
fn test_frame_reader_rejects_short_payload_len() {
    for payload_len in [0u16, 1, 2, 3] {
        let frame = raw_frame(AcpMethod::Heartbeat.code(), payload_len, &[]);
        let mut reader = FrameReader::new(frame.as_slice());
        assert!(
            matches!(reader.read_frame(), Err(AipError::InvalidPayload)),
            "payload_len={payload_len}"
        );
    }
}

/// 声明的长度超过单帧上限 → InvalidPayload
#[test]
fn test_frame_reader_rejects_oversize_declared_payload() {
    let frame = raw_frame(AcpMethod::Heartbeat.code(), 4000, &[0u8; 16]);
    let mut reader = FrameReader::new(frame.as_slice());
    assert!(matches!(reader.read_frame(), Err(AipError::InvalidPayload)));
}

/// Magic 不匹配
#[test]
fn test_frame_reader_rejects_bad_magic() {
    let mut frame = raw_frame(AcpMethod::Heartbeat.code(), 4, &[0, 0, 0, 1]);
    frame[0] = 0x00;
    frame[1] = 0x00;
    let mut reader = FrameReader::new(frame.as_slice());
    assert!(matches!(reader.read_frame(), Err(AipError::InvalidHeader)));
}

/// 校验和不匹配
#[test]
fn test_frame_reader_rejects_bad_checksum() {
    let msg = AcpMessage::with_payload(AcpMethod::Heartbeat, 1, b"test").unwrap();
    let mut encoded = msg.encode().unwrap();
    let last = encoded.len() - 1;
    encoded[last] ^= 0xFF;

    let mut reader = FrameReader::new(encoded.as_slice());
    assert!(matches!(
        reader.read_frame(),
        Err(AipError::ChecksumMismatch)
    ));
}

/// 底层报超时 → Timeout
#[test]
fn test_frame_reader_maps_timeout() {
    use crate::transport::FailingDevice;
    let device = FailingDevice {
        kind: embedded_io::ErrorKind::TimedOut,
    };
    let mut reader = FrameReader::new(device);
    assert!(matches!(reader.read_frame(), Err(AipError::Timeout)));
}

/// 帧体缓冲区足以容纳最大帧
#[test]
fn test_frame_body_len_covers_max_payload() {
    use aipmc::basic::ACP_MAX_PAYLOAD_LEN;
    assert_eq!(FRAME_BODY_LEN, ACP_SESSION_ID_LEN + ACP_MAX_PAYLOAD_LEN);
}

/// 最大负载帧也能正常读取
#[test]
fn test_frame_reader_handles_max_payload() {
    use aipmc::basic::ACP_MAX_PAYLOAD_LEN;
    let big = vec![0x5Au8; ACP_MAX_PAYLOAD_LEN];
    let msg = AcpMessage::with_payload(AcpMethod::Stream, 9, &big).unwrap();
    let encoded = msg.encode().unwrap();
    let mut reader = FrameReader::new(encoded.as_slice());
    assert_eq!(reader.read_frame().unwrap(), msg);
}

/// IO 错误类型映射
#[test]
fn test_io_error_mapping() {
    use embedded_io::{ErrorKind, ReadExactError};

    assert_eq!(map_io_error(ErrorKind::TimedOut), AipError::Timeout);
    assert_eq!(
        map_io_error(ErrorKind::InvalidInput),
        AipError::InvalidMessage
    );
    assert_eq!(
        map_io_error(ErrorKind::InvalidData),
        AipError::InvalidMessage
    );
    assert_eq!(
        map_io_error(ErrorKind::Unsupported),
        AipError::NotImplemented
    );
    assert_eq!(map_io_error(ErrorKind::Other), AipError::TransportError);

    let eof: ReadExactError<ErrorKind> = ReadExactError::UnexpectedEof;
    assert_eq!(map_read_exact_error(eof), AipError::UnexpectedEof);
    let other: ReadExactError<ErrorKind> = ReadExactError::Other(ErrorKind::BrokenPipe);
    assert_eq!(map_read_exact_error(other), AipError::TransportError);
}

/// 帧头与 `AcpHeader` 解析保持一致：读取后 header 字段可回读
#[test]
fn test_frame_reader_preserves_header_flags() {
    let mut msg = AcpMessage::with_payload(AcpMethod::Stream, 1, b"x").unwrap();
    msg.header.set_final(false);
    msg.header.set_encrypted(true);
    msg.refresh_checksum();

    let encoded = msg.encode().unwrap();
    let mut reader = FrameReader::new(encoded.as_slice());
    let decoded = reader.read_frame().unwrap();
    assert!(!decoded.header.is_final());
    assert!(decoded.header.is_encrypted());
    assert_eq!(decoded.header.payload_len, msg.header.payload_len);
}
