---
trigger: always_on
---
# RULE #5: Strict Language Policy (100% English Code/Docs/Skills, Zero Chinese, Vietnamese Only for Game Render)

## 1. 100% English in Code, Comments, Docs, Rules & Skills
- **ALL** Rust source files (`src/**/*.rs`), comments, docstrings, identifiers, test names, commit messages, `GEMINI.md`, `REWRITE_PLAN.md`, `.agents/STATE.md`, `.agents/rules/*.md`, and `.agents/skills/**/SKILL.md` MUST be written **100% in English**.

## 2. Zero Chinese (`0%` CJK Characters) Across the Entire Project
- **NO Chinese characters (`\u4e00-\u9fff`)** are permitted anywhere in `/Users/phucnguyen/Dev/king-pirate` (`src/`, `.agents/`, `GEMINI.md`, `REWRITE_PLAN.md`, `docs/`, or `legacy-ref/`).
- Automated tests (`cargo test`) scan the repository and fail immediately if any Chinese character is introduced.

## 3. Vietnamese Strictly Limited to Game UI Rendering
- **Vietnamese** is permitted **ONLY** in player-facing game rendering assets/localization strings (and in conversational replies when the user speaks Vietnamese in chat).
- **NEVER** use Vietnamese in Rust source code (`src/**/*.rs`), code comments, `.agents/rules/`, or `.agents/skills/`.
