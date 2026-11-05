# King Pirate (`v298`) — Project Knowledge Base & Agent Rules (`GEMINI.md`)

This document is the **Single Source of Truth (SSoT)** for the **King Pirate** project (`/Users/phucnguyen/Dev/king-pirate`), a **100% Rust (`aarch64-apple-darwin` Single-Binary Architecture + Zero Dead Code)** rewrite of the legacy `v298` game server.

---

## 1. MANDATORY RULE #1: Zero Hallucination (100% Ground-Truth Enforcement)

> **Supreme Principle:** **NEVER GUESS (No Guessing)** and **NEVER HALLUCINATE (No Hallucination)** any packet header byte, AMF3 flag, cryptographic key, damage/rage formula, or database column. Every line of Rust code and technical claim **MUST** cite (`file:line`) verified ground truth in `legacy-ref/`:

| Ground-Truth Subsystem | Verified Reference Path in `/Users/phucnguyen/Dev/king-pirate/` |
| :--- | :--- |
| **8-Byte Framing + Double AMF3 + MD5 XOR Checksum** | `legacy-ref/rpcfw/lib/PHPProxy.class.php` (`L108-L381`), `legacy-ref/rpcfw/lib/Util.class.php` (`L492-L538`) |
| **DES-CBC Encryption (`BabelCrypt`) & Auth Hash** | `legacy-ref/rpcfw/lib/BabelCrypt.class.php` (`L18-L66`), `legacy-ref/rpcfw/conf/BabelCrypt.cfg.php` (`L18-L29`) |
| **RPC Dispatch & Session Lifecycle (`RPCFramework`)** | `legacy-ref/rpcfw/lib/RPCFramework.class.php` (`L185-L800`), `legacy-ref/rpcfw/lib/ServerProxy.class.php` (`L32-L248`) |
| **Static Game Store (`BtstoreElement` + `flock`)** | `legacy-ref/rpcfw/lib/SimpleBtstore.php` (`L18-L394`) |
| **Query Builder (`CData`) & Cache (`LocalQueryCache`)** | `legacy-ref/rpcfw/lib/data/NewData.class.php` (`L58-L927`), `legacy-ref/rpcfw/lib/data/LocalQueryCache.class.php` (`L18-L411`) |
| **4 C++ Daemon Configurations (`lcserver`, `dataproxy`, `phpproxy`, `battle`)** | `legacy-ref/lcserver/conf/game20002.xml`, `legacy-ref/dataproxy/conf/config.xml`, `legacy-ref/phpproxy/conf/module.xml`, `legacy-ref/battle/conf/config.xml` |
| **Combat Engine DWARF C++ Structs (`battle`)** | `legacy-ref/dwarf/battle-dwarf-clean.hpp` (`CHero`, `CSkill`, `CFormation`, `CBuffer`, `CPet`, `CActionRecord`, `CReactionRecord`) |
| **Static Combat Tables (`battle/data`)** | `legacy-ref/battle/data/skill.csv` (`2.5 MB`), `buffer.csv` (`157 KB`), `target.csv` (`4.6 KB`) |
| **Database Schema (210 Game Tables + 3 Web Tables)** | `legacy-ref/schema/pirate20002-schema.sql` (`4,118` lines), `legacy-ref/schema/t9-rxhzw-web-schema.sql` (`82` lines) |

---

## 2. MANDATORY RULE #2: Second Brain (`Graphify + Obsidian`) & Token Optimization (`RTK`)

1. **Query Knowledge Graph (`Graphify`) & Obsidian Vault Before Reading Raw Files:**
   - Knowledge Graph report: `graphify-out/GRAPH_REPORT.md` and `graphify-out/graph.json`.
   - Obsidian Second Brain Vault: `graphify-out/obsidian/` (open directly in **Obsidian.app** for interactive Graph View with `[[wikilinks]]` and `#community/...` tags).
   - **Mandatory Lookup Sequence:** Read `.agents/STATE.md` and `graphify-out/GRAPH_REPORT.md` first -> locate the exact target file and symbol -> read only the required line range (`StartLine`..`EndLine`).
