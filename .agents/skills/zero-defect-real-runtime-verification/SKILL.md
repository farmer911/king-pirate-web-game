---
name: zero-defect-real-runtime-verification
description: Use when writing, testing, or verifying any Rust code, network server, protocol parser, or gameplay feature, before claiming any task or phase is complete.
---

# Zero-Defect Code & Real Runtime Verification

## Overview

Code must compile with **zero errors and zero warnings**, handle all malformed or adversarial inputs with **zero crashes (no panics/unwraps on untrusted input)**, pass **comprehensive edge-case tests**, and be verified on the **real runtime environment** (real server process, real sockets, real browser/client payloads) — never theoretical lip service.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `test-driven-development`, `verification-before-completion`, and `empirical-metrics-and-knowledge-tools`.

## The Iron Laws

```
1. ZERO ERRORS & ZERO WARNINGS: ENFORCE #![deny(warnings, dead_code, unused_imports, unused_variables)].
2. ZERO CRASHES ON UNTRUSTED INPUT: NEVER USE .unwrap() OR .expect() ON NETWORK BYTES, AMF3 PAYLOADS, OR CLIENT ARGS; ALWAYS RETURN Result<T, E>.
3. EXHAUSTIVE EDGE-CASE TESTS: TEST HAPPY PATHS, MALFORMED HEADERS, CORRUPTED CHECKSUMS, BOUNDARY VALUES, DUPLICATE NAMES, AND UNAUTHENTICATED ACCESS.
4. REAL RUNTIME EXECUTION: SPAWN THE REAL ./target/debug/king-pirate SERVER, SEND REAL CLIENT PACKETS OVER TCP/WEBSOCKET/HTTP, AND CAPTURE REAL LOGS/SCREENSHOTS.
```

## Mandatory Verification Matrix

1. **Compiler Gate:** `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test` -> `0 errors`, `0 warnings`, `0 failed`.
2. **Crash-Freedom Gate:** Fuzz/test truncated frames (`0..7` bytes), invalid Zlib streams, invalid AMF3 markers (`0xFF`), negative currency inputs, and missing RPC parameters.
3. **Real Environment Gate:**
   - Start `./target/debug/king-pirate` listening on real OS ports (`:8081` HTTP, `:8003` WebSocket, `:8002` TCP).
   - Execute real HTTP + WebSocket/TCP requests (and headless/live browser verification for UI flows) against the running process.
   - Record actual response bytes, status codes, process RSS (`ps -o rss=`), and latency.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "Unit tests pass, so the server works in the browser" | Unit tests do not test socket framing, WebSocket upgrades, or Flash client boot sequences. Run the real server and real client. |
| "Using `.unwrap()` on client args is fine for now" | A single malformed packet from a client will panic the worker thread or crash the server. Use `?` and `Result`. |
| "One compiler warning is harmless" | Warnings hide bugs and dead code. Zero warnings is mandatory. |

## Red Flags - STOP and Fix

- Any `.unwrap()` or `.expect()` inside non-test runtime handlers (`src/gateway.rs`, `src/amf3.rs`, `src/domain/*.rs`).
- Claiming "game works" or "character registration works" without running the live binary and executing the real login + character creation flow over the socket/browser.
