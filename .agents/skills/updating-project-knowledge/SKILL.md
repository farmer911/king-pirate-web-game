---
name: updating-project-knowledge
description: Use when starting any task to locate files without scanning the repository, or after adding, modifying, renaming, or deleting any code, rule, skill, or architecture file in king-pirate.
---

# Updating Project Knowledge

## Overview

`king-pirate` uses a local **Graphify + Obsidian Second Brain** (`graphify-out/GRAPH_REPORT.md`, `graphify-out/graph.json`, `graphify-out/obsidian/`) plus `.agents/STATE.md` and `GEMINI.md` so agents never waste tokens scanning hundreds of legacy files and never lose context between sessions.

**Core principle:** Read the graph first to save tokens; update the graph, `GEMINI.md`, `STATE.md`, and the corresponding skill in the same turn whenever anything changes.

## The Iron Laws

```
1. NEVER SCAN RAW DIRECTORIES BEFORE READING .agents/STATE.md AND graphify-out/GRAPH_REPORT.md.
2. NEVER END A TURN THAT MODIFIED FILES WITHOUT SYNCING GEMINI.md, STATE.md, CORRESPONDING SKILLS, AND GRAPHIFY + OBSIDIAN.
```

## Step 1: Zero-Scan Context Lookup (At Task Start)

1. Read `.agents/STATE.md` (`<= 50` lines) for current phase status and active architecture.
2. Read `graphify-out/GRAPH_REPORT.md` (Community 10, Community 45, and God Nodes) to identify exact file paths and relationships.
3. Read only the target file and line range (`StartLine`..`EndLine`). Do **not** run broad `find` or multi-file `grep` across `legacy-ref/` unless a symbol is absent from the graph.

## Step 2: Same-Turn Knowledge & Skill Synchronization (Before Task Completion)

Whenever you add, modify, rename, or delete any file in `/Users/phucnguyen/Dev/king-pirate`:

1. **Update `.agents/STATE.md` & `GEMINI.md`:** Keep the component table, active rules (`01..05`, `07-evidence-based-truth-no-sycophancy.md`, `08-peak-performance-best-outcome.md`, `keep-up-to-date.md`), and checklist accurate (100% English).
2. **Update the Corresponding Skill in `.agents/skills/`:**
   - Added/changed Rust code or shared helpers in `src/`? Update the **Shared Primitives Registry** in `.agents/skills/porting-legacy-to-rust/SKILL.md`.
   - Added/changed planning phases or architecture? Update `.agents/skills/planning-with-archify/SKILL.md` and run Archify.
   - Added/changed graph nodes or scripts? Update `.agents/skills/updating-project-knowledge/SKILL.md`.
3. **Run the Incremental Graphify + Obsidian Sync Script:**
   ```bash
   bash /Users/phucnguyen/Dev/king-pirate/.agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh
   ```
   This script uses cached AST hashes (`graphify-out/cache/`) so only modified files are re-parsed, updating `graphify-out/graph.json`, `graphify-out/GRAPH_REPORT.md`, and `graphify-out/obsidian/` in ~2 seconds with **zero LLM token cost**.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I'm in a 30-second rush, I'll just `ls` and read `src/` directly" | Reading `.agents/STATE.md` takes 1 tool call (40 lines) and prevents duplicating code or missing ground-truth citations. |
| "I only added one small Rust file (`src/crypt.rs`), no need to update skills or Graphify" | Unregistered modules become invisible to future sessions, causing duplicate implementations. Run `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` and update `porting-legacy-to-rust/SKILL.md`. |
| "I'll commit `graphify-out/` to Git so it's saved" | **Forbidden.** `graphify-out/` is local-only and must stay in `.gitignore`. |

## Red Flags - STOP and Fix Immediately

- Running `grep` or reading multiple files in `legacy-ref/` before reading `.agents/STATE.md` and `graphify-out/GRAPH_REPORT.md`.
- Adding a new file/module without updating `GEMINI.md`, `.agents/STATE.md`, and the corresponding skill in `.agents/skills/`.
- Claiming task completion without running `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh`.
