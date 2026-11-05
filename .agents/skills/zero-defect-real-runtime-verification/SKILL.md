---
name: zero-defect-real-runtime-verification
description: Use when writing, testing, or verifying any Rust code, network server, protocol parser, or gameplay feature, before claiming any task or phase is complete.
---

# Zero-Defect Code & Real Runtime Verification

## Overview

Code must compile with **zero errors and zero warnings**, have **100% of domain modules wired into the live production router (never only in `#[cfg(test)]`)**, handle all malformed or adversarial inputs with **zero crashes**, and be verified on the **real runtime environment** (`./target/debug/king-pirate serve` + real browser/socket requests).

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `playwright-e2e-evidence`, `honesty`, `test-driven-development`, `verification-before-completion`, and `empirical-metrics-and-knowledge-tools`.

## The Iron Laws

```
1. ZERO ERRORS & ZERO WARNINGS: ENFORCE #![deny(warnings, dead_code, unused_imports, unused_variables)].
2. ZERO TEST-ONLY DEAD-CODE BYPASS: EVERY MODULE IN src/domain/*.rs, src/btstore.rs, AND src/battle.rs MUST BE CALLED BY LIVE PRODUCTION dispatch_client_rpc (VERIFIED BY test_honesty_no_fake_metrics_no_test_only_domains_no_junk).
3. ZERO CRASHES ON UNTRUSTED INPUT: NEVER USE .unwrap() OR .expect() ON NETWORK BYTES, AMF3 PAYLOADS, OR CLIENT ARGS; ALWAYS RETURN Result<T, E>.
4. REAL RUNTIME EXECUTION: ALWAYS RUN cargo build BEFORE STARTING ./target/debug/king-pirate serve, SEND REAL CLIENT PACKETS OVER TCP/WEBSOCKET/HTTP, AND VERIFY IN REAL CHROME.
```

## Mandatory Verification Matrix

1. **Compiler & Honesty Gate:** `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test` -> `0 errors`, `0 warnings`, `0 failed` (including `test_honesty_no_fake_metrics_no_test_only_domains_no_junk`).
2. **Production Binary Rebuild Gate:** Run `rtk ~/.cargo/bin/cargo build` before launching `./target/debug/king-pirate serve` so the daemon binary is never stale.
3. **Real Environment Gate:**
   - Start `./target/debug/king-pirate serve` listening on `:8081` HTTP, `:8003` WebSocket, `:8002` TCP.
   - Execute real HTTP + WebSocket RPC calls for login, character creation, and gameplay domains (`hero`, `bag`, `forge`, `copy`, `arena`, `boss`, `guild`, `town`, `pet`, `economy`) against the live server.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I tested `copy::attack_copy_enemy` by calling it directly in `#[test]`, so it works" | Direct Rust function calls in `#[test]` bypass `dispatch_client_rpc`. You must wire it into `dispatch_client_rpc` and test it through the RPC router. |
| "`cargo test` passed, so `./target/debug/king-pirate` is up to date" | `cargo test` only compiles the test harness binary (`target/debug/deps/...`), leaving `target/debug/king-pirate` stale! Always run `cargo build`. |

## Red Flags - STOP and Fix

- Any domain module in `src/domain/*.rs` that is called in `#[cfg(test)]` but missing from `dispatch_client_rpc`.
- Starting `./target/debug/king-pirate serve` without running `cargo build` after the last `.rs` edit.
