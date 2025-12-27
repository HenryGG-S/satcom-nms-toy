# satcom-nms-toy

A “toy” **Satellite Communications Network Management System (NMS)** project.

The goal is to build a small slice of *ground segment / satcom management* software:
- strict protocol parsing of untrusted network input
- telemetry ingestion + state tracking
- alarm generation with anti-flap persistence
- alarm correlation (network-level symptoms)
- command scheduling + audit trail
- deterministic fault-injection demos (replayable scenarios)

---

## What exists right now

Documentation:
- Requirements: `docs/requirements.md`
- Architecture (PlantUML): `docs/architecture.puml`
- Protocol specification: `docs/protocol.md`
- Verification strategy: `docs/verification.md`

Implementation:
- Not yet committed (this is intentionally docs-first)

---

## MVP scope (high level)

### Telemetry ingestion
- UDP receiver
- frame parsing + strict validation (magic/version/length/CRC)
- decode telemetry payload (v1)
- update “latest state” per node
- event log for accepted/rejected frames

### Alarms
- per-node alarms with persistence timers (anti-flap)
- correlation alarm (e.g., many nodes lose lock in a short window)

### Commands
- accept commands via a small API (CLI or HTTP)
- schedule + send commands to simulator
- log outcomes (success/fail/timeout)

### Replayable demo
A scripted scenario:
nominal → injected fault → alarm(s) raised → mitigation command → recovery/clear

---

## Roadmap

- [ ] Implement simulator (telemetry generator + fault injection)
- [ ] Implement ingest service (UDP + parser/CRC + telemetry decoder)
- [ ] Implement event log + latest-state store
- [ ] Implement alarm engine (persistence timers)
- [ ] Implement correlation rule(s)
- [ ] Implement command scheduler + command protocol + ACKs
- [ ] Add end-to-end demo script
- [ ] Add automated tests + CI

---

## Design constraints (intentional)

- Treat all UDP traffic as **untrusted input**
- Avoid floats in wire formats (use scaled integers)
- Explicit versioning and rejection of unsupported versions
- Keep memory bounded (O(number_of_nodes))
- Prefer deterministic behaviour where possible (seeded scenarios)

---

## License

Currently: **The Unlicense** (public domain dedication).  
considering switching to MIT/BSD-2 later.
