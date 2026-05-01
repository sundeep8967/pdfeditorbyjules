# ADR 002: Third-Party Rust Crate Dependency Policy

## Status
Accepted

## Context
Since this SDK is our proprietary business moat, we cannot rely on open-source PDF logic. Doing so would violate our IP strategy and potentially entangle us in copyleft (GPL/AGPL) licenses. However, rewriting basic math or compression algorithms wastes engineering time without adding value.

## Decision
We are permitted to use third-party Rust crates only if they meet these strict criteria:
1. **License:** MIT or Apache 2.0 only. No GPL, LGPL, or BSL.
2. **Scope:** Must be a single-purpose, primitive utility (e.g., compression, crypto, threading).
3. **Health:** Actively maintained with high download counts.

**Strictly Forbidden:** Any crate handling PDF object parsing, content stream interpretation, font metrics, or page layout.

### Approved Initial Crates:
- `flate2` (FlateDecode/zlib)
- `rayon` (Data parallelism)
- `aes`, `md-5`, `sha2` (Crypto/Encryption)
- `crc32fast` (Integrity)
- `thiserror`, `log` (Base utilities)

## Consequences
- Requires developers to reinvent the wheel for PDF-specific logic, but ensures 100% ownership.
