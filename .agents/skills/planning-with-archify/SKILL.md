---
name: planning-with-archify
description: Use when creating or updating REWRITE_PLAN.md, a phase, a checklist, or an architecture decision in king-pirate.
---

# Planning

`REWRITE_PLAN.md` is the single plan. It is local-only (git-ignored) and is the place where phases, checkboxes, decisions and the stub list live.

## Rules

1. **Plan, then stop.** When the user asks for a plan, write or update `REWRITE_PLAN.md`, summarize it in chat, and wait for approval before coding (core rule R2).
2. **Checkboxes are evidence.** Every task is `- [ ]`. It becomes `- [x]` only after its verification command passed in the same turn (core rule R3). Stubs stay `- [ ]`.
3. **Decisions table.** Every architecture choice and every crate (planned or used) is a row: decision, reason, status (`planned` / `in Cargo.toml`).
4. **No invented numbers.** Targets are written as targets ("goal: under 100 MB RSS"), never as results, until measured (`honesty`).
5. **Options for big changes.** For a change that alters architecture, list 2-3 options with trade-offs and let the user choose.

## Optional: Archify Diagram

If the Archify tool is installed (`ARCHIFY` env var points to `archify.mjs`), regenerate the HTML view after a plan change:

```bash
node "$ARCHIFY" finalize explainer .archify/explainer-vht-rust-rewrite/candidate.archify.md docs/rewrite-plan.html --repo-root . --json
```

If it is not installed, skip it and say so. A missing diagram tool never blocks planning.
