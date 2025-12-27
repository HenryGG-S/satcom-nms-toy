mod frame;
mod telemetry;

use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis() as u64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // sends telemetry to ingest-service running locally on UDP 9000
    let target = "127.0.0.1:9000";
    let sock = UdpSocket::bind("0.0.0.0:0").await?;

    let mut seq: u32 = 0;
    let mut rng = rand::thread_rng();

    loop {
        for node_id in 1u16..=5u16 {
            let snr_x10 = rng.gen_range(80..180);     // 8.0..18.0 dB
            let ber_ppb = rng.gen_range(0..50_000);   // 0..5e-5 approx
            let lock = if snr_x10 > 95 { 1 } else { 0 };
            let temp_c_x10 = rng.gen_range(180..420); // 18.0..42.0 C
            let cpu_pct = rng.gen_range(10..85);

            let payload = telemetry::build_telemetry_payload(
                node_id, snr_x10, ber_ppb, lock, temp_c_x10, cpu_pct,
            );

            let frame = frame::build_frame(frame::MsgType::Telemetry, seq, now_ms(), &payload);
            seq = seq.wrapping_add(1);

            sock.send_to(&frame, target).await?;
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await; // ~5 Hz batch
    }
}
