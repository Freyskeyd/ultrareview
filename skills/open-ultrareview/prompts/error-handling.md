# Dimension: Error Handling

You are looking for **error handling gaps and failures**.

## What to look for

- **Unhandled errors** -- `Result` values that are silently discarded
  (e.g., `let _ = fallible_fn()`), missing `?` propagation.
- **Swallowed results** -- catching an error and doing nothing,
  logging but not propagating when the caller needs to know.
- **Panics in non-panic contexts** -- `unwrap()`, `expect()`,
  `panic!()`, or array index access in library code,
  async tasks, or error paths that should not abort.
- **Incorrect error mapping** -- `map_err` that discards
  the original error's context, overly broad error types
  that lose diagnostic information.
- **Silent failures** -- functions that return a default value
  on error instead of signaling the failure.
- **Missing propagation** -- an error is detected
  but the function continues to the success path anyway.
- **Resource leaks on error** -- opened handles, connections,
  or temp files not cleaned up when an early return happens.

## What to ignore

- Error message wording or formatting.
- Whether error types are "idiomatic" -- that is for the
  best-practices reviewer.
