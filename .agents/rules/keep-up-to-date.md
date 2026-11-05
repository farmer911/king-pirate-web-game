---
trigger: always_on
---
# RULE #6: Keep-Up-To-Date (Mandatory Same-Turn Synchronization of Docs, Skills, Graph & Archify Plan)

> **Non-Negotiable Principle:** Whenever you initialize, add, modify, rename, or delete ANY component, module, protocol, rule, or architectural decision in `/Users/phucnguyen/Dev/king-pirate`, you **MUST update all corresponding knowledge artifacts and skills in the SAME turn** so future sessions never suffer context drift or waste tokens rescanning files.

## Mandatory Pre-Completion Checklist (Every Structural or Feature Change)

1. **Added or Modified a Rust Module, Protocol, or Crate Dependency?**
   - Update the Component Inventory table in `GEMINI.md` and the phase checklist in `.agents/STATE.md`.
   - **Update the Corresponding Skill in `.agents/skills/`:**
     - If you added/changed a core primitive in `src/` (`protocol.rs`, `amf3.rs`, `crypt.rs`, `data.rs`, `battle.rs`), update `.agents/skills/porting-legacy-to-rust/SKILL.md` (Quick Reference / Shared Primitives table) so future agents reuse it instead of duplicating code.
     - If you changed the knowledge graph pipeline or directory layout, update `.agents/skills/updating-project-knowledge/SKILL.md`.
     - If you changed the planning workflow or phase structure, update `.agents/skills/planning-with-archify/SKILL.md`.
2. **Created or Edited Any Plan, Roadmap, or Architecture Decision?**
   - **REQUIRED SKILL:** Use `planning-with-archify` (`.agents/skills/planning-with-archify/SKILL.md`).
   - Every phase and sub-task MUST have granular `- [ ]` / `- [x]` checkboxes in `REWRITE_PLAN.md`, `.agents/STATE.md`, and `candidate.archify.md`.
   - Update `REWRITE_PLAN.md`, `.agents/STATE.md`, AND regenerate the standalone interactive Archify page at `docs/rewrite-plan.html` (`archify.mjs finalize explainer ...`).
3. **Added, Renamed, Modified, or Deleted Any File?**
   - **REQUIRED SKILL:** Use `updating-project-knowledge` (`.agents/skills/updating-project-knowledge/SKILL.md`).
   - Run `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` to incrementally update `graphify-out/graph.json`, `graphify-out/GRAPH_REPORT.md`, and the Obsidian Second Brain vault (`graphify-out/obsidian/`).
4. **Verify Language & Compiler Gates Before Claiming Completion:**
   - Run `rtk cargo check && rtk cargo test` and confirm all unit, differential, and language-policy tests pass with `0 warnings` and `0 errors`.
5. **Completed a Phase? Push Only Source Code to Git (`https://github.com/farmer911/king-pirate-web-game.git`):**
   - Flip completed phase checkboxes from `- [ ]` to `- [x]`.
   - Never stage, commit, or push plan/local files (`REWRITE_PLAN.md`, `docs/`, `.archify/`, `.agents/STATE.md`, `graphify-out/`, `legacy-ref/`).
   - Commit and push the verified phase source code to `origin main`.
