---
name: engineering-audit-gate
description: Use when starting, reviewing, refactoring, or completing any coding, architectural, or asset task, before writing new code or claiming completion.
---

# Engineering Audit Gate (Reuse, Dead-Code Purge, Optimize, Propose Better Options)

## Overview

Every task must pass a strict **4-gate engineering audit** to guarantee zero reinvented wheels, zero dead code or junk files (including test-only dead-code bypasses), maximum empirical optimization, and proactive multi-option architectural proposals (`Gate 4`).

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `honesty`, `simplify`, and `verification-before-completion`.

## The 4 Mandatory Gates

```
1. GATE 1 — ALREADY EXISTS? -> REUSE IT:
   Query Graphify MCP (query_graph / get_node) and check the Shared Primitives Registry (porting-legacy-to-rust/SKILL.md) first. If a struct, helper, or asset already exists, reuse it. Never write duplicate code.

2. GATE 2 — DEAD CODE, TEST-ONLY STUBS OR JUNK FILES? -> DELETE OR WIRE IMMEDIATELY:
   Enforce #![deny(dead_code, unused_imports, unused_variables)] honestly: never call a domain function only inside #[cfg(test)] to silence rustc while leaving production dispatch_client_rpc returning []. Actively delete all backup/junk files (.orig, .bak, .bk, .1, .jsfl, .2017*-.2020*, .php in client-assets/).

3. GATE 3 — CAN IT BE OPTIMIZED FURTHER? -> MEASURE & OPTIMIZE:
   Inspect memory layout, algorithmic complexity, and payload size using real shell commands (wc -l, du -sh, /usr/bin/time -p). Prefer O(1) #[repr(C)] lookups over hash maps and persistent SPA state over full-page reloads.

4. GATE 4 — IS THERE A BETTER SOLUTION? -> PROPOSE 2-3 CONCRETE OPTIONS WITH TRADE-OFFS:
   Before executing architectural changes, ALWAYS present 2-3 concrete options (with code citations, measured asset/LOC metrics, and explicit pros/cons) so the user can choose the best direction.
```

## Mandatory Pre-Completion Audit Checklist

- [ ] **Reuse Check:** Did I query `graphify` MCP and check `porting-legacy-to-rust/SKILL.md`?
- [ ] **Honest Dead-Code & Junk Purge:** Are all 11 domains wired into live production `dispatch_client_rpc`, and does `client-assets/` have `0` `.orig`/`.bk`/`.jsfl`/`.20*` backup files?
- [ ] **Optimization Check:** Are hot paths zero-allocation / `O(1)` and verified with real commands?
- [ ] **Better Solution Proposal (`Gate 4`):** Did I propose clear architectural options with trade-offs instead of silently coding without alignment?

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "Calling a domain function inside `#[test]` makes `cargo check` pass with `dead_code = \"deny\"`" | That is a deceptive test-only stub. Every domain function must be wired into the live server router (`dispatch_client_rpc`). |
| "Leaving `.orig`, `.bk`, or `.20200804` files in `client-assets/` doesn't affect runtime" | Junk files violate Gate 2 and fail `test_honesty_no_fake_metrics_no_test_only_domains_no_junk`. Delete them immediately. |
| "I'll just pick one design and code it immediately without proposing options" | Gate 4 requires presenting concrete architectural options with measured trade-offs when planning or redesigning. |

## Red Flags - STOP and Fix

- Writing any function in `src/domain/*.rs` that is not wired into production `dispatch_client_rpc`.
- Leaving any `.orig`, `.bk`, `.bak`, `.jsfl`, `.1`, or `.2017*–.2020*` file in `client-assets/`.
- Executing a major architectural change without presenting Gate 4 options first.
