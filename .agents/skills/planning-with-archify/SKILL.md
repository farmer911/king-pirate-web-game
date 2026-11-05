---
name: planning-with-archify
description: Use when creating, updating, reviewing, or executing any implementation plan, architecture roadmap, phase status, or system design in king-pirate.
---

# Planning with Archify & Granular Checkbox Guardrail

## Overview

Every planning or architectural update in `king-pirate` must start from `.agents/STATE.md` + **Graphify MCP** (`query_graph`, `get_node`, `get_neighbors`), propose concrete architectural options (`Gate 4`), wait for user approval when in `/plan` or answering questions (never self-approving `- [ ] -> - [x]` in the same turn without user approval), compile `docs/rewrite-plan.html` via Archify, and never push plan files to Git.

**Core principle:** Answer questions and align on options first; track every task with `- [ ]` checkboxes; verify every metric with real commands (`0` fake metrics or phantom crates); and regenerate `docs/rewrite-plan.html`.

**REQUIRED SUB-SKILL:** Use `honesty`, `empirical-metrics-and-knowledge-tools`, and `engineering-audit-gate`.

## The Iron Laws

```
1. ALIGN BEFORE EXECUTING: WHEN THE USER INVOKES /plan OR ASKS QUESTIONS, ANSWER DIRECTLY, PRESENT GATE 4 OPTIONS, AND WAIT FOR USER APPROVAL BEFORE MODIFYING PRODUCTION CODE.
2. GRAPHIFY MCP FIRST: READ .agents/STATE.md AND QUERY GRAPHIFY MCP (query_graph / get_node / get_neighbors) BEFORE READING SOURCE FILES.
3. ZERO FAKE METRICS OR PHANTOM CRATES IN PLANS: EVERY METRIC AND DEPENDENCY IN REWRITE_PLAN.md, GEMINI.md, STATE.md, AND candidate.archify.md MUST MATCH Cargo.toml AND REAL SHELL COMMANDS.
4. MANDATORY CHECKBOX GUARDRAIL: EVERY PHASE AND SUB-TASK MUST HAVE GRANULAR "- [ ]" CHECKBOXES THAT ONLY FLIP TO "- [x]" AFTER FRESH VERIFICATION PASSES.
5. NO PLAN UPDATE WITHOUT ARCHIFY & GIT ISOLATION: ALWAYS COMPILE docs/rewrite-plan.html VIA archify.mjs finalize explainer, AND NEVER PUSH PLAN FILES TO GIT.
```

## Mandatory 5-Step Planning & Execution Guardrail

### Step 1: Instant Context Bootstrap via Graphify MCP
1. Read `/Users/phucnguyen/Dev/king-pirate/.agents/STATE.md` (`<= 50` lines).
2. Query `call_mcp_tool(ServerName="graphify", ToolName="query_graph" | "get_node" | "get_neighbors")` to inspect symbols and edges without scanning raw files.

### Step 2: Propose Options (`Gate 4`) & Author Granular Checkboxes (`- [ ]`) (100% English in Repo Files)
1. If the user asked a question or invoked `/plan`, present 2–3 architectural options with measured trade-offs and wait for approval.
2. Record granular `- [ ]` checkboxes in `REWRITE_PLAN.md`, `.agents/STATE.md`, and `.archify/explainer-vht-rust-rewrite/candidate.archify.md`.
3. Only flip `- [ ]` to `- [x]` after `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test` (including `test_honesty_no_fake_metrics_no_test_only_domains_no_junk`) passes.

### Step 3: Compile Interactive Archify Explainer (`docs/rewrite-plan.html`)
```bash
cd /Users/phucnguyen/Dev/king-pirate && node /Users/phucnguyen/.gemini/config/skills/archify/bin/archify.mjs finalize explainer .archify/explainer-vht-rust-rewrite/candidate.archify.md docs/rewrite-plan.html --repo-root /Users/phucnguyen/Dev/king-pirate --json && rm -f docs/rewrite-plan.finalize*.json
```

### Step 4: Sync Second Brain
Run `bash /Users/phucnguyen/Dev/king-pirate/.agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh`.

### Step 5: Phase-Completion Git Push (Source Code Only — Never Push Plan Files)
Verify `rtk git status` confirms `REWRITE_PLAN.md`, `docs/`, `.archify/`, `.agents/STATE.md`, `graphify-out/`, and `legacy-ref/` are ignored by `.gitignore`, then push verified source code to `origin main`.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I'll add `- [ ]` to `STATE.md`, write the code, and mark `- [x]` right away when the user asked me a question" | Self-approving without answering the user first violates `honesty` Iron Law #1 and Rule 09. |
| "I can leave theoretical numbers (`5–15 µs`, `DashMap`) in `REWRITE_PLAN.md`" | Fails `test_honesty_no_fake_metrics_no_test_only_domains_no_junk`. Only real dependencies in `Cargo.toml` and measured numbers are allowed. |

## Red Flags - STOP and Fix Immediately

- Executing code changes during `/plan` before the user approves the plan.
- Writing any crate name in `REWRITE_PLAN.md` or `STATE.md` that is not in `Cargo.toml`.
- Editing `REWRITE_PLAN.md` or `.agents/STATE.md` without regenerating `docs/rewrite-plan.html` via Archify.
