---
trigger: always_on
---

# Rule 09: Mandatory Planning Auto-Trigger & Zero-Fabrication Guardrail (`09-mandatory-planning-and-zero-fabrication-guardrail.md`)

1. **Mandatory Pre-Execution Planning Auto-Trigger:**
   - Before writing or modifying any production code (`src/**`, `web/**`, `client-assets/**`), the agent MUST first update `.agents/STATE.md` and `REWRITE_PLAN.md` with explicit `- [ ]` checkboxes listing:
     - Exact legacy ground-truth files and line ranges being ported (`legacy-ref/**`).
     - Exact target files and verification tests to execute.
   - Only after the live E2E verification passes may `- [ ]` be updated to `- [x]`.

2. **Zero UI/UX & Logic Fabrication (`1:1` Legacy Ground-Truth Parity):**
   - Never invent custom replacement UIs, fake layouts, or synthetic metrics.
   - For the web portal (`web/src/**`), preserve 100% of the legacy `t9-rxhzw-web` UI/UX (`legacy-ref/www/{index,member,level,fix}.php`, `legacy-ref/www/app/**`, `legacy-ref/www/core/Cx_Ajax.php`):
     - DOM classes: `body_2`, `main_bj`, `Cx_Nav`, `DH_Panel`, `User_Panel bg-main`, `#Main_Content.bg.border.border-main`, `#footer.bg-inverse`.
     - Exact Layer Modal workflows: Login (`app/sys/login.php`), Register (`app/sys/reg.php`), Edit Password (`app/sys/editpass.php`), Find Password (`app/sys/findpass.php`), Server Selection (`app/Game/index.php` -> `app/Game/game.php?FQ=1`), Gold Exchange (`app/dh/index.php`), Level Update (`level.php`), and Account/Copy Fix (`fix.php`).

3. **Zero Raw `.php` File Downloads & Browser Cache Purge:**
   - Never leave `.php` files inside `client-assets/` where a static file server could serve them as `application/octet-stream` downloads. All reference `.php` files must reside strictly in `legacy-ref/www/`.
   - Portal HTML and AJAX responses MUST include `Cache-Control: no-store, no-cache, must-revalidate, max-age=0` and `Clear-Site-Data: "cache"` on entry/session transitions so browser disk caches never replay stale downloads.

4. **Mechanical Test Gate (`cargo test`):**
   - `src/protocol.rs` MUST contain an automated test (`test_mandatory_guardrail_and_zero_php_leak`) that fails the build if:
     - Any `.php` file exists under `client-assets/`.
     - This guardrail rule file or `web/src/App.tsx` is missing.
     - Any `.rs` file in `src/` violates the English-only source policy.
