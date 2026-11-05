---
name: updating-project-knowledge
description: Use when starting any task to locate files without scanning the repository, or after adding, modifying, renaming, or deleting any code, rule, skill, or architecture file in king-pirate.
---

# Updating Project Knowledge

## Overview

`king-pirate` uses a local **Graphify MCP + Obsidian Second Brain** (`graphify-out/graph.json`, `graphify-out/GRAPH_REPORT.md`, `graphify-out/obsidian/`) plus `.agents/STATE.md` and `GEMINI.md` so agents locate exact functions and call edges in `< 100` tokens instead of reading full files.

**Core principle:** Query the `graphify` MCP server first to save tokens; update the graph, `GEMINI.md`, `STATE.md`, and the corresponding skill in the same turn whenever anything changes.

## The Iron Laws

```
1. NEVER READ FULL SOURCE FILES OR SCAN DIRECTORIES BEFORE QUERYING THE GRAPHIFY MCP SERVER (query_graph, get_node, get_neighbors).
2. NEVER END A TURN THAT MODIFIED FILES WITHOUT SYNCING GEMINI.md, STATE.md, CORRESPONDING SKILLS, AND GRAPHIFY + OBSIDIAN.
```

## Step 1: Zero-Scan Context Lookup via Graphify MCP (At Task Start)

1. Read `.agents/STATE.md` (`<= 50` lines) for current phase status and active architecture.
2. Call `call_mcp_tool(ServerName="graphify", ToolName="query_graph" | "get_node" | "get_neighbors")` with `project_path="/Users/phucnguyen/Dev/king-pirate"` to get exact `src=... loc=L...` coordinates and `[calls]` edges.
3. Call `read_file` **ONLY** on the target file and line range (`StartLine`..`EndLine`, `<= 120` lines). Never read 10–20 full files sequentially.

## Step 2: Same-Turn Knowledge & Skill Synchronization (Before Task Completion)

Whenever you add, modify, rename, or delete any file in `/Users/phucnguyen/Dev/king-pirate`:

1. **Update `.agents/STATE.md` & `GEMINI.md`:** Keep the component table, active rules (`01..05`, `07..09`, `keep-up-to-date.md`), and checklist accurate (100% English, 0 fake metrics or phantom crates).
2. **Update the Corresponding Skill in `.agents/skills/`:**
   - Added/changed Rust code or shared helpers in `src/`? Update the **Shared Primitives Registry** in `.agents/skills/porting-legacy-to-rust/SKILL.md`.
   - Added/changed planning phases or architecture? Update `.agents/skills/planning-with-archify/SKILL.md` and run Archify.
3. **Run the Incremental Graphify + Obsidian Sync Script:**
   ```bash
   bash /Users/phucnguyen/Dev/king-pirate/.agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh
   ```

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I'll just `read_file` across `src/domain/*.rs` and `web/src/App.tsx` directly" | Reading full source files wastes thousands of tokens. One `graphify` MCP call (`get_neighbors` or `query_graph`) returns exact line numbers and edges in `< 100` tokens. |
| "I only added one small Rust function, no need to update skills or Graphify" | Unregistered functions become invisible to `graphify` MCP queries in future turns. Always run `sync-knowledge.sh`. |

## Red Flags - STOP and Fix Immediately

- Calling `read_file` on source files without querying `graphify` MCP (`query_graph`, `get_node`, `get_neighbors`) first.
- Claiming task completion without running `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh`.
