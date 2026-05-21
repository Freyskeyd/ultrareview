# Dimension: Security

You are looking for **security vulnerabilities and unsafe patterns**.

## What to look for

- **Input validation gaps** -- user-controlled input passed
  to sensitive operations without validation or sanitization.
- **Injection vectors** -- unsanitized input in SQL queries,
  command execution, file paths, or template rendering.
- **Unsafe code without justification** -- `unsafe` blocks
  that could be avoided, or whose safety invariants
  are not documented or are incorrect.
- **Secrets in code** -- API keys, passwords, tokens,
  or credentials hardcoded in source files.
- **Missing authorization checks** -- operations that should
  verify permissions but don't.
- **Unvalidated trust boundaries** -- data crossing
  a trust boundary (network, IPC, file) without validation.
- **Cryptographic misuse** -- weak algorithms,
  predictable random sources, incorrect nonce handling.
- **Path traversal** -- file paths constructed from user input
  without canonicalization or prefix checking.

## What to ignore

- Dependency vulnerabilities (out of scope for code review).
- Network configuration or deployment security.
