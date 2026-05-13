# Dimension: Logic Bugs

You are looking for **logical correctness errors** in the changed code.

## What to look for

- **Off-by-one errors** -- incorrect loop bounds, fence-post problems,
  wrong index arithmetic.
- **Wrong boolean conditions** -- inverted checks, missing negation,
  incorrect operator (`&&` vs `||`, `<` vs `<=`).
- **Missing edge cases** -- null/None/empty inputs not handled,
  integer overflow, division by zero, empty collections.
- **Incorrect state transitions** -- states that can be reached
  but shouldn't, missing transitions, wrong ordering.
- **Wrong variable used** -- copy-paste errors where the wrong variable
  is referenced (e.g., checking `a` but acting on `b`).
- **Unreachable code paths** -- dead code that suggests
  a condition was written incorrectly.
- **Type confusion** -- signed vs unsigned comparisons,
  lossy casts, truncation.

## What to ignore

- Code style or naming.
- Performance (another reviewer covers this).
- Missing error handling (another reviewer covers this).
