---
trigger: always_on
---

# RULE #2: Second Brain (Graphify MCP + Obsidian) & RTK Token Optimization

## 1. Mandatory Graphify MCP Query Before Reading Any Source File
- The project knowledge graph and Obsidian Second Brain live at:
  - **Graphify MCP Server (`ServerName: "graphify"`):** `query_graph`, `get_node`, `get_neighbors`, `get_community`, `god_nodes`.
  - `graphify-out/GRAPH_REPORT.md` — High-level God nodes and communities.
  - `graphify-out/graph.json` — Queryable NetworkX knowledge graph (`370+` nodes, `718+` edges).
  - `graphify-out/obsidian/` — Obsidian vault with `[[wikilinks]]` and `#community/...` tags.
- **Mandatory Lookup Sequence (Never Read Full Source Files or Scan Directories First):**
  1. Read `.agents/STATE.md` first for current phase status.
  2. Call `call_mcp_tool(ServerName="graphify", ToolName="query_graph" | "get_node" | "get_neighbors")` to retrieve exact symbol coordinates (`src=... loc=L...`) and call edges (`[calls]`).
  3. Call `read_file` **ONLY** on the exact file and line window (`StartLine`..`EndLine`, `<= 120` lines) returned by Graphify MCP. Never sequentially read full files when Graphify MCP indexes the symbol.
- **Git Safety:** `graphify-out/` MUST remain in `.gitignore` and must NEVER be committed or pushed to a Git remote.

## 2. RTK (Rust Token Killer) CLI Compression
- Always prefix high-output shell commands with `rtk` (`rtk ls`, `rtk git status`, `rtk git diff`, `rtk cargo check`, `rtk cargo test`, `rtk grep`) to compress terminal output and conserve context window tokens.
