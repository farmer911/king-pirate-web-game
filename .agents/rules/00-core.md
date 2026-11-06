---
trigger: always_on
---

# King Pirate - Core Rules (the only always-on rule file)

These rules are short on purpose. Every rule below is either checked by `cargo test` or easy to self-check.
If two instructions ever conflict, the **Priority Order** decides. Nothing else in `.agents/` overrides this file.

## Priority Order (highest wins)

1. The user's latest message in chat.
2. Safety: never commit secrets, `legacy-ref/`, `client-assets/` or `data/`; never force-push.
3. Legacy parity: behavior must match `legacy-ref/` (R1).
4. The verification gate passes (R3).
5. Low RAM, fast startup, one-command Docker (the product goal).
6. Simplicity: shortest correct change, no dead code.
7. Documentation and knowledge sync (phase end only).

## Hard Rules

- **R1 Ground truth.** Protocol bytes, crypto, formulas, constants, table columns and RPC return shapes come from `legacy-ref/` and are cited as `legacy-ref/<path>:<line>` in a doc comment. If the source is not in `legacy-ref/`, write `NOT FOUND IN legacy-ref: <what>` in your reply and stop. Never invent a value.
- **R2 Answer first.** If the user asks a question or asks for a plan, answer or plan, then stop. Edit production code only after the user says to proceed (e.g. "ok", "do it", "implement").
- **R3 Verify before claiming.** After your last edit, run `cargo test` (plus `cd web && npx playwright test` when `web/` or the Ruffle shell changed). Paste the final summary line. No run, no claim. If it fails, say it failed.
- **R4 No invented numbers.** Any number about speed, RAM, size, LOC or coverage must come from a command run in this session, shown in the reply. Otherwise write "not measured".
- **R5 No disguised stubs.** A handler that returns hard-coded or empty data gets a `// STUB:` comment and stays `- [ ]` in `REWRITE_PLAN.md`. Never mark a stub as done.
- **R6 Language.** English in `src/`, docs, rules and skills. Vietnamese only in player-facing UI strings. `legacy-ref/` is a verbatim, read-only copy of the legacy server and keeps its original (including Chinese) text and line numbers.
- **R7 Dependencies.** A crate may be used only if it is in `Cargo.toml` and listed in the Decisions table of `REWRITE_PLAN.md`. Never mention a crate in docs that is not in `Cargo.toml`.
- **R8 Reuse.** Before writing a helper, check the Shared Primitives table in `.agents/skills/porting-legacy-to-rust/SKILL.md`. Add new shared helpers to that table in the same change.
- **R9 Git.** Commit and push only verified work (R3 passed), only source code, at the end of a phase or when the user asks.

## Which Skill to Load (load only the matching one)

| Task | Skill |
| :--- | :--- |
| Port or fix a Rust module, RPC handler, formula | `porting-legacy-to-rust` |
| Write a test first / fix a bug | `test-driven-development` |
| Change `web/` or the Ruffle game shell | `playwright-e2e-evidence` |
| About to say "done", "fixed", "passing" | `verification-before-completion` |
| Any claim about metrics, deps or legacy behavior | `honesty` |
| Create or update a plan / phase | `planning-with-archify` |
| End of a phase (sync docs and graph) | `updating-project-knowledge` |
| Edit a skill | `writing-skills` |

## Self-Check Before Ending Any Turn That Edited Files

- [ ] R2: did the user ask me to change code? (if not, revert and answer instead)
- [ ] R3: I ran `cargo test` after my last edit and pasted the result.
- [ ] R1/R4: every value and number I added has a `legacy-ref/` citation or a command output.
- [ ] R5: every hard-coded handler I touched is marked `// STUB:` and unchecked in the plan.
