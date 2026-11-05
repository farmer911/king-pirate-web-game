---
name: engineering-audit-gate
description: Use when starting, reviewing, refactoring, or completing any coding, architectural, or asset task, before writing new code or claiming completion.
---

# Engineering Audit Gate (Reuse, Dead-Code Purge, Optimize, Propose Better)

## Overview

Every task must pass a strict **4-gate engineering audit** to guarantee zero reinvented wheels, zero dead code or junk files, maximum empirical optimization, and proactive architectural upgrades.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `simplify` and `verification-before-completion`.

## The 4 Mandatory Gates

```
1. GATE 1 — ALREADY EXISTS? -> REUSE IT:
   Check the Shared Primitives Registry (porting-legacy-to-rust/SKILL.md), Rust stdlib, and existing modules first. If a struct, helper, or asset already exists, reuse it. Never write duplicate code.

2. GATE 2 — DEAD CODE OR JUNK FILES? -> DELETE IMMEDIATELY:
   Enforce #![deny(dead_code, unused_imports, unused_variables)] in Rust and actively delete unused functions, hollow wrappers, backup files (.bak, .swf--, .20200624), authoring sources (.fla), OS artifacts (Thumbs.db), and unreferenced assets.

3. GATE 3 — CAN IT BE OPTIMIZED FURTHER? -> BENCHMARK & OPTIMIZE:
   Inspect memory layout, algorithmic complexity, and payload size. Prefer O(1) #[repr(C)] lookups over hash maps, L1/L2 RAM actors over synchronous network I/O, and compressed formats (CWS/ZWS/Zstd) over raw bytes.

4. GATE 4 — IS THERE A BETTER SOLUTION? -> PROPOSE WITH CODE & METRICS:
   Never settle for a suboptimal design just because it is faster to type. Always evaluate and propose the highest-performance, lowest-latency, most secure solution backed by file:line citations and measured metrics (Rule 07 & Rule 08).
```

## Mandatory Pre-Completion Audit Checklist

Before finishing any task, verify all 4 gates:
- [ ] **Reuse Check:** Did I check `porting-legacy-to-rust/SKILL.md` and register any new reusable primitive?
- [ ] **Dead Code Purge:** Does `cargo check && cargo test` pass with zero dead code, and are all backup/junk files removed?
- [ ] **Optimization Check:** Are hot paths zero-allocation / `O(1)` and are static/client assets compressed?
- [ ] **Better Solution Proposal:** Did I surface any superior architectural or performance upgrade with measured evidence?

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "Writing a small helper locally is faster than checking `porting-legacy-to-rust/SKILL.md`" | Duplicate helpers create drift and bloat. Always check and reuse the Shared Primitives Registry. |
| "Leaving `.fla` or `.bak` files in `client-assets/` doesn't hurt runtime" | Junk files waste disk space and pollute directory scans. Delete them immediately. |
| "The user didn't ask if there is a better solution" | Rule 08 and Gate 4 require proactively proposing superior solutions with empirical metrics on every task. |

## Red Flags - STOP and Fix

- Writing a new function without checking if `src/{protocol,amf3,crypt,actor,data,gateway,btstore,battle}.rs` already provides it.
- Leaving any dead code, unused import, or `.bak`/`.fla`/`Thumbs.db` file on disk.
- Proposing an optimization without measuring actual byte/latency savings first.
