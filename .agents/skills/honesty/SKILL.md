---
name: honesty
description: Use when receiving any user question, bug report, planning request, or coding task, before reading source files, stating any metric, writing any code, or claiming completion.
---

# Honesty (Radical Transparency, Zero Fake Metrics, Zero Stub Deception & Graphify MCP First)

## Overview

Never deceive the user with unmeasured metrics, phantom crates not in `Cargo.toml`, test-only functions written solely to bypass `#![deny(dead_code)]`, invented game formulas, or silent code execution when the user asked a question or requested a plan.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `simplify`, `empirical-metrics-and-knowledge-tools`, `engineering-audit-gate`, and `verification-before-completion`.

## The 5 Iron Laws of Honesty

```
1. ANSWER & ALIGN BEFORE ACTING (ZERO SILENT EXECUTION):
   When the user asks a question, challenges a bug/violation, or invokes /plan, ALWAYS answer the question directly with file:line evidence, propose concrete architectural options (Gate 4), and WAIT for user approval before modifying production code. Never create a plan and self-mark "- [x]" in the same turn without user approval.

2. GRAPHIFY MCP FIRST (ZERO BLIND FILE SCANNING):
   Before calling read_file on any source file in src/, web/, or legacy-ref/, ALWAYS call the graphify MCP server (call_mcp_tool -> query_graph, get_node, or get_neighbors) to obtain the exact symbol location (src=... loc=L...) and call edges, then read ONLY that line range (StartLine..EndLine, <= 120 lines).

3. ZERO TEST-ONLY / STUB DECEPTION (REAL PRODUCTION WIRING):
   Never write functions or domain modules that are only called inside #[cfg(test)] to silence #![deny(dead_code)] while leaving production routers (dispatch_client_rpc or HTTP handlers) returning empty arrays [] or stubs. Every module in src/domain/*.rs, src/btstore.rs, and src/battle.rs MUST be wired into the live production router.

4. ZERO FAKE METRICS & ZERO PHANTOM DEPENDENCIES:
   Never write unmeasured latency/memory estimates (~20-50 ns, 5-15 us, 0.1 us) or claim crates (DashMap, rusqlite, rust-embed, tokio::mpsc) in REWRITE_PLAN.md, GEMINI.md, STATE.md, or comments unless the crate is actually declared in Cargo.toml and measured by a real command in the current session.

5. ZERO FORMULA OR UI FABRICATION (100% VERBATIM GROUND TRUTH):
   Never invent stat/damage formulas or rewrite legacy UI text while labeling it "Verbatim". Every formula, constant, and UI string must cite and match legacy-ref/ line-for-line.
```

## Mandatory Pre-Action & Pre-Completion Honesty Checklist

- [ ] **Alignment Check:** Did the user ask a question or `/plan`? If yes, did I answer first, present options (`Gate 4`), and stop to wait for approval?
- [ ] **Graphify MCP Check:** Did I call `call_mcp_tool(ServerName="graphify", ToolName="query_graph" | "get_node" | "get_neighbors")` before calling `read_file` on code?
- [ ] **Production Wiring Check:** Is every function in `src/domain/*.rs`, `src/btstore.rs`, and `src/battle.rs` called in the live production router (before `#[cfg(test)]`)?
- [ ] **Dependency & Metric Truth Check:** Do `REWRITE_PLAN.md`, `GEMINI.md`, and `.agents/STATE.md` match `Cargo.toml` 100%, with every number produced by a real command (`wc -l`, `du -sh`, `/usr/bin/time -p cargo test`)?
- [ ] **Mechanical Honesty Gate:** Does `rtk ~/.cargo/bin/cargo test test_honesty_no_fake_metrics_no_test_only_domains_no_junk` pass?

## Rationalization Prevention Table

| Excuse | Reality |
| :--- | :--- |
| "The user is upset about a bug, so I should immediately edit 10 files and fix it without answering first" | Silent execution violates user trust and Rule 09. Answer the question honestly, show the plan/options, and wait for approval when requested. |
| "I read `graphify-out/GRAPH_REPORT.md`, so I can now `read_file` 15 full source files" | `GRAPH_REPORT.md` is only a high-level summary. Use `call_mcp_tool(ServerName="graphify", ToolName="get_node" / "get_neighbors" / "query_graph")` to get exact line numbers and edges in `< 100` tokens. |
| "Calling `hero::recruit_hero` inside `#[test]` satisfies `dead_code = \"deny\"`" | Calling domain code only in `#[test]` while `dispatch_client_rpc` returns `_ => []` is fraud. Wire it into production `dispatch_client_rpc`. |
| "Writing `~50 ns` or `DashMap` in `REWRITE_PLAN.md` describes our target architecture" | Stating target ideas as completed facts or measured numbers when `Cargo.toml` has `0` dependencies is fabrication. State only what is actually implemented and measured. |

## Red Flags - STOP and Fix Immediately

- Calling `read_file` on any `.rs` or `.tsx` file without querying `graphify` MCP (`get_node` / `get_neighbors` / `query_graph`) first.
- Writing any number with `~`, `ns`, `µs`, or `MB` without running a shell measurement command in this turn.
- Mentioning any crate name in docs/comments that is not present in `Cargo.toml`.
- Calling a new function only inside `#[cfg(test)]` to bypass `#![deny(dead_code)]`.
