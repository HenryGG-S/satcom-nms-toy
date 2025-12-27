pub fn build_telemetry_payload(
    node_id: u16,
    snr_x10: i16,
    ber_ppb: u32,
    lock: u8,
    temp_c_x10: i16,
    cpu_pct: u8,
) -> Vec<u8> {
    let mut p = Vec::with_capacity(2 + 2 + 4 + 1 + 2 + 1);
    p.extend_from_slice(&node_id.to_le_bytes());
    p.extend_from_slice(&snr_x10.to_le_bytes());
    p.extend_from_slice(&ber_ppb.to_le_bytes());
    p.push(lock);
    p.extend_from_slice(&temp_c_x10.to_le_bytes());
    p.push(cpu_pct);
    p
}
