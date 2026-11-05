---
name: verification-before-completion
description: Use when about to claim work is complete, fixed, or passing, before committing, pushing, or finishing a turn, or when tempted to say "should work", "done", or "looks good"
---

# Verification Before Completion

## Overview

**Core principle:** Evidence before claims, always.

**Violating the letter of this rule is violating the spirit of this rule.**

## The Iron Law

```
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

If you haven't run the verification command **after your last file edit** in this turn, you cannot claim it passes or is complete.

## The Gate Function

```
BEFORE claiming any status or expressing satisfaction:

1. IDENTIFY: What command proves this claim?
2. RUN: Execute the FULL command (fresh, after the final edit)
3. READ: Full output, check exit code, count failures
4. VERIFY: Does output confirm the claim?
   - If NO: State actual status with evidence (and trigger Continuous Learning Loop in GEMINI.md)
   - If YES: State claim WITH evidence
5. ONLY THEN: Make the claim

Skip any step = lying, not verifying
```

## RentalCar Project Verification Matrix

In the **RentalCar** monorepo, every completion claim requires fresh evidence from the matching gates below:

| Change Scope | Mandatory Verification Command(s) | Required Evidence in Output |
|--------------|-----------------------------------|-----------------------------|
| **Any `.ts` / `.tsx` edit (`be/`, `fe/`, `mobile/`, `e2e/`)** | `rtk bun run typecheck` | Exit code `0`, `0` TypeScript errors |
| **BE Gateway, Modules, DB, or SSR SEO (`be/`, `fe/server.ts`)** | `rtk bun run test:api` | `0 fail`, all tests green |
| **FE UI flows, React Router, Zustand, React Hook Form (`fe/src/`)** | `rtk bun run test:api` (and `rtk bun run test:e2e` for E2E flows) | All assertions pass, exit code `0` |
| **New/changed file, module, route, table, stack, or skill/rule** | Check `GEMINI.md` + run `graphify update .` | `GEMINI.md` updated in same turn + `graphify-out/graph.json` rebuilt |

## Common Failures

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| Tests pass | `rtk bun run test:api` output: `0 fail` | Previous run before last edit, "should pass" |
| TypeScript clean | `rtk bun run typecheck` output: exit `0` | IDE syntax look, partial file check |
| Bug fixed | Failing test reproduced first (RED) -> now passes (GREEN) | Code changed, assumed fixed |
| Knowledge synced | `GEMINI.md` edited + `graphify update .` output | "I'll update docs next turn" |
| Agent completed | `rtk git status` / `rtk git diff` confirms changes | Subagent reports "success" |

## Red Flags - STOP

- Using "should", "probably", "seems to"
- Expressing satisfaction before verification ("Great!", "Perfect!", "Done!", etc.)
- Editing `GEMINI.md` or code *after* running tests and not re-running verification
- About to commit/push/PR without fresh verification
- Trusting subagent success reports without checking files/diff
- Thinking "it's a one-line change, no need to run `typecheck`"
- **ANY wording implying success without having run verification after the last edit**

## Rationalization Prevention

| Excuse | Reality |
|--------|---------|
| "Should work now" | RUN the verification command (`rtk bun run typecheck && rtk bun run test:api`) |
| "I only fixed a tiny typo after the test ran" | Any edit after the test invalidates the test run. Re-run it. |
| "I'm confident" | Confidence ≠ evidence |
| "Just this once" | No exceptions |
| "`tsc --noEmit` passed so tests pass" | Typechecker ≠ runtime behavior. Run `rtk bun run test:api`. |
| "Agent said success" | Verify independently with `rtk git diff` and test commands |
| "Different words so rule doesn't apply" | Spirit over letter |

## Key Patterns

**Code & API changes:**
```
✅ [Run: rtk bun run typecheck && rtk bun run test:api] [See: 5 pass, 0 fail] "All typechecks and API/SSR tests pass"
❌ "Should pass now" / "Looks correct"
```

**Regression tests (TDD Red-Green):**
```
✅ Write test → Run (MUST FAIL) → Implement fix → Run (PASS)
❌ "I've added a test and fixed it" (without watching RED first)
```

**Knowledge Synchronization (`Keep-Up-To-Date`):**
```
✅ Update GEMINI.md → Run `graphify update .` → Run `rtk bun run typecheck && rtk bun run test:api`
❌ Finish code changes and forget to sync GEMINI.md or graphify-out/
```
