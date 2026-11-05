---
name: playwright-e2e-evidence
description: Use when planning, implementing, testing, or verifying any web portal, React SPA, Ruffle WebAssembly, or Rust RPC gameplay feature, before claiming any feature works or any task is complete.
---

# Playwright E2E Browser & Legacy Differential Evidence

## Overview

Never claim a feature works based only on Rust unit tests (`cargo test`) or static code inspection. Every feature ported from the legacy system (`legacy-ref/` and `Vua-Hai-Tac-2025-UP-LouLx`) to Rust + React SPA must be verified end-to-end by **Playwright headless Chromium** (`web/e2e/*.spec.ts`) running against the live `./target/debug/king-pirate serve` daemon (`:8081` HTTP, `:8003` WebSocket, `:8002` TCP), backed by verbatim Playwright CLI output, DOM/WebSocket assertions, and legacy ground-truth citations (`file:line`).

**Core principle:** If Playwright did not click it, intercept its network/WebSocket frames, and assert its persisted state against `legacy-ref/`, it is NOT verified.

**REQUIRED SUB-SKILL:** Use `honesty`, `test-driven-development`, `zero-defect-real-runtime-verification`, and `verification-before-completion`.

## The 5 Iron Laws of Playwright Evidence

```
1. RED PLAYWRIGHT TEST FIRST (TDD FOR E2E):
   Before modifying production code in src/ or web/src/, write or update a Playwright E2E test in web/e2e/legacy-parity.spec.ts, run it against the live server, and record the exact RED failure output.

2. ZERO .PHP ROUTES & ZERO <IFRAME> GAME WRAPPERS:
   Every Playwright test must attach a page.on("request") listener asserting zero ".php" URLs and assert page.locator("iframe").count() === 0 while verifying direct Ruffle WebAssembly (<ruffle-player> / <ruffle-embed>) rendering inside #ruffle-game-container.

3. 100% LEGACY GROUND-TRUTH CROSS-REFERENCE BEFORE CODING:
   Before porting any RPC handler or web tool, inspect the exact legacy PHP file (legacy-ref/rpcfw/module/<mod>/*.class.php, legacy-ref/rpcfw/def/*.def.php, legacy-ref/www/*.php) via Graphify MCP + targeted read_file (<= 120 lines), and document the exact input args, DataStore tables, userModify callback, and return structure.

4. ZERO STATELESS STACK DUMMIES OR SILENT "_ => []" FALLBACKS:
   Never instantiate temporary structs on the stack and drop them, and never let Main.swf RPC calls fall into "_ => Amf3Value::dense_list([])". Every RPC call logged in /tmp/king-pirate-live.log must read/write DataStore and push babeltime.player.userModify when currencies change.

5. VERBATIM PLAYWRIGHT + CARGO TEST PROOF BLOCK:
   Never claim completion without running both:
   - rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test
   - cd web && node ./node_modules/@playwright/test/cli.js test
   fresh after the final edit and pasting the verbatim output.
```

## Mandatory 5-Step Verification Workflow

1. **Step 1 — Legacy Ground-Truth Audit (`Graphify MCP -> legacy-ref/`):**
   - Query `graphify` MCP (`query_graph` / `get_node` / `get_neighbors`) and read the exact `legacy-ref/` method (`<= 120` lines) to determine the required `DataStore` tables (`t_user`, `t_hero`, `t_bag`, `t_item`, `t_hero_formation`, `t_charity`, `t_switch`, `t_task`), `SwitchDef` bitflags (`1..63`), and AMF3 return schema.
2. **Step 2 — Write Failing Playwright Test (`Verify RED`):**
   - Add an E2E test in `web/e2e/legacy-parity.spec.ts` covering the UI interaction, REST/WebSocket payload, and persistence check.
   - Run `cd /Users/phucnguyen/Dev/king-pirate/web && node ./node_modules/@playwright/test/cli.js test` and watch it fail (`RED`).
3. **Step 3 — Build & Restart Live Production Daemon:**
   - Run `rtk ~/.cargo/bin/cargo build` and `cd web && npx tsc --noEmit && npm run build`.
   - Restart `./target/debug/king-pirate serve` so `:8081` serves the latest Rust binary and React SPA bundle (`client-assets/portal/app.js`).
4. **Step 4 — Run Playwright E2E Suite (`Verify GREEN`):**
   - Execute `cd /Users/phucnguyen/Dev/king-pirate/web && node ./node_modules/@playwright/test/cli.js test`.
   - Verify `0` `.php` requests, `0` `<iframe>` elements, `0` console/page errors, `0` `[UNHANDLED_RPC]` lines in `/tmp/king-pirate-live.log`, and `100%` passing assertions.
5. **Step 5 — Run Rust Compiler & Mechanical Honesty Gate:**
   - Execute `rtk ~/.cargo/bin/cargo check && rtk ~/.cargo/bin/cargo test` and attach both command outputs in the response.

## Rationalization Prevention Table

| Excuse | Reality |
| :--- | :--- |
| "`cargo test` passed 21/21, so the game features work" | `cargo test` did not catch `user.getSwitch` returning `[]` (locking all 63 UI icons in `Main.swf`), `36/58` RPCs falling into `_ => []`, `.php` fetch URLs in `App.tsx`, or missing `/admin_loulxgame`. Only Playwright E2E + live RPC log audits catch real runtime failures. |
| "Returning `Amf3Value::dense_list([])` for unknown RPCs prevents disconnects" | Returning `[]` silently breaks in-game buttons (`charity.fetchPresigeSalary`, `user.getSwitch`, `turntable.getInfo`) while hiding the bug. Log `[UNHANDLED_RPC]` and implement the real `legacy-ref/` logic. |
| "Creating `PlayerBag::new(24)` inside `dispatch_client_rpc` satisfies `bag.bagInfo`" | Ephemeral stack structs lose all items on the very next RPC call. Always load and save state in `DataStore`. |
| "Embedding `/T9NuoYa/T9NuoYa.html` in an `<iframe>` counts as Ruffle in React SPA" | The user explicitly forbade `<iframe>` wrappers and `.php` routes. Mount Ruffle WebAssembly (`window.RufflePlayer.newest().createPlayer()`) directly inside a React `useRef` container (`#ruffle-game-container`). |

## Red Flags - STOP and Fix Immediately

- Claiming any UI or gameplay feature works without running `node ./node_modules/@playwright/test/cli.js test`.
- Any network request or browser URL containing `.php` during a Playwright test run.
- Any `<iframe>` tag in `web/src/App.tsx`.
- `user.getSwitch` returning `[]` instead of the unlocked `SwitchDef` IDs (`1..63`).
- Any `Main.swf` RPC call in `/tmp/king-pirate-live.log` falling into `_ => []` or failing to emit `babeltime.player.userModify` when player currencies change.
