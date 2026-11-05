---
trigger: always_on
---

# Rule 04: Zero Dead Code, Zero Duplication & Clean Domain Modularity (`100% Rust`)

1. **Compiler-Enforced Zero Dead Code (`rustc`):**
   - `Cargo.toml` and `src/main.rs` MUST always enforce `dead_code = "deny"`, `unused_imports = "deny"`, and `unused_variables = "deny"`.
   - Never leave commented-out code, unused struct fields, or unreferenced helper functions in the repository.

2. **Zero Code Duplication (DRY Shared Primitives):**
   - Before writing any packet parser, AMF3 encoder, crypto cipher, DB query helper, or battle formula, check `.agents/skills/porting-legacy-to-rust/SKILL.md` (**Shared Primitives Registry**).
   - If a primitive already exists in `src/protocol.rs`, `src/amf3.rs`, `src/crypt.rs`, `src/data.rs`, or `src/battle.rs`, reuse it directly.

3. **Eliminate Legacy Ceremony While Preserving Clean Domain Modularity:**
   - **Never port the `785+` dead files** identified in `legacy-ref/rpcfw` (`exlib/Smarty-3.0.8/`, `exlib/phpunit/`, `script/`, `cov/`, `.bak`, `doc/`).
   - **Collapse the 7-file Java ceremony per module** (`I<Module>`, `<Module>`, `En<Module>`, `<Module>Dao`, `<Module>Logic`, `<Module>Def`, `<Module>Conf`) into **1 cohesive, strongly-typed Rust module per bounded domain** (`src/domain/<domain>.rs`), giving both clean separation of concerns and parallel `rustc` compilation without wrapper bloat.
