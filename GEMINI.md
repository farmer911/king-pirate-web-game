# King Pirate - Shared Project Knowledge

Read by Gemini (this file) and by Claude (`CLAUDE.md` imports it). Keep facts here, rules in `.agents/rules/00-core.md`, plans in `REWRITE_PLAN.md`.

@.agents/rules/00-core.md

## 1. Goal

Rewrite the legacy "Vua Hai Tac 2025" (BabelTime `v298`) game server as **one Rust binary in one Docker container**, started with `docker compose up`, light enough (low RAM, fast start) to host cheaply for a small group of friends. Non-commercial.

## 2. Repository Map

| Path | What | In git |
| :--- | :--- | :--- |
| `src/` | Rust server (protocol, AMF3, crypto, gateway, portal, data store, battle, domains) | yes |
| `web/` | React portal + Ruffle game shell, Playwright tests in `web/e2e/` | yes |
| `.agents/rules/00-core.md` | The only always-on rule file | yes |
| `.agents/skills/*/SKILL.md` | Skills shared by Gemini and Claude | yes |
| `CLAUDE.md`, `GEMINI.md` | Agent entry points (this file is the shared knowledge) | yes |
| `legacy-ref/` | Verbatim read-only copy of the legacy server, see `legacy-ref/INVENTORY.md` | no (local) |
| `client-assets/` | Game client (`Main.swf`, Ruffle, built portal) | no (local) |
| `data/` | Runtime save data | no (local) |
| `REWRITE_PLAN.md` | Phases, checkboxes, decisions, stub list | no (local) |

## 3. Commands

| Task | Command |
| :--- | :--- |
| Unit + policy tests | `cargo test` |
| Build server binary | `cargo build` (required before running; `cargo test` does not rebuild it) |
| Run server | `cargo run` (always serves: HTTP `:8081`, WS `:8003`, TCP `:8002`, Flash policy `:843`; a `serve` argument is accepted and ignored). Unported RPCs print `[UNHANDLED_RPC] <class>.<method>` to stderr |
| Build portal | `cd web && npm install && bun run build` (writes `client-assets/portal/app.js`; needs Bun) |
| Portal E2E | `cd web && npx playwright test` (server must be running) |
| Legacy oracle (local only) | Stage once in WSL: `tools/legacy-oracle/stage.sh <mounted VM root> /root/kp/oracle`, `docker build -t kp-legacy-oracle /root/kp/oracle`; then `docker compose -f tools/legacy-oracle/compose.yml up -d` (game `127.0.0.1:18002`, old portal `127.0.0.1:18080`) |
| Record legacy traffic | `cargo run -- record` (or `--upstream host:port`), open `http://127.0.0.1:8081`, play; each connection is saved to `tests/fixtures/sessions/` |
| Env vars | `KING_PIRATE_ASSETS` (default `client-assets`), `KING_PIRATE_STORE` (default `data/pirate_store.amf3`) |

## 4. Legacy System (ground truth)

Full inventory: `legacy-ref/INVENTORY.md`. Summary (measured from the extracted VM, 2026-11-06):

- **Stack:** CentOS 6.4 VM. Four C++ daemons (`lcserver` gateway :8002, `phpproxy` unix socket, `dataproxy` :3300, `battle` :1234), PHP 5 `rpcfw` behind php-cgi FastCGI :5555, MySQL 5.1.59, memcached, nginx portal.
- **Game logic to port:** `rpcfw/module/` has 106 modules, 918 PHP files, 192,070 lines, at least 790 RPC methods. Framework (`lib/`, `conf/`, `def/`, `hook/`) adds 24,687 lines. Largest: `copy` 10,870, `user` 8,312, `item` 7,984, `groupwar` 7,799, `hero` 7,235.
- **Static data:** `rpcfw/data/btstore/` (184 generated tables, 147 MB) and `battle/data/*.csv`.
- **Combat:** only a C++ binary (`battle/bin/battle`), but it is **not stripped and carries DWARF debug info**, so it can be decompiled. It has elemental damage (fire, thunder, water, wind), composite skills, buffs and `attack_coef`/`defend_coef = 0.0001` (`battle/conf/config.xml`). It is also a standalone TCP service, so it can serve as a test oracle.
- **Database:** 210 tables in `db-raw/pirate20002/` (raw MySQL 5.1 files; needs a MySQL 5.1-compatible server to dump).
- **Rust coverage today:** the dispatcher names about 84 methods across 37 classes (regex count over `src/domain/mod.rs`), about 11% of 790, and most of those return hard-coded data.
- **Where to look:** RPC entry `rpcfw/module/<class>/<Class>.class.php`, rules `<Class>Logic.class.php`, SQL `<Class>Dao.class.php`, constants `rpcfw/conf/*.cfg.php`, method access lists `rpcfw/conf/Framework.cfg.php:203-266`, token-exempt methods `lcserver/conf/game20002.xml`.

