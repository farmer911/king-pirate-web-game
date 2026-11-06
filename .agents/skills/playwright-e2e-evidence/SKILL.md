---
name: playwright-e2e-evidence
description: Use when changing anything under web/, the portal routes in src/portal.rs, or the Ruffle game shell, before claiming the change works.
---

# Playwright E2E Evidence

`cargo test` does not prove a page or the game shell works. A UI change is verified only by Playwright against a freshly built, running server.

## Workflow

1. **RED:** add or update a test in `web/e2e/legacy-parity.spec.ts` for the behavior. Run it and see it fail.
2. **Build:** `cargo build` and `cd web && npm run build` (outputs `client-assets/portal/app.js`).
3. **Run the server:** `cargo run -- serve` (HTTP `:8081`, WS `:8003`, TCP `:8002`).
4. **GREEN:** `cd web && npx playwright test`. Paste the `N passed` line.
5. **Unit gate:** `cargo test`. Paste the result line.

## Assertions Every Portal Test Keeps

- Zero network requests and zero URLs containing `.php`.
- Zero `<iframe>`; Ruffle mounts directly in `#ruffle-game-container`.
- Zero console errors and zero `[UNHANDLED_RPC]` lines in the server log for the flow under test.
- Legacy layout classes and text come from `legacy-ref/www/` (cite the line).

## Red Flags

- Claiming a UI works with only `cargo test` output.
- Playwright run against a server started before the last `cargo build`.
- A `.php` URL or `<iframe>` in `web/src/App.tsx`.
