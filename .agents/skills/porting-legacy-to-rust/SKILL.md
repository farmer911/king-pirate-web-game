---
name: porting-legacy-to-rust
description: Use when implementing, porting, refactoring, or fixing any Rust module, RPC handler, database query, or combat formula in king-pirate, before writing implementation code.
---

# Porting Legacy to Rust

Port one legacy RPC (or one formula) at a time. A port is done only when the Rust response matches the legacy response for recorded traffic.

## Where the Legacy Lives

`legacy-ref/` is a verbatim, read-only copy of the legacy server extracted from the VM disk (see `legacy-ref/INVENTORY.md`). Main entry points:

| Concern | Legacy location |
| :--- | :--- |
| RPC dispatch, sessions | `legacy-ref/rpcfw/lib/RPCFramework.class.php`, `legacy-ref/rpcfw/conf/Framework.cfg.php` |
| Game modules (one dir per RPC class) | `legacy-ref/rpcfw/module/<class>/` (`<Class>.class.php` entry, `<Class>Logic.class.php` rules, `<Class>Dao.class.php` SQL) |
| Constants | `legacy-ref/rpcfw/conf/*.cfg.php`, `legacy-ref/rpcfw/def/*.def.php` |
| Static game data | `legacy-ref/rpcfw/data/btstore/` (184 generated tables) |
| Combat engine (C++ binary, no source) | `legacy-ref/battle/bin/battle` (unstripped, DWARF) + `legacy-ref/battle/data/*.csv` + `legacy-ref/battle/conf/config.xml` |
| DB schema | `legacy-ref/rpcfw/module/*/doc/*.sql` (per module) and raw MySQL files in `legacy-ref/db-raw/pirate20002/` |
| Daemon configs | `legacy-ref/{lcserver,dataproxy,phpproxy,battle}/conf/` |

## Workflow (one RPC per cycle)

1. **Find it.** Locate `<class>.<method>` in `legacy-ref/rpcfw/module/`. Read the entry method, then follow calls into `Logic`/`Dao` only as far as needed. Note: args, tables read/written, `userModify` fields, return shape.
2. **Get a fixture.** Find the request and its legacy reply in `tests/fixtures/sessions/*.log` (grep `# <class>.<method>`; the next `s2c` line is the reply). No recording yet? Record one: start the oracle (`docker compose -f tools/legacy-oracle/compose.yml up -d`), run `cargo run -- record`, play the feature. Never invent the expected output.
3. **RED.** Add a `#[test]` that feeds the fixture through `dispatch_client_rpc` and asserts the legacy response. Run `cargo test` and see it fail.
4. **GREEN.** Implement in `src/domain/<domain>.rs`, wired into `dispatch_client_rpc`. Doc-comment each formula and constant with its `legacy-ref/<path>:<line>`.
5. **Verify.** `cargo test` passes. Remove the `// STUB:` marker and tick the RPC in `REWRITE_PLAN.md`.
6. **Register.** New shared helper? Add it to the table below in the same change.

## Code Rules

- Never `.unwrap()` / `.expect()` on network bytes, AMF3 payloads or client args. Return `Result`.
- Unknown RPC: log `[UNHANDLED_RPC] <class>.<method>` before falling back, so gaps are visible.
- One cohesive module per bounded domain. Do not port the PHP 7-file ceremony (`I<Mod>`, `En<Mod>`, `<Mod>Def`, ...) as separate files.
- Never port `exlib/Smarty*`, `exlib/phpunit`, `script/`, `cov/`, `*.bak`, `doc/`.
- English identifiers and comments only.

## Shared Primitives Registry (check before writing any helper)

