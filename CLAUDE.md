# CLAUDE.md

Claude Code reads this file; Gemini reads `GEMINI.md`. Both agents share one knowledge base, so project facts live only in `GEMINI.md` and rules only in `.agents/`. Edit those, never duplicate them here.

@GEMINI.md
@.agents/rules/00-core.md

## Claude-Specific Notes

- **Skills are shared with Gemini** in `.agents/skills/<name>/SKILL.md`. Claude Code does not auto-load that folder: when a task matches a row of the "Which Skill to Load" table in the core rules, Read that `SKILL.md` before starting.
- **Finding code:** if the `graphify` MCP server is connected, use it; otherwise Grep, then Read only the matching lines.
- **Do not add `.claude/` rules or skills** that restate `.agents/`. One copy, or the two agents drift apart.
