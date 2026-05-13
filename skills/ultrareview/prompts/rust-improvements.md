# Dimension: Rust Improvements

You are looking for **meaningful opportunities
to make the Rust code more idiomatic, safer, or cleaner**.
The code works correctly as-is --
you are suggesting concrete improvements, not reporting bugs.

## What to look for

- **Iterator patterns** -- manual loops that could use
  `map`, `filter`, `fold`, `collect`, or other combinators
  for clarity and potential performance.
- **Type system leverage** -- places where newtypes,
  enums, or marker types could make invalid states
  unrepresentable at compile time.
- **Trait design** -- opportunities to use standard traits
  (`From`/`Into`, `Display`, `Default`, `AsRef`)
  instead of ad-hoc methods.
- **Ownership patterns** -- places where borrowing, `Cow`,
  or zero-copy patterns would avoid unnecessary cloning.
- **Standard library APIs** -- using `Entry` API for maps,
  `Option` combinators instead of `match`,
  `str` methods instead of manual parsing.
- **Error type design** -- opportunities to use
  more specific error types, `thiserror` derive,
  or structured error context.
- **Builder patterns** -- complex construction that would
  benefit from a builder or `Default` + selective overrides.

## What to ignore

- Formatting and naming (those are style, not substance).
- Changes that would be correct but don't provide
  clear benefit in readability, safety, or performance.
