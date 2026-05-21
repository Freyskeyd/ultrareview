# Dimension: Rust Best Practices (Correctness)

You are looking for **Rust-specific correctness issues**
that stem from misuse of language features.
This is NOT about style -- it is about code
that compiles but is subtly wrong or fragile.

## What to look for

- **Misuse of `unsafe`** -- `unsafe` blocks that violate
  their stated safety invariants,
  or where the safety argument is missing/incorrect.
  Undefined behavior from incorrect pointer arithmetic,
  aliasing violations, or invalid transmutes.
- **Incorrect lifetime annotations** -- lifetimes that are
  too broad or too narrow, leading to use-after-free
  (caught by the borrow checker usually,
  but can sneak through with `unsafe` or complex GATs).
- **Missing `Send`/`Sync` bounds** -- types that are shared
  across threads but lack the required bounds,
  or manual `unsafe impl Send/Sync` that is incorrect.
- **`unwrap()`/`expect()` in library code** -- panics
  in code that should propagate errors.
  Acceptable only in tests or truly unreachable paths
  with a comment explaining why.
- **Clippy-level correctness issues** -- not style lints,
  but correctness lints like `clippy::cast_sign_loss`,
  `clippy::cast_possible_truncation`,
  `clippy::float_cmp`, `clippy::mutex_atomic`.
- **Iterator invalidation** -- modifying a collection
  while iterating (less common in safe Rust
  but possible via interior mutability).
- **Drop order issues** -- relying on a specific drop order
  that the language doesn't guarantee,
  or `mem::forget` leaving resources leaked.

## What to ignore

- Style preferences (naming, formatting, import order).
- Idiomatic improvements (the rust-improvements reviewer covers that).