- **The VM image is infected** (botnet, backdoor, extra root accounts, PHP webshells; details in `REWRITE_PLAN.md` Phase 2). Never boot it. The oracle container uses only `/home/pirate` on a clean CentOS 7 base.
- **Recorded traffic:** `tests/fixtures/sessions/*.log`, one frame per line (`<ms> <c2s|s2c> <hex>`) with a `# method` / `# reply <callback> (err=..)` comment. A legacy reply is `{ ret: [ByteArray, ...] }`, each ByteArray an AMF3 `{ err, callback: { callbackName }, ret }`.

## 5. Component Status (honest, update at phase end)

| Component | Files | Status |
| :--- | :--- | :--- |
| Wire framing, AMF3, BabelCrypt DES, MD5 | `src/protocol.rs`, `src/amf3.rs`, `src/crypt.rs` | done (unit tests) |
| HTTP/WS/TCP gateway, Flash policy | `src/gateway.rs`, `src/main.rs` | done, one OS thread per connection |
| Web portal (login, register, GM, level, fix) | `src/portal.rs`, `web/src/App.tsx` | works; session cookie is the raw username (forgeable) and passwords are stored in plain text |
| Data store | `src/data.rs` | in-memory `BTreeMap` rows with linear scans + whole-file AMF3 snapshot; to be replaced (see plan) |
| RPC dispatch | `src/domain/mod.rs` | 67 arms marked `// STUB:` (hard-coded or fake data, listed in `REWRITE_PLAN.md`); unknown RPCs log `[UNHANDLED_RPC]` and return `[]`. Access control uses the legacy per-method lists (`src/access_list.rs`, generated from `Framework.cfg.php:203-411`); a method in no list closes the connection, as in legacy |
| Battle engine | `src/battle.rs`, `src/btstore.rs` | simplified placeholder: one flat formula, single target, no buffs; does not load the real skill table |
| Gameplay domains | `src/domain/*.rs` | thin; most legacy modules not started |
| Legacy oracle + recorder | `tools/legacy-oracle/`, `src/recorder.rs` | works: full login, create character, enter town session recorded (52 requests) |
| Ruffle client | `client-assets/T9NuoYa/`, Ruffle 0.7.1 | loads, but Main.swf parses 72.8 MB of XML before connecting: about 7-9 minutes in headless Chromium (`maxExecutionDuration: 600` in `src/portal.rs`) |
| Docker | `Dockerfile`, `docker-compose.yml` | broken: image has no `client-assets/` (the binary now serves by default), the MariaDB service and `DATABASE_URL` are unused by the code |

## 6. Lessons Log (add one entry when a rule is broken or a bug escapes)

- **2026-11-06 Stale binary.** `cargo test` does not rebuild `target/debug/king-pirate`; a stale binary rejected `payback.getCurAvailablePayBackIds` and disconnected the client at the loading bar. Prevention: `cargo build` before `serve`.
- **2026-11-06 Skipped legacy portal.** `GET /` redirected straight into the game, bypassing register/login/server select. Prevention: port the legacy portal pages (`legacy-ref/www/`) 1:1.
- **2026-11-06 Wrong constants.** `cur_execution = 100` was used instead of `UserConf::INIT_EXECUTION = 100000` (`User.cfg.php:188`); generic `{}` returns caused AS3 `TypeError #1010`. Prevention: copy constants and return shapes from `legacy-ref/` with a citation.
- **2026-11-06 Cached `.php` downloads.** Static serving of `client-assets/member.php` with `max-age=3600` made Chrome cache it as a download. Prevention: no `.php` in `client-assets/` (checked by `cargo test`), `Cache-Control: no-store` on portal routes.
- **2026-11-06 Infected legacy image.** The downloaded VM carried a botnet, a backdoor, two extra uid-0 accounts and PHP webshells (two were copied into `legacy-ref/` and are now deleted). Prevention: never execute anything from the image; scan copied PHP for `web-shell`/`shellname=` (done by `tools/legacy-oracle/stage.sh`); keep `.php` out of `client-assets/` (checked by `cargo test`).
- **2026-11-06 Rules ignored.** 10 always-on rules and 12 skills (103,844 bytes with GEMINI.md, 77 "MUST/NEVER/ALWAYS/ZERO"-style absolutes) contradicted each other (e.g. "zero external crates" vs "flush to MariaDB"; the rule banning unmeasured numbers sat next to Rule 08, which itself quoted an unmeasured nanosecond figure), required tools that only exist on one Mac (Graphify before every read, Archify, `/Users/phucnguyen/...`), and one skill carried another project's commands (RentalCar `bun run typecheck`). The agent could not satisfy all of them, so it satisfied none reliably. Prevention: one short always-on file with a priority order (`00-core.md`), skills routed by a table, and `cargo test` asserting only one always-on rule exists.
