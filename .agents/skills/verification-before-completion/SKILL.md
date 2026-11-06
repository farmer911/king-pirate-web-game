---
name: verification-before-completion
description: Use when about to claim work is complete, fixed, or passing, before committing, pushing, or finishing a turn, or when tempted to say "should work", "done", or "looks good"
---

# Verification Before Completion

**Iron law:** no completion claim without a verification command run after your last edit, in this turn.

## The Gate

1. Identify the command that proves the claim (table below).
2. Run it fresh, after the final edit.
3. Read the whole output and the exit code.
4. Claim only what the output shows. If it failed, report the failure first.

## King Pirate Verification Matrix

| Change | Command | Evidence to paste |
| :--- | :--- | :--- |
| Any `src/**/*.rs` or `Cargo.toml` | `cargo test` | `test result: ok. N passed; 0 failed` |
| Before starting the server | `cargo build` | build finished (a stale `target/debug/king-pirate` caused a real outage, see GEMINI.md Lessons) |
| `web/**` or Ruffle shell | `cd web && npx playwright test` against a running `cargo run` | `N passed` |
| `Dockerfile` / `docker-compose.yml` | `docker compose up -d --build` then `docker compose ps` and `docker stats --no-stream` | container `running`, RAM column |
| Ported RPC (parity) | the module's differential test against recorded legacy traffic | test name + pass line |
| Rule or skill edit | `cargo test` (the policy tests in `src/protocol.rs` read `.agents/`) | pass line |

## Not Sufficient

| Claim | Not enough |
| :--- | :--- |
| Tests pass | A run from before the last edit; "should pass" |
| Server works | `cargo test` only (it does not rebuild `target/debug/king-pirate`) |
| UI works | Unit tests only |
| Subagent finished | Its report without checking `git diff` |

## Red Flags

- Words like "should", "probably", "seems to" next to a status.
- "Done!" before any command output appears in the reply.
- Editing a file after the test run and not re-running.