2. **Compress CLI Output with `rtk`:**
   - Always prefer `rtk ls`, `rtk git status`, `rtk git diff`, `rtk cargo check`, `rtk cargo test` over raw shell commands.
3. **Git Safety:**
   - `graphify-out/` MUST remain in `.gitignore` and must NEVER be committed or pushed to Git remote.

---

## 3. MANDATORY RULE #3: Zero Dead Code, Zero Duplication & Clean Bounded Domains (`100% Rust`)

1. **Compiler-Enforced Zero Dead Code (`rustc`):**
   - `dead_code = "deny"`, `unused_imports = "deny"`, and `unused_variables = "deny"` are enforced in `Cargo.toml` and `src/main.rs`.
2. **Zero Code Duplication (DRY Shared Primitives):**
   - Always check the **Shared Primitives Registry** in `.agents/skills/porting-legacy-to-rust/SKILL.md` before writing any helper, byte parser, or struct.
3. **Eliminate 785+ Legacy Dead Files & Collapse 7-File Ceremony into Clean Bounded Domains (`src/domain/<domain>.rs`):**
   - Never port `exlib/Smarty-3.0.8`, `exlib/phpunit`, `script/*.script.php`, `cov/`, `.bak`, or `doc/`.
   - Collapse `I<Module>` + `<Module>` + `En<Module>` + `<Module>Dao` + `<Module>Logic` + `<Module>Def` + `<Module>Conf` into `1` cohesive, strongly-typed Rust module per bounded domain (`src/domain/<domain>.rs`) for maximum `rustc` parallel codegen and maintainability (no artificial file-count cap).

---

## 4. MANDATORY RULE #4: Strict Language Policy

1. **100% English in Code, Comments, Docs, Rules & Skills:**
   - All Rust files (`src/**/*.rs`), comments, docstrings, identifiers, commit messages, `GEMINI.md`, `REWRITE_PLAN.md`, `.agents/STATE.md`, `.agents/rules/*.md`, and `.agents/skills/**/SKILL.md` MUST be written **100% in English**.
2. **Zero Chinese (`0%` CJK Characters):**
   - Zero Chinese characters (`\u4e00-\u9fff`) are permitted anywhere in `/Users/phucnguyen/Dev/king-pirate`. Automated tests in `src/protocol.rs` enforce this on every `cargo test` run.
3. **Vietnamese Reserved Strictly for Game Render UI:**
   - Vietnamese is used **ONLY** for player-facing game UI rendering strings (never in Rust source code, code comments, rules, or skills).

---

## 5. MANDATORY RULE #5: Keep-Up-To-Date (`keep-up-to-date.md`), Checkbox Guardrail & Git Phase-Push

- **Every structural or code change MUST update in the SAME turn:**
  1. `GEMINI.md` and `.agents/STATE.md` (with granular `- [ ]` / `- [x]` checkboxes for every task)
  2. The **corresponding skill** in `.agents/skills/` (e.g. registering new primitives in `porting-legacy-to-rust/SKILL.md`)
  3. The interactive Archify plan (`docs/rewrite-plan.html`) via `planning-with-archify` whenever plans/phases/checkboxes change
  4. The Graphify + Obsidian Second Brain via `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh`
  5. **Phase-by-Phase Git Push (`https://github.com/farmer911/king-pirate-web-game.git`):** Push new Rust source code after each phase passes verification; **NEVER** push plan files (`REWRITE_PLAN.md`, `docs/`, `.archify/`, `.agents/STATE.md`, `graphify-out/`, `legacy-ref/`).

---

## 6. MANDATORY RULE #6: Code-First Evidence, Empirical Verification & Zero Sycophancy (`07-evidence-based-truth-no-sycophancy.md`)

1. **Always Speak With Code and Verified Evidence (`Code-First Communication`):**
   - Never make abstract claims without citing exact source code (`file:line` with concrete snippets), binary inspection (`ldd`, `file`, `dwarfdump`), or empirical execution results (`cargo test`, `/usr/bin/time`).
   - Explicitly state the underlying computer science / systems engineering principle alongside the code proof.
