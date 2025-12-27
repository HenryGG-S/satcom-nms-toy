# SatCom NMS Mini-Stack — Requirements (MVP)

## 1. Scope and intent
This project implements a minimal Network Management System (NMS) for a simulated satellite communications network.
It focuses on telemetry ingestion, validation, alarm generation/correlation, command scheduling, and replayable demos.

The system is intentionally "ground-segment style" rather than a physics simulation.

## 2. Definitions
- Node: a managed network element (e.g., terminal, modem, hub component).
- Telemetry: periodic measurements sent from a node to the NMS.
- Alarm: a stateful alert raised when a monitored condition persists beyond a threshold.
- Event: an immutable log entry describing an occurrence (telemetry received, alarm raised, command issued, etc.).
- Frame: a protocol message containing header + payload + CRC.

## 3. Functional requirements (MVP)

### 3.1 Telemetry ingestion + validation
R1. The ingest service shall listen for incoming UDP datagrams on a configurable host:port.
R2. The ingest service shall parse frames according to the protocol specification (docs/protocol.md).
R3. The ingest service shall reject any frame that fails CRC verification.
R4. The ingest service shall reject any frame with an unknown magic value.
R5. The ingest service shall reject any frame declaring a payload_len that exceeds the remaining bytes in the datagram.
R6. The ingest service shall support telemetry payload version 1.
R7. For each accepted telemetry message, the ingest service shall update the latest-known state for that node_id.

Acceptance test: sending a known valid telemetry frame updates node state; flipping 1 bit in the frame causes rejection.

### 3.2 Event log
R8. The ingest service shall emit an event record for each accepted telemetry message.
R9. The ingest service shall emit an event record for each rejected frame including a reason category
    (e.g., BAD_CRC, BAD_MAGIC, BAD_LENGTH, UNSUPPORTED_VERSION).
R10. Each event record shall include a timestamp (unix_ms) and a monotonically increasing local event_id.

Acceptance test: run with mixed valid/invalid frames; verify event log contains entries with correct categories.

### 3.3 Alarm engine (per-node)
R11. The system shall implement an alarm rule LOCK_LOST which becomes active when lock==0 persists for >= T_lock_lost ms.
R12. The system shall implement an alarm rule HIGH_BER which becomes active when ber_ppb >= BER_threshold persists for >= T_ber ms.
R13. Each alarm shall have a lifecycle state in {INACTIVE, ACTIVE, ACKED}.
R14. An ACTIVE alarm shall transition to INACTIVE when its triggering condition is not met for >= T_clear ms.
R15. The alarm engine shall suppress flapping by requiring persistence timers for activation and clearance.

Acceptance test: feed lock flaps at < T_lock_lost; verify alarm never becomes ACTIVE. Feed lock==0 for >= T_lock_lost; verify ACTIVE.

### 3.4 Alarm correlation (network-level)
R16. The system shall raise a NETWORK_OUTAGE_SUSPECTED alarm if >= N nodes transition to LOCK_LOST active within a sliding window W ms.
R17. When correlation triggers, the system shall emit a correlation event linking the contributing node alarms.

Acceptance test: cause 3 nodes to lose lock within W; verify correlation alarm raised once and linked.

### 3.5 Command scheduling (MVP)
R18. The system shall accept commands via an API (CLI or HTTP) specifying (node_id, command_type, parameters).
R19. The command scheduler shall assign each command a unique command_id and persist it in a command log.
R20. The system shall transmit commands to the simulator using the command frame format (docs/protocol.md).
R21. The system shall record a command outcome as SUCCESS, FAILURE, or TIMEOUT.

Acceptance test: issue command to a node; simulator responds with ACK; command outcome becomes SUCCESS.

### 3.6 Replayability / demo scenarios
R22. The simulator shall support seeded randomness so runs are repeatable given the same seed and scenario.
R23. The simulator shall support fault injection scenarios including at least:
     - Packet loss (drop rate)
     - Corrupted frame (bit flip)
     - Telemetry stuck-at (constant value)
     - Step-change in BER
R24. The system shall provide a "demo scenario" script that reproduces:
     nominal → injected fault → alarm(s) raised → mitigation command → recovery/clear

Acceptance test: running the demo script twice produces the same alarm timeline (within defined tolerances).

## 4. Non-functional requirements

### 4.1 Reliability and robustness
NFR1. The ingest service shall not crash when receiving malformed or random UDP datagrams up to MAX_DGRAM bytes.
NFR2. The ingest service shall bound memory growth; node state storage shall be O(number_of_nodes).
NFR3. The system shall be able to process at least 500 telemetry frames per second on a developer laptop (best-effort).

### 4.2 Maintainability and design
NFR4. The protocol spec shall be versioned and support forwards compatibility by rejecting unknown versions explicitly.
NFR5. The architecture shall be documented with component and sequence diagrams (docs/architecture.puml).
NFR6. The project shall include automated tests and CI.

### 4.3 Security posture (lightweight, but intentional)
NFR7. Logs shall avoid including raw command parameters that may be treated as sensitive (redact or hash if needed).
NFR8. Protocol parser shall be constant-time with respect to payload contents where practical (no data-dependent loops on attacker-controlled sizes).

## 5. Out of scope (explicitly)
- Cryptography, authentication, and real secure SATCOM protocols.
- Real orbital dynamics, RF propagation models, or detailed link budgets.
- High-availability clustering, distributed consensus, or real database scaling.
- Full-featured web UI.

## 6. Configuration defaults (initial)
- T_lock_lost: 2000 ms
- T_ber: 5000 ms
- T_clear: 3000 ms
- BER_threshold: 20000 ppb
- Correlation: N=3 nodes, W=1000 ms
- MAX_DGRAM: 2048 bytes