| Module | File | Provided primitives (reuse, never duplicate) |
| :--- | :--- | :--- |
| Wire protocol & policy tests | `src/protocol.rs` | `HEAD_SIZE`, `AMF3_PREFIX`, `FLAG_COMPRESSED`, `FLAG_ENCRYPTED`, `FLAG_UNWRAP_RET`, `MESS_CODE_PRIMARY`, `MESS_CODE_SECONDARY`, `BABEL_CRYPT_KEY`, `BABEL_CRYPT_IV`, `PLAY_HASH_KEY`, `FrameHeader::{encode,decode}`, `build_xor_checksum_buffer`, `pack_pid_bytes`, `unpack_pid_bytes` |
| AMF3 codec | `src/amf3.rs` | `Amf3Value::{assoc_map,dense_list}`, `amf3_encode`, `amf3_decode`, `write_u29`, `read_u29`, `write_amf3_str` |
| Crypto & auth | `src/crypt.rs` | `md5_digest`, `md5_hex`, `sha1_digest`, `base64_encode`, `websocket_accept_key`, `phpproxy_checksum`, `babel_encrypt_number`, `babel_decrypt_number`, `compute_login_play_hash`, `verify_login_request` |
| Player actor | `src/actor.rs` | `UPPER_LIMIT_BELLY`, `PlayerSessionActor`, `PlayerDirtyDelta`, `add_belly`, `add_gold`, `add_prestige`, `add_execution`, `add_fight_cdtime`, `move_in_town`, `take_dirty_delta` |
| Data store | `src/data.rs` | `ColumnValue::{as_i64,as_str}`, `UpdateOp`, `WhereCond`, `IdGenerator::next_id`, `DataStore::{insert_or_update,select,select_count,update,delete,apply_write_behind_delta,save_to_disk,load_from_disk}` |
| Traffic recorder | `src/recorder.rs` | `SESSION_DIR`, `DEFAULT_UPSTREAM`, `Dir`, `SessionLog::{new,feed}`, `proxy_connection` |
| RPC access lists | `src/access_list.rs` (generated) | `AccessList`, `PRIVATE_METHODS`, `EXCLUDE_CONNECT_METHODS`, `EXCLUDE_LOGIN_METHODS`, `PUBLIC_METHODS` |
| Gateway | `src/gateway.rs` | `FLASH_POLICY_REQUEST`, `FLASH_POLICY_RESPONSE`, `ConnectionSession`, `MethodAccessTier`, `classify_method`, `authorize_rpc_call`, `encode_framed_packet`, `decode_framed_packet`, `frame_len`, `accept_websocket`, `encode_babeltime_response`, `encode_babeltime_response_with_modify`, `parse_websocket_key`, `read_ws_frame`, `write_ws_frame`, `render_game_launcher_html`, `handle_http_stream_with_store` |
| Web portal | `src/portal.rs`, `web/src/App.tsx` | `INITIAL_ACCOUNT_DJ`, `GOLD_EXCHANGE_RATIO`, `LEVEL_STEP_EXP`, `calculate_all_exp`, `try_handle_portal_route`, `handle_api_session`, `handle_cx_ajax`, `handle_dh_ajax`, `handle_level_api`, `handle_fix_api`, `handle_gm_info`, `handle_gm_execute` |
| Static game store | `src/btstore.rs` | `RawSkill`, `RawBuffer`, `RawTarget`, `StaticBtstore::{new_baseline,load_skills_csv,get_skill,get_buffer,get_target}` |
| Combat engine | `src/battle.rs` | `HeroUnit`, `FormationGrid`, `ReactionRecord`, `ActionRecord`, `BattleResult::to_amf3`, `compute_skill_damage`, `execute_battle_do_hero` |
| Domains | `src/domain/*.rs` | `dispatch_client_rpc`, `load_player_bag_from_store`, `user::*`, `hero::*`, `bag::*`, `forge::*`, `copy::*`, `arena::*`, `boss::*`, `guild::*`, `town::*`, `pet::*`, `economy::*` |
| Entrypoint | `src/main.rs` | `king-pirate` always serves (HTTP `:8081`, WS `:8003`, TCP `:8002`, Flash policy `:843`) |

## Rationalizations to Reject

| Excuse | Reality |
| :--- | :--- |
| "I know how this game probably calculates damage" | Probably is invented. Cite the line or write NOT FOUND. |
| "Returning `[]` keeps the client from disconnecting" | Fine as a temporary fallback only with `[UNHANDLED_RPC]` logging and a `// STUB:`. It is never "done". |
| "Calling it from `#[test]` satisfies dead_code" | Not wired = not done. Wire it into `dispatch_client_rpc`. |
