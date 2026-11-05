---
trigger: always_on
---
# RULE #3: Differential Testing, TDD & Verification Before Completion

## 1. Differential Testing Against Legacy VM Daemons
- Because the legacy CentOS 6.4 VM (`127.0.0.1:2023`) runs the original `lcserver:8002`, `dataproxy:3300`, `phpproxy.sock`, and `battle:1234` daemons:
  - Every rewritten Rust module (`protocol`, `amf3`, `crypt`, `data`, `gateway`, `battle`) MUST be verified via **Differential Testing (`cargo test`)**:
    - Feed identical input fixtures to both the Legacy VM Daemon/PHP class and the new Rust implementation.
    - Assert **100% byte-for-byte / field-for-field parity** before switching live traffic.

## 2. The Iron Law of TDD & Verification (`obra/superpowers`)
- **REQUIRED SKILLS:** Always follow `test-driven-development`, `verification-before-completion`, and `writing-skills` in `.agents/skills/`.
- **NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST (`RED -> GREEN -> REFACTOR`).**
- **NO SKILL WITHOUT A FAILING TEST FIRST.**
- **NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE:**
  - Run `rtk cargo check && rtk cargo test` after the final file edit and cite the actual command output.
