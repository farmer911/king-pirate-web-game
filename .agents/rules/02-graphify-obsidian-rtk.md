---
trigger: always_on
---
# RULE #2: Second Brain (Graphify + Obsidian) & RTK Token Optimization

## 1. Query Second Brain Before Reading Raw Files
- The project knowledge graph and Obsidian Second Brain live at:
  - `graphify-out/GRAPH_REPORT.md` — God nodes, communities, and architectural relationships.
  - `graphify-out/graph.json` — Queryable NetworkX knowledge graph.
  - `graphify-out/obsidian/` — Obsidian vault with `[[wikilinks]]` and `#community/...` tags.
- **Mandatory Lookup Sequence (Never Scan Raw Directories First):**
  1. Read `.agents/STATE.md` and `graphify-out/GRAPH_REPORT.md` first.
  2. Read only the exact file and line range (`StartLine`..`EndLine`) identified by the graph.
- **Git Safety:** `graphify-out/` MUST remain in `.gitignore` and must NEVER be committed or pushed to a Git remote.

## 2. RTK (Rust Token Killer) CLI Compression
- Always prefix high-output shell commands with `rtk` (`rtk ls`, `rtk git status`, `rtk git diff`, `rtk cargo check`, `rtk cargo test`, `rtk grep`) to compress terminal output and conserve context window tokens.
