---
trigger: always_on
---
# RULE #1: Zero Hallucination & 100% Legacy Ground-Truth Enforcement

You are working on **King Pirate (`/Users/phucnguyen/Dev/king-pirate`)**, the **100% Rust (`aarch64-apple-darwin` Single-Binary)** rewrite of the legacy game server (`v298`).
This rule is **ABSOLUTE AND NON-NEGOTIABLE**.

## 1. Zero Guessing, Zero Assumption, Zero Hallucination
- **NEVER** guess, invent, or extrapolate:
  - Binary packet headers, byte order, compression flags, or checksum algorithms.
  - AMF3 serialization flags (`PHP_AMF3_PREFIX`, `AMF_AMF3`) or session compression formats.
  - Cryptographic keys, IVs, or token formulas (`BabelCrypt`, `PlayHashKey`, `MESS_CODE`).
  - Database table names, sharding suffixes (`_0`..`_9`), `va_*` blob structures, or SQL column types.
  - Combat formulas, rage calculations, buffer hooks, or skill target matrices.

## 2. Mandatory Ground-Truth Citation (`file:line`)
Before writing or modifying ANY Rust module (`src/**/*.rs`), you MUST inspect and cite the exact source of truth in `/Users/phucnguyen/Dev/king-pirate/legacy-ref/`:
1. **Wire Protocol & Checksum:** Cite `legacy-ref/rpcfw/lib/PHPProxy.class.php:108-135`, `:235-381`, `legacy-ref/rpcfw/lib/Util.class.php:492-538`, and `legacy-ref/lcserver/conf/game20002.xml:30-60`.
2. **Encryption & Auth:** Cite `legacy-ref/rpcfw/lib/BabelCrypt.class.php:18-66` and `legacy-ref/rpcfw/conf/BabelCrypt.cfg.php:18-29`.
3. **RPC Dispatch & Session Lifecycle:** Cite `legacy-ref/rpcfw/lib/RPCFramework.class.php:185-800` and `legacy-ref/rpcfw/lib/ServerProxy.class.php:32-248`.
4. **Database & Query Cache:** Cite `legacy-ref/rpcfw/lib/data/NewData.class.php:58-927`, `legacy-ref/rpcfw/lib/data/LocalQueryCache.class.php:18-411`, `legacy-ref/dataproxy/conf/config.xml:64-143`, and `legacy-ref/schema/pirate20002-schema.sql`.
5. **Combat Engine (`battle`):** Cite DWARF struct definitions in `legacy-ref/dwarf/battle-dwarf-clean.hpp:1-386` (`CHero`, `CSkill`, `CFormation`, `CBuffer`, `CPet`, `CActionRecord`, `CReactionRecord`), `legacy-ref/battle/conf/config.xml:52-72`, `legacy-ref/battle/data/{skill,buffer,target}.csv`, and `legacy-ref/rpcfw/module/battle/BattleLogic.class.php:19-185`.

## 3. Missing Ground-Truth Protocol
- If a required file or symbol is not in `legacy-ref/`, do **NOT** fabricate behavior.
- Extract the missing file or symbol directly from the running CentOS 6.4 VM (`ssh -p 2023 root@127.0.0.1` inside `/home/pirate/`), strip non-English comments while preserving exact line numbers, save it into `legacy-ref/`, run the `updating-project-knowledge` skill, and cite its exact line numbers.
