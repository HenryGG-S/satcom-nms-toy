use crc32fast::Hasher;

pub const MAGIC: u32 = 0x4E4D5353; // "SSMN" little-endian-ish marker
pub const VERSION: u8 = 1;

#[repr(u8)]
pub enum MsgType {
    Telemetry = 1,
    Command = 2,
}

pub fn build_frame(msg_type: MsgType, seq: u32, unix_ms: u64, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + 1 + 1 + 2 + 4 + 8 + 2 + payload.len() + 4);

    buf.extend_from_slice(&MAGIC.to_le_bytes());
    buf.push(VERSION);
    buf.push(msg_type as u8);
    buf.extend_from_slice(&0u16.to_le_bytes()); // flags
    buf.extend_from_slice(&seq.to_le_bytes());
    buf.extend_from_slice(&unix_ms.to_le_bytes());
    let payload_len: u16 = payload.len().try_into().expect("payload too large");
    buf.extend_from_slice(&payload_len.to_le_bytes());
    buf.extend_from_slice(payload);

    let mut hasher = Hasher::new();
    hasher.update(&buf);
    let crc = hasher.finalize();
    buf.extend_from_slice(&crc.to_le_bytes());

    buf
}
