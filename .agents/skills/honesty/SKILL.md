---
name: honesty
description: Use when stating a metric, naming a dependency, describing legacy behavior, reporting status, or writing any number or claim into a reply, plan, doc or comment.
---

# Honesty

Every claim the user reads must be traceable to a file, a line, or a command you ran in this session.

## The Rules

1. **Numbers come from commands.** RAM, latency, size, LOC, test counts: run the command (`wc -l`, `du -sh`, `docker stats --no-stream`, `cargo test`, `/usr/bin/time -v`) and show its output. No command, no number: write "not measured".
2. **Dependencies come from `Cargo.toml`.** Never name a crate in a doc, plan or comment unless it is in `Cargo.toml`. Planned crates go only in the `REWRITE_PLAN.md` Decisions table, marked "planned".
3. **Legacy behavior comes from `legacy-ref/`.** Cite `legacy-ref/<path>:<line>`. If you cannot find it, say `NOT FOUND IN legacy-ref: <what>`. Do not fill the gap with a plausible value.
4. **Status is literal.** "Done" means wired into the live router and verified (R3). Hard-coded or empty returns are "stub". Code reached only from `#[cfg(test)]` is "not wired".
5. **Report failures plainly.** A failing or skipped check is stated in the first lines of the reply, with its output.

## Finding Code Cheaply

- If the `graphify` MCP server is available, use `query_graph`, `get_node` or `get_neighbors` to get the file and line, then read only that window.
- If it is not available, use `grep -n` / `rg -n`, then read only the matching window. Never stop work because a tool is missing.

## Reply Style

- First sentence: the answer or the status.
- Then short bullets with `file:line` or command output as proof.
- No greetings, no recap of earlier turns, no closing filler.

## Red Flags (stop and fix)

| You are about to write | Do instead |
| :--- | :--- |
| "~50 ns", "uses 10 MB", "2x faster" without output | Run a measurement or write "not measured" |
| A crate name that is not in `Cargo.toml` | Remove it, or add it to Decisions as "planned" |
| A formula or constant without a `legacy-ref/` line | Find the line, or write NOT FOUND |
| "Feature X works" after only `cargo test` on a UI change | Run Playwright too (`playwright-e2e-evidence`) |
| "All done" while `// STUB:` handlers remain | List the stubs |
