---
name: empirical-metrics-and-knowledge-tools
description: Use when stating any metric, reading code across the repository, updating file documentation, or drawing architecture diagrams and plans.
---

# Empirical Metrics & Knowledge Tooling (Graphify, Obsidian, Archify)

## Overview

Never invent, estimate, or guess any number or benchmark. Every metric must come from a real command execution, and all code reading, file knowledge tracking, and diagramming must use **Graphify**, **Obsidian**, and **Archify**.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `simplify`, `updating-project-knowledge`, and `planning-with-archify`.

## The Iron Laws

```
1. ZERO FABRICATED METRICS: NEVER CITE ANY NUMBER (MB, MS, RPS, LOC, FILE COUNT, %) WITHOUT RUNNING A REAL COMMAND (du, ps, time, wc, cargo test) FIRST.
2. GRAPHIFY FOR FAST CODE READING: READ graphify-out/GRAPH_REPORT.md AND .agents/STATE.md FIRST TO LOCATE SYMBOLS AND EDGES IN O(1) WITHOUT SCANNING RAW FILES.
3. OBSIDIAN FOR PER-FILE KNOWLEDGE: RUN sync-knowledge.sh ON EVERY FILE CHANGE SO graphify-out/obsidian/*.md MAINTAINS UP-TO-DATE WIKILINKS FOR EVERY MODULE.
4. ARCHIFY FOR DIAGRAMS & PLANNING: ALWAYS UPDATE .archify/explainer-vht-rust-rewrite/candidate.archify.md AND COMPILE docs/rewrite-plan.html WHEN PLANNING OR DIAGRAMMING.
```

## Tooling Workflow

| Task | Mandatory Tool | Command / Artifact |
| :--- | :--- | :--- |
| **Measure Disk / RAM / Time** | Shell + `rtk` | `du -sh`, `ps -o pid,rss,command`, `/usr/bin/time -p`, `cargo test` |
| **Read Code Architecture Fast** | **Graphify** | Read `.agents/STATE.md` -> `graphify-out/GRAPH_REPORT.md` -> `read_file` exact lines |
| **Update Per-File Knowledge** | **Obsidian Vault** | `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` -> `graphify-out/obsidian/` |
| **Draw Diagrams & Plans** | **Archify** | `node ~/.gemini/config/skills/archify/bin/archify.mjs finalize explainer ...` -> `docs/rewrite-plan.html` |

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I can estimate that this takes ~5ms or ~10MB" | Unmeasured estimates are hallucinations. Measure it with a real command or do not state a number. |
| "I'll grep the whole repo instead of checking `GRAPH_REPORT.md`" | Grepping wastes tokens. `GRAPH_REPORT.md` already indexes all god nodes and communities. |
| "I updated a `.rs` file but skipped `sync-knowledge.sh`" | Skipping `sync-knowledge.sh` leaves the Obsidian per-file notes stale. Always sync in the same turn. |

## Red Flags - STOP and Fix

- Outputting `~`, `approximately`, or theoretical numbers not produced by a command in this session.
- Modifying `src/**/*.rs` without syncing `graphify-out/obsidian/` via `sync-knowledge.sh`.
- Updating `REWRITE_PLAN.md` without regenerating `docs/rewrite-plan.html` via Archify.
