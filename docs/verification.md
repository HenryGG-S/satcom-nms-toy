# Verification & Test Strategy (MVP)

## 1. Philosophy
This system processes untrusted network input and produces stateful alarms.
Verification focuses on:
- Parser correctness and robustness (no crashes, strict validation).
- Alarm correctness (persistence timers prevent false positives).
- Determinism for demos (seeded scenarios).
- Traceability (event log supports post-mortem reasoning).

## 2. Test layers

### 2.1 Unit tests (Rust simulator)
U1. Frame builder produces correct CRC32 (golden vector test).
U2. Telemetry payload encoding matches spec (byte-for-byte).
U3. Fault injector determinism: given a fixed seed and scenario, emitted telemetry sequence is identical.

### 2.2 Unit tests (Java ingest)
U4. Parser rejects bad magic.
U5. Parser rejects payload_len that exceeds buffer.
U6. Parser rejects bad CRC (flip one bit).
U7. Telemetry decoder rejects wrong payload size for v1.
U8. Telemetry decoder parses known golden bytes into expected fields.

Implementation note: keep a `testdata/` directory with one or two golden frames as raw hex.

### 2.3 Property / fuzz-style tests
P1. For random byte arrays up to MAX_DGRAM, the ingest parser never throws uncaught exceptions.
P2. For random frames with consistent declared lengths but random payload, parser either:
    - accepts and decodes successfully, or
    - rejects with a categorized reason,
    but never crashes or hangs.

If full fuzzing is too heavy, do "poor man's fuzzing" in a normal test loop.

### 2.4 Integration tests (end-to-end)
I1. Run ingest service + simulator; verify latest node states update for all node_ids.
I2. Inject fault: lock=0 for node 3 for >= T_lock_lost; verify LOCK_LOST alarm becomes ACTIVE.
I3. Clear condition: restore lock=1 for >= T_clear; verify alarm returns to INACTIVE.
I4. Correlation: drop lock across >= N nodes within W; verify NETWORK_OUTAGE_SUSPECTED raised once.
I5. Command flow: issue RESET_MODEM; simulator returns ACK; command outcome becomes SUCCESS.

These can be executed via a scripted harness (shell, or a small test runner).

## 3. Alarm verification specifics

### 3.1 Persistence timers (anti-flap)
Test cases:
A1. lock flaps: 0 for 500ms then 1 for 500ms repeatedly → should never raise LOCK_LOST if T_lock_lost=2000ms.
A2. lock stays 0 for 2500ms → should raise at ~2000ms, not earlier.
A3. after raised, lock returns 1 briefly (< T_clear) then back to 0 → should remain ACTIVE (no premature clear).
A4. lock stays 1 for >= T_clear → clears.

### 3.2 Threshold boundary tests
B1. BER exactly == BER_threshold counts as high (or not) — choose and document, then test.
B2. BER jitter around threshold should not cause repeated raise/clear due to persistence.

## 4. Performance and resource checks (lightweight)
Perf1. Ingest processes 500 frames/s sustained for 10s without dropping more than X% (best-effort).
Perf2. Memory growth is bounded with number_of_nodes; run 1e5 frames and verify heap does not grow unbounded.

## 5. Observability checks
Obs1. For every rejected frame, emit an event with a reason category.
Obs2. For every alarm state transition (raise, ack, clear), emit an event with node_id and alarm type.
Obs3. For every command, event log includes command_id and outcome.

## 6. Deterministic demo verification
D1. Running demo scenario twice with the same seed yields the same sequence of:
- alarm raised timestamps within +/- 250ms
- command issued timestamp within +/- 250ms
- alarm cleared timestamp within +/- 250ms

Note: allow small tolerance due to scheduler timing, especially on laptops.

## 7. Tooling / CI expectations
- Run unit tests on every push (CI).
- Run formatting/linting:
  - Rust: cargo fmt, cargo clippy
  - Java: mvn test (and optionally spotless/checkstyle if you want extra polish)

## 8. What we deliberately do NOT verify (yet)
- Cryptographic integrity/authenticity of frames (out of scope).
- Real SATCOM protocol compatibility (out of scope).
- Distributed deployment reliability (out of scope).