2. **No Unverified Theory (`Mandatory Empirical Gate`):**
   - Every claim about performance, security, or compatibility must be backed by direct code inspection in `legacy-ref/` / `src/` or a reproducible command execution.
3. **Radical Transparency & Zero Sycophancy (`Truth Over Agreement`):**
   - Never flatter, appease, or blindly agree with the user when a premise in the prompt contains a technical misconception or factual error.
   - Explicitly separate what is **Technically TRUE** from what is **Technically FALSE / Misconceived**, backed by code evidence, and never hide trade-offs or security flaws.

---

## 7. MANDATORY RULE #7: Peak Performance & Best Outcome First (`08-peak-performance-best-outcome.md`)

1. **Supreme Priority (`Peak Performance & Engineering Excellence > Implementation Time > File Count`):**
   - Implementation time and minimizing file count are **NOT** constraints. Always propose and build the highest-performance, lowest-latency (`p99`), and most scalable architecture.
2. **Proactive Architecture Upgrade Mandate:**
   - **In-Memory Player Actor (`PlayerSessionActor` per `uid`) + Write-Behind Batch Persistence:** Keep hot player state (`User`, `Hero`, `Bag`, `Formation`, `Task`) in L1/L2 RAM (`~20–50 ns` access) and flush dirty diffs (`userModify`) asynchronously in batched transactions to MariaDB/PostgreSQL (Online Docker) or SQLite WAL (macOS Local Dev).
   - **Compile-Time / Zero-Allocation `#[repr(C)]` Static Store:** Convert `btstore` and `battle/data/*.csv` into contiguous `#[repr(C)]` tables with `O(1)` array index lookup (`0` runtime heap allocations).
   - **100% Pure Native Rust Completion:** Port all active game domains into `src/domain/*.rs` with zero legacy PHP fallback remaining in production.

---

## 8. Architecture & Component Inventory (`100% Rust`)

| Component | Technology | Path | Status |
| :--- | :--- | :--- | :--- |
| **Rust Server Crate & Wire Protocol** | **Rust 1.99 (`edition = "2024"`, `#![deny(dead_code)]`)** | `Cargo.toml`, `src/main.rs`, `src/protocol.rs` | ✅ `cargo check` & `cargo test` Pass (`4/4`) |
| **Peak-Performance Actor + Hybrid Storage** | **In-Memory `PlayerSessionActor` (`~50 ns`) + Write-Behind SQLite (Mac Dev) / MariaDB (Linux Docker Prod)** | `REWRITE_PLAN.md` (Section 5–7), `docs/rewrite-plan.html` | ✅ Forensic Code & Security Verified |
| **Forensic Investigation & Rust Plan** | **Markdown + Archify Interactive HTML** | `REWRITE_PLAN.md`, `.agents/STATE.md`, `docs/rewrite-plan.html` | ✅ 100% Ground-Truth Verified |
| **Legacy Ground-Truth Corpus** | **PHP 5.5 + XML + DWARF C++ Headers + SQL Schema** | `legacy-ref/` | ✅ Extracted & Sanitized (`0` CJK chars) |
| **Second Brain (Graphify + Obsidian)** | **NetworkX + Obsidian Markdown Vault** | `graphify-out/GRAPH_REPORT.md`, `graphify-out/obsidian/` | ✅ Initialized & Synced |
| **Agent Rules (`always_on`)** | **8 Enforced Rules** | `.agents/rules/{01..05,07-evidence-based-truth-no-sycophancy,08-peak-performance-best-outcome,keep-up-to-date}.md` | ✅ Active (`always_on`) |
| **TDD Skills (`obra/superpowers`)** | **6 TDD Skills (100% English)** | `.agents/skills/{porting-legacy-to-rust,updating-project-knowledge,planning-with-archify,test-driven-development,writing-skills,verification-before-completion}/SKILL.md` | ✅ Verified (`RED -> GREEN -> REFACTOR`) |
