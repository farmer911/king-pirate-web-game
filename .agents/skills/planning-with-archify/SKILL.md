---
name: planning-with-archify
description: Use when creating, updating, reviewing, or executing any implementation plan, architecture roadmap, phase status, or system design in king-pirate.
---

# Planning with Archify & Granular Checkbox Guardrail

## Overview

Every planning or architectural update in `king-pirate` must start from the compact Second Brain state (`.agents/STATE.md` + `graphify-out/GRAPH_REPORT.md`), define **granular, actionable Markdown checkboxes (`- [ ]` / `- [x]`) for every sub-task in every phase**, compile a validated standalone interactive Archify page (`docs/rewrite-plan.html`), and enforce that **plan files are never pushed to Git** while completed source-code phases are pushed to `https://github.com/farmer911/king-pirate-web-game.git`.

**Core principle:** Never plan without actionable `- [ ]` checkboxes for the agent to follow step-by-step, never update a plan without regenerating `docs/rewrite-plan.html` via Archify, and never push plan files to Git.

## The Iron Laws

```
1. ZERO CONTEXT RELOAD: READ .agents/STATE.md AND graphify-out/GRAPH_REPORT.md FIRST BEFORE ASKING QUESTIONS OR SCANNING FILES.
2. PEAK PERFORMANCE FIRST (RULE 08): ALWAYS PROPOSE THE HIGHEST-PERFORMANCE, BEST-OUTCOME ARCHITECTURE REGARDLESS OF IMPLEMENTATION TIME OR FILE COUNT.
3. MANDATORY CHECKBOX GUARDRAIL: EVERY PHASE AND SUB-TASK IN REWRITE_PLAN.md, .agents/STATE.md, AND candidate.archify.md MUST HAVE GRANULAR "- [ ]" CHECKBOXES THAT THE AGENT FOLLOWS AND MARKS "- [x]" UPON VERIFIED COMPLETION.
4. NO PLAN UPDATE WITHOUT ARCHIFY: EVERY CHANGE TO THE PLAN OR CHECKBOX STATUS MUST UPDATE .agents/STATE.md, REWRITE_PLAN.md, AND docs/rewrite-plan.html VIA archify.mjs finalize explainer.
5. GIT PHASE-PUSH & ZERO-PLAN-ON-GIT: NEVER COMMIT OR PUSH PLAN FILES (REWRITE_PLAN.md, docs/, .archify/, .agents/STATE.md, graphify-out/, legacy-ref/). PUSH ONLY NEW SOURCE CODE AFTER EACH PHASE PASSES VERIFICATION.
```

## Mandatory 5-Step Planning & Execution Guardrail

### Step 1: Instant Context Bootstrap (Zero User Re-Prompting)
1. Read `/Users/phucnguyen/Dev/king-pirate/.agents/STATE.md` (`<= 50` lines) and `/Users/phucnguyen/Dev/king-pirate/GEMINI.md`.
2. Read `/Users/phucnguyen/Dev/king-pirate/graphify-out/GRAPH_REPORT.md` if module relationships are needed.
3. Never ask the user to repeat architectural decisions already recorded in `STATE.md` or `GEMINI.md`.

### Step 2: Author Granular Checkboxes (`- [ ]`) in Plan Files (100% English)
1. Every phase in `/Users/phucnguyen/Dev/king-pirate/REWRITE_PLAN.md`, `/Users/phucnguyen/Dev/king-pirate/.agents/STATE.md`, and `/Users/phucnguyen/Dev/king-pirate/.archify/explainer-vht-rust-rewrite/candidate.archify.md` MUST break work down into concrete, verifiable `- [ ]` checkboxes:
   - `[ ]` Target `.rs` module & ground-truth citation (`file:line` in `legacy-ref/`)
   - `[ ]` Unit / differential test (`cargo test` RED -> GREEN)
   - `[ ]` Registration in `porting-legacy-to-rust/SKILL.md` Shared Primitives Registry
   - `[ ]` Phase verification & Git push (`git push origin main`)
2. As each task completes and passes `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test`, immediately flip `- [ ]` to `- [x]`.

### Step 3: Compile Interactive Archify Explainer (`docs/rewrite-plan.html`)
1. Update `/Users/phucnguyen/Dev/king-pirate/.archify/explainer-vht-rust-rewrite/candidate.archify.md` (including the phase checkbox list at the bottom of the explainer).
2. Run Archify `finalize explainer`:
   ```bash
   cd /Users/phucnguyen/Dev/king-pirate && node /Users/phucnguyen/.gemini/config/skills/archify/bin/archify.mjs finalize explainer .archify/explainer-vht-rust-rewrite/candidate.archify.md docs/rewrite-plan.html --repo-root /Users/phucnguyen/Dev/king-pirate --json && rm -f docs/rewrite-plan.finalize*.json
   ```
3. Verify `"ok": true` and `"diagnostics": []` (`0 errors, 0 warnings`).

### Step 4: Sync Second Brain
Run `bash /Users/phucnguyen/Dev/king-pirate/.agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` so the updated plan and Archify artifact are linked in `graphify-out/obsidian/`.

### Step 5: Phase-Completion Git Push (Source Code Only — Never Push Plan Files)
Whenever a phase is completed and all its `- [x]` checkboxes and `cargo test` suites pass:
1. Verify `rtk git status` confirms `REWRITE_PLAN.md`, `docs/`, `.archify/`, `.agents/STATE.md`, `graphify-out/`, and `legacy-ref/` are ignored by `.gitignore`.
2. Commit and push the new source code for that phase to `https://github.com/farmer911/king-pirate-web-game.git`.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I'll write a high-level prose paragraph for Phase 2 instead of `- [ ]` checkboxes" | Prose paragraphs cause agents to skip sub-tasks or verification steps. Every deliverable, module, and test MUST be a `- [ ]` checkbox. |
| "Running `archify` CLI is unnecessary overhead for a quick checkbox update" | Skipping `archify` leaves `docs/rewrite-plan.html` out of sync with `REWRITE_PLAN.md`. `archify.mjs finalize explainer` takes `< 0.6s`. Run it every time. |
| "I'll commit `REWRITE_PLAN.md` or `docs/rewrite-plan.html` to Git so it's backed up" | **Forbidden.** The user explicitly mandated never pushing plan files to Git. Keep them local and listed in `.gitignore`. |

## Red Flags - STOP and Fix Immediately

- Writing any phase in `REWRITE_PLAN.md`, `.agents/STATE.md`, or `candidate.archify.md` without granular `- [ ]` checkboxes.
- Starting or finishing a phase without updating its `- [ ]` -> `- [x]` checkboxes.
- Editing `REWRITE_PLAN.md` or `.agents/STATE.md` without running `archify.mjs finalize explainer` to update `docs/rewrite-plan.html`.
- Staging or pushing `REWRITE_PLAN.md`, `docs/`, `.archify/`, `.agents/STATE.md`, `graphify-out/`, or `legacy-ref/` to Git.
