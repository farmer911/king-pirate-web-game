---
name: updating-project-knowledge
description: Use at the end of a phase, or after adding, renaming or deleting a module, rule or skill, to bring GEMINI.md, the plan and the knowledge graph back in sync.
---

# Updating Project Knowledge

Knowledge sync happens at the end of a phase, not after every file edit.

## Checklist

1. `GEMINI.md` Component Inventory: status column matches reality (done / stub / not started). Both Gemini and Claude read this file (`CLAUDE.md` imports it).
2. `REWRITE_PLAN.md`: phase checkboxes reflect verified work only.
3. `.agents/skills/porting-legacy-to-rust/SKILL.md`: Shared Primitives table lists every new shared helper.
4. Lessons: if a rule was broken or a bug escaped, add one entry to the GEMINI.md Lessons Log (where, why, prevention).
5. Graph (optional, only if graphify is installed):

```bash
bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh
```

The script uses `GRAPHIFY_PYTHON` (default `python3`). If graphify is not installed, skip it and say so.

## Finding Code

Prefer the `graphify` MCP server (`query_graph`, `get_node`, `get_neighbors`) when it is running; otherwise `rg -n` / `grep -n` and read only the matching lines.
