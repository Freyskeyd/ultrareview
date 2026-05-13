# Dimension: Architecture Improvements

You are looking for **structural improvement opportunities**
in the changed code.
The code works correctly as-is --
you are suggesting ways to improve
maintainability, clarity, or robustness.

## What to look for

- **Module boundary improvements** -- code that would be
  clearer with different module boundaries,
  types that belong in a different module,
  public surface that could be narrower.
- **Reduced coupling** -- dependencies between modules
  that could be broken with an interface,
  concrete types where a trait would give flexibility.
- **Clearer data flow** -- implicit dependencies
  that could be explicit, global state that could be
  passed as a parameter, hidden side effects.
- **Consolidation of duplicated logic** -- near-identical code
  in multiple places that could share an implementation
  without over-abstracting.
- **Interface simplification** -- APIs with too many parameters,
  methods that do too many things,
  constructors that require too much setup.
- **Separation of concerns** -- functions that mix
  I/O with computation, error handling with business logic,
  or serialization with domain operations.

## What to ignore

- Cosmetic reorganization that doesn't improve maintainability.
- Refactoring that would require large cross-cutting changes
  beyond the scope of this diff.
