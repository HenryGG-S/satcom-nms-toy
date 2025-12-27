# Protocol Specification (MVP)

## 1. Goals
- Simple, explicit, binary framing suitable for UDP transport.
- Robust against malformed input (length checks, CRC).
- Versioned payloads for forward evolution.
- Avoid floats in payload encoding (scaled integers).

## 2. Transport
- UDP datagrams.
- Each datagram contains exactly one frame.
- Receiver MUST treat all datagrams as untrusted and validate strictly.

## 3. Endianness
All multi-byte integers are **little-endian**.

## 4. Frame format (v1)
Frame = Header + Payload + CRC32

### 4.1 Header fields
| Field         | Type | Size | Description |
|--------------|------|------|-------------|
| magic        | u32  | 4    | Constant 0x4E4D5353 |
| version      | u8   | 1    | Frame version. Start at 1. |
| msg_type     | u8   | 1    | 1=Telemetry, 2=Command, 3=Ack (optional) |
| flags        | u16  | 2    | Reserved; set to 0 for now |
| seq          | u32  | 4    | Sender sequence number (wrap allowed) |
| unix_ms      | u64  | 8    | Sender timestamp in ms since Unix epoch |
| payload_len  | u16  | 2    | Number of bytes in payload |

Header size: 22 bytes.

### 4.2 Payload
`payload_len` bytes.

### 4.3 CRC32
| Field | Type | Size | Description |
|------|------|------|-------------|
| crc32 | u32 | 4 | CRC32 of bytes from `magic` through end of `payload` |

Polynomial: implementation-defined CRC32 (common IEEE) is acceptable; both sides must match.
(Recommendation: CRC32 IEEE via crc32fast (Rust) and java.util.zip.CRC32 (Java).)

## 5. Validation rules
Receiver MUST:
- Reject frames shorter than minimum header + CRC.
- Reject frames where magic != constant.
- Reject frames where payload_len exceeds remaining bytes.
- Reject frames where CRC32 does not match.
- Reject unsupported frame version values.
- Reject unknown msg_type values (or ignore explicitly, but do not crash).

## 6. Message types

### 6.1 Telemetry message (msg_type = 1)
Payload versioning: Telemetry payload format is tied to **frame version** for MVP.
(If you later want separate payload versions, add a payload_version field inside payload.)

Telemetry payload v1 size: 12 bytes.

| Field       | Type | Size | Encoding |
|------------|------|------|----------|
| node_id    | u16  | 2    | Unique ID of node |
| snr_x10    | i16  | 2    | SNR in dB * 10 |
| ber_ppb    | u32  | 4    | Bit error rate in parts-per-billion |
| lock       | u8   | 1    | 0 or 1 |
| temp_c_x10 | i16  | 2    | Temperature °C * 10 |
| cpu_pct    | u8   | 1    | 0..100 |

Notes:
- Using scaled ints avoids float parsing issues and is common in embedded/telemetry systems.
- Values outside expected ranges should be treated as invalid telemetry (implementation choice: clamp or reject).

### 6.2 Command message (msg_type = 2)
For MVP, a minimal command payload:

| Field       | Type | Size | Description |
|------------|------|------|-------------|
| node_id    | u16  | 2    | Target node |
| command_id | u32  | 4    | Unique id assigned by scheduler |
| cmd_type   | u8   | 1    | 1=RESET_MODEM, 2=SET_MODCOD, 3=SET_FREQ_PLAN |
| param_len  | u16  | 2    | parameter byte length |
| params     | u8[] | var  | command parameters |

Command receiver MUST validate:
- param_len fits inside payload.
- cmd_type supported; otherwise respond with failure (if ACK implemented) or ignore safely.

### 6.3 Ack message (msg_type = 3) (optional MVP+)
Ack payload:

| Field       | Type | Size | Description |
|------------|------|------|-------------|
| node_id    | u16  | 2    | reporting node |
| command_id | u32  | 4    | command being acknowledged |
| status     | u8   | 1    | 0=SUCCESS, 1=FAILURE |
| code       | u16  | 2    | optional failure code |
| msg_len    | u16  | 2    | length of message |
| message    | u8[] | var  | utf-8 (optional) |

## 7. Versioning strategy (practical)
- Frame `version` increments when header semantics change.
- Receivers MUST explicitly reject unsupported versions and log UNSUPPORTED_VERSION.
- Backwards compatibility goal: ingest service supports v1; v2 can be added later with a feature flag.
- Additive changes to payload should be done by creating a v2 payload and allowing both decoders.

## 8. Security / abuse considerations
- UDP allows spoofing; treat source addresses as untrusted.
- Always bound payload_len and datagram size.
- Avoid allocating based solely on declared sizes without verifying bounds.
