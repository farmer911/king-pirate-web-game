---
name: porting-legacy-to-rust
description: Use when implementing, porting, refactoring, or fixing any Rust module, protocol handler, database query, or combat formula in king-pirate, before writing implementation code.
---

# Porting Legacy to Rust

## Overview

Porting legacy `C++03` daemons (`lcserver`, `phpproxy`, `dataproxy`, `battle`) and `PHP 5.5` (`rpcfw`) into `king-pirate` requires **100% ground-truth fidelity, zero code duplication, zero dead code, and 100% English source code**.

**Core principle:**Violating the letter of these rules is violating the spirit of these rules.

**REQUIRED SUB-SKILL:** Use `test-driven-development` and `verification-before-completion`.

## The Iron Laws

```
1. NO RUST CODE WITHOUT A LEGACY GROUND-TRUTH CITATION (file:line) AND A FAILING TEST FIRST.
2. PEAK PERFORMANCE FIRST (RULE 08): PRIORITIZE MAXIMUM THROUGHPUT, L1/L2 CACHE LOCALITY, AND ZERO ALLOCATIONS OVER CODING TIME OR ARTIFICIAL FILE-COUNT CAPS.
3. ZERO DUPLICATION & ZERO DEAD CODE: REUSE SHARED PRIMITIVES IN src/ AND ENFORCE #![deny(dead_code)].
4. CLEAN DOMAIN MODULARITY: COLLAPSE 7-FILE PHP CEREMONY INTO 1 COHESIVE .rs MODULE PER BOUNDED DOMAIN.
5. 100% ENGLISH IN CODE/COMMENTS: ZERO CHINESE ANYWHERE; VIETNAMESE ONLY IN GAME RENDER UI.
6. ADDED A NEW MODULE OR HELPER? UPDATE THE SHARED PRIMITIVES TABLE IN THIS SKILL IMMEDIATELY.
```

## Shared Primitives Registry (Check Before Writing Any Code)

Before writing any function, struct, or constant, check this table. If it already exists, import and reuse it. When you add a new shared function or module, **you MUST add it to this table in the same turn**.

| Module | File | Provided Primitives (Reuse — Never Duplicate) | Ground-Truth Source |
| :--- | :--- | :--- | :--- |
| **Wire Protocol** | `src/protocol.rs` | `HEAD_SIZE`, `AMF3_PREFIX`, `FLAG_COMPRESSED`, `FLAG_ENCRYPTED`, `FLAG_UNWRAP_RET`, `MESS_CODE_PRIMARY`, `MESS_CODE_SECONDARY`, `BABEL_CRYPT_KEY`, `BABEL_CRYPT_IV`, `PLAY_HASH_KEY`, `FrameHeader::{encode,decode}`, `build_xor_checksum_buffer`, `pack_pid_bytes`, `unpack_pid_bytes` | `PHPProxy.class.php:108-381`, `BabelCrypt.class.php:18-66`, `game20002.xml:51-55` |
| **AMF3 Codec** | `src/amf3.rs` | `Amf3Value::{assoc_map,dense_list}`, `amf3_encode`, `amf3_decode` (`obj_refs` + `trait_refs`), `write_u29`, `read_u29`, `write_amf3_str` | `Util.class.php:492-538`, `PHPProxy.class.php:185-269` |
| **Crypto & Auth Guard** | `src/crypt.rs` | `md5_digest`, `md5_hex`, `sha1_digest`, `base64_encode`, `websocket_accept_key`, `phpproxy_checksum`, `babel_encrypt_number`, `babel_decrypt_number`, `compute_login_play_hash`, `verify_login_request` | `BabelCrypt.class.php:18-66`, `UserLogic.class.php:23-82`, `User.class.php:47-64`, RFC 6455 |
| **In-Memory Player Actor** | `src/actor.rs` | `UPPER_LIMIT_BELLY`, `PlayerSessionActor`, `PlayerDirtyDelta`, `add_belly`, `add_gold`, `add_prestige`, `add_execution`, `add_fight_cdtime`, `move_in_town`, `take_dirty_delta` | `OtherUserObj.class.php:53-55, 108-130, 447-650` |
| **Dual-Mode Data Store** | `src/data.rs` | `SqlDialect::{SqliteWal,MariaDbInnoDb}`, `ColumnValue`, `UpdateOp`, `WhereCond`, `build_upsert_sql`, `IdGenerator::next_id`, `DataStore::{insert_or_update,select,select_count,update,delete,apply_write_behind_delta,save_to_disk,load_from_disk}` | `NewData.class.php:18-48, 748-988`, `Framework.cfg.php:40` |
| **RPC & WebSocket Gateway** | `src/gateway.rs` | `FLASH_POLICY_REQUEST`, `FLASH_POLICY_RESPONSE`, `ConnectionSession`, `MethodAccessTier`, `classify_method`, `authorize_rpc_call`, `encode_framed_packet`, `decode_framed_packet`, `encode_babeltime_response`, `parse_websocket_key`, `read_ws_frame`, `write_ws_frame`, `render_game_launcher_html`, `handle_http_stream` | `game20002.xml:39-100`, `RPCFramework.class.php:348-404, 783-865`, `Framework.cfg.php:203-411`, `Main.swf` (`BabelTimeSocket`) |
| **Static Game Store** | `src/btstore.rs` | `RawSkill`, `RawBuffer`, `RawTarget`, `StaticBtstore::{new_baseline,load_skills_csv,get_skill,get_buffer,get_target}` | `battle-dwarf-clean.hpp:243-244, 318`, `skill.csv`, `buffer.csv`, `target.csv` |
| **Combat Engine** | `src/battle.rs` | `HeroUnit`, `FormationGrid`, `ReactionRecord`, `ActionRecord`, `BattleResult::{to_amf3}`, `compute_skill_damage`, `execute_battle_do_hero` | `battle-dwarf-clean.hpp:1-386`, `Battle.class.php:81-93, 128-134` |
| **Bounded Domains** | `src/domain/*.rs` | `DomainContext`, `dispatch_domain_rpc`, `dispatch_client_rpc`, `user::*`, `hero::*`, `bag::*`, `forge::*`, `copy::*`, `arena::*`, `boss::*`, `guild::*`, `town::*`, `pet::*`, `economy::*` | `Framework.cfg.php:279-411`, `User.class.php:34-650` |
| **Entrypoint** | `src/main.rs` | Multi-protocol daemon (`./target/debug/king-pirate serve` on HTTP `:8081`, WS `:8003`, TCP `:8002`) & compile-time lint gate (`#![deny(warnings, dead_code, unused_imports, unused_variables)]`) | `start_all.sh` |

