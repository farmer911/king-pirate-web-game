---
name: empirical-metrics-and-knowledge-tools
description: Use when stating any metric, reading code across the repository, updating file documentation, or drawing architecture diagrams and plans.
---

# Empirical Metrics & Knowledge Tooling (Graphify MCP, Obsidian, Archify)

## Overview

Never invent, estimate, or guess any number, benchmark, or dependency. Every metric must come from a real command execution in the current session, every technology claim must match `Cargo.toml`, and all code reading, file knowledge tracking, and diagramming must use **Graphify MCP**, **Obsidian**, and **Archify**.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `honesty`, `simplify`, `updating-project-knowledge`, and `planning-with-archify`.

## The Iron Laws

```
1. ZERO FABRICATED METRICS OR PHANTOM CRATES: NEVER CITE ANY NUMBER (MB, MS, US, NS, LOC, %) WITHOUT RUNNING A REAL COMMAND (du, ps, time, wc, cargo test) FIRST, AND NEVER CLAIM ANY CRATE NOT PRESENT IN Cargo.toml.
2. GRAPHIFY MCP FIRST FOR O(1) CODE READING: ALWAYS CALL call_mcp_tool(ServerName="graphify", ToolName="query_graph" | "get_node" | "get_neighbors") BEFORE CALLING read_file ON SOURCE CODE, THEN READ ONLY THE EXACT LINE WINDOW (StartLine..EndLine, <= 120 LINES).
3. OBSIDIAN FOR PER-FILE KNOWLEDGE: RUN sync-knowledge.sh ON EVERY FILE CHANGE SO graphify-out/obsidian/*.md MAINTAINS UP-TO-DATE WIKILINKS FOR EVERY MODULE.
4. ARCHIFY FOR DIAGRAMS & PLANNING: ALWAYS UPDATE .archify/explainer-vht-rust-rewrite/candidate.archify.md AND COMPILE docs/rewrite-plan.html WHEN PLANNING OR DIAGRAMMING.
```

## Tooling Workflow

| Task | Mandatory Tool | Command / Artifact |
| :--- | :--- | :--- |
| **Locate Symbol & Call Edges Fast** | **Graphify MCP** | `call_mcp_tool(ServerName="graphify", ToolName="get_node" / "get_neighbors" / "query_graph")` -> `read_file` exact lines (`<= 120` lines) |
| **Measure Disk / RAM / Time / LOC** | Shell + `rtk` | `du -sh`, `ps -o pid,rss,command`, `/usr/bin/time -p`, `rtk wc -l`, `rtk cargo test` |
| **Update Per-File Knowledge** | **Obsidian Vault** | `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` -> `graphify-out/obsidian/` |
| **Draw Diagrams & Plans** | **Archify** | `node ~/.gemini/config/skills/archify/bin/archify.mjs finalize explainer ...` -> `docs/rewrite-plan.html` |

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I can estimate that this takes `~50 ns` or `5–15 µs`" | Unmeasured estimates are hallucinations. Measure it with a real command or do not state a number. |
| "I read `GRAPH_REPORT.md`, so I can now read 15 full `.rs` files" | `GRAPH_REPORT.md` only summarizes top God nodes. Call `graphify` MCP (`get_node`, `get_neighbors`, `query_graph`) to get exact `loc=L...` in `< 100` tokens. |
| "I'll mention `DashMap` or `rusqlite` in `REWRITE_PLAN.md` even though `Cargo.toml` has 0 dependencies" | Claiming crates that are not in `Cargo.toml` is architectural deception. Fails `test_honesty_no_fake_metrics_no_test_only_domains_no_junk`. |

## Red Flags - STOP and Fix

- Calling `read_file` on a `.rs` or `.tsx` file before querying `graphify` MCP (`query_graph`, `get_node`, `get_neighbors`).
- Outputting `~`, `approximately`, `ns`, or `µs` numbers not produced by a command in this session.
- Modifying `src/**/*.rs` without syncing `graphify-out/obsidian/` via `sync-knowledge.sh`.
