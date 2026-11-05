---
trigger: always_on
---

# Rule 08: Peak Performance & Best Outcome First (Quality > Time > File Count)

1. **Supreme Priority (`Peak Performance & Engineering Excellence > Implementation Time > File Count`):**
   - Implementation time is **NOT** a constraint. Never propose shortcuts, temporary workarounds, or half-measures to save coding time.
   - Minimizing file count (`fewest files`) is **NOT** a goal. Never cram unrelated domains into giant monolithic files just to reduce file count; structure modules for clean domain boundaries, parallel `rustc` LLVM codegen, and long-term maintainability (while strictly forbidding dead code and redundant wrapper layers).
   - Always optimize for **maximum runtime performance, lowest latency (`p99`), highest concurrency throughput, rock-solid security, and cleanest architecture**.

2. **Proactive Better-Plan Mandate:**
   - Whenever evaluating or updating a plan, proactively identify every architectural bottleneck and propose the highest-performance solution backed by code evidence (`file:line`) and systems engineering proof:
     - **100% Pure Rust Completion (No Legacy PHP Container Left Behind):** Port all active game domains into native Rust so zero `php-fpm` processes remain in production.
     - **In-Memory Player Actor (`uid`) + Write-Behind Persistence:** Keep active player state (`User`, `Hero`, `Bag`, `Formation`, `Task`) in L1/L2 RAM (`PlayerSessionActor`, `~50 ns` reads/mutations) and flush dirty diffs (`userModify`) asynchronously in batched transactions to MariaDB/PostgreSQL (Online Docker) or SQLite WAL (macOS Local Dev).
     - **Zero-Allocation Static Game Data (`#[repr(C)]` Compile-Time / `mmap` Tables):** Convert `btstore` and `battle/data/*.csv` into contiguous, cache-aligned `#[repr(C)]` structs with `O(1)` integer index lookup (`0` runtime string hashing or heap allocation).
     - **L1-Cache Contiguous Combat Engine:** Execute `CHero`, `CSkill`, `CFormation`, and `CBuffer` on stack/arena-allocated arrays (`[HeroUnit; 18]`) with deterministic PRNG and zero heap allocations per round.