## Mandatory 5-Step Porting Workflow

### 1. Lookup Ground Truth via Second Brain (Zero Raw Scanning)
1. Read `.agents/STATE.md` and `graphify-out/GRAPH_REPORT.md` first (follow `updating-project-knowledge`).
2. Read the exact reference lines in `legacy-ref/` (`PHPProxy.class.php`, `RPCFramework.class.php`, `NewData.class.php`, `BattleLogic.class.php`, `battle-dwarf-clean.hpp`, `pirate20002-schema.sql`).
3. Check the **Shared Primitives Registry** above to avoid re-implementing existing logic.

### 2. Write Failing Unit / Differential Test (`RED`)
1. Add a `#[test]` inside the target `.rs` file's `#[cfg(test)] mod tests` block using real byte vectors or struct states from `legacy-ref/`.
2. Run `rtk ~/.cargo/bin/cargo test` and watch it fail.

### 3. Implement Minimal, Consolidated Rust Code (`GREEN`)
1. Collapse legacy PHP 7-file ceremony (`I<Mod>`, `<Mod>`, `En<Mod>`, `<Mod>Dao`, `<Mod>Logic`, `<Mod>Def`, `<Mod>Conf`) into **one `.rs` file**.
2. Write **100% English** identifiers, doc comments (`///`), and inline comments (`//`). Never copy Chinese comments (`\u4e00-\u9fff`) from `legacy-ref/` and never write Vietnamese in `.rs` files, even if the prompt includes them.
3. Run `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test` and verify 0 errors and 0 warnings.

### 4. Update This Skill's Shared Primitives Registry
If you added a new module, struct, or reusable helper in `src/`, update the **Shared Primitives Registry** table in `.agents/skills/porting-legacy-to-rust/SKILL.md` immediately.

### 5. Sync Knowledge Graph
Run `bash .agents/skills/updating-project-knowledge/scripts/sync-knowledge.sh` and update `GEMINI.md` + `.agents/STATE.md`.

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "I'll just scan `src/` directly instead of reading `GRAPH_REPORT.md`" | Scanning burns tokens and misses `legacy-ref/` citations. Always read `.agents/STATE.md` + `GRAPH_REPORT.md` first. |
| "The user or legacy PHP file included a Chinese/Vietnamese comment, so I'll paste it into `src/`" | **Forbidden.** Rule #5 mandates 100% English in all code, comments, rules, and skills. Translate technical comments to English; strip CJK characters completely. |
| "Updating `.agents/skills/porting-legacy-to-rust/SKILL.md` is unrelated to adding a helper in `src/crypt.rs`" | If you don't register `src/crypt.rs` in the Shared Primitives Registry, the next agent session will re-implement `pack_pid_bytes` or DES-CBC from scratch. Update the table every time. |
| "I'll create a trait `ICrypt` or `KitchenDao` struct for clean architecture" | Single-implementation traits and pass-through `Dao` wrappers are the exact legacy PHP bloat we are eliminating. Write direct functions and flat structs. |

## Red Flags - STOP and Start Over

- Writing Rust code before checking the **Shared Primitives Registry** above.
- Any Chinese character (`\u4e00-\u9fff`) or Vietnamese word inside `src/**/*.rs`, `.agents/`, `GEMINI.md`, or `REWRITE_PLAN.md`.
- Creating `I<Module>` traits, `<Module>Dao` wrapper structs, or separate boilerplate files.
- Finishing a turn after adding a Rust module/helper without updating the **Shared Primitives Registry** in this skill.
