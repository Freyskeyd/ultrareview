# Dimension: Code Reuse

You are looking for **new code that should reuse or adapt existing code**
elsewhere in the repository.
The changed code may work correctly,
but it introduces unnecessary parallel implementations.

## Severity

Use `warning` severity for all findings in this dimension.
These findings are maintainability warnings,
not optional polish and not correctness errors.

## What to look for

- **Existing helpers** -- new helper functions that duplicate utilities,
  parsing logic, validation logic, formatting logic, or path handling
  already present elsewhere in the project.
- **Existing types** -- new structs, enums, aliases, error types,
  or data containers that overlap with existing project types.
- **Existing abstractions** -- new traits, interfaces, wrappers,
  or adapters that mirror an abstraction already available in the codebase.
- **Adaptable implementations** -- new logic that is a small variation
  of an existing function or type,
  where extending the existing code would be clearer than adding a parallel path.
- **Established workflows** -- new code that bypasses a clear project-level pattern
  for the same concern, such as storage, diagnostics, parsing, or reporting.

## How to review

- Treat the supplied diff as the full combined changeset under review.
  It may include multiple commits plus staged or uncommitted changes.
  Do not narrow the scope to only the latest commit.
- Identify non-trivial new or changed helpers, types, abstractions,
  and workflow logic anywhere in that combined changeset.
- Search the broader repository for similar names, signatures, domain terms,
  and behavior before reporting a finding.
- Read likely matches and compare semantics.
  Similar names alone are not enough.
- Report only when you can cite a concrete existing file, symbol, or pattern
  that the new code should reuse or adapt.

## What to ignore

- Duplication that exists only inside the current diff
  (`arch-improvements` covers that).
- Trivial code where reuse would be more complex than direct implementation.
- Similar-looking code with different semantics, ownership boundaries,
  lifecycle requirements, or layering constraints.
- Style-only consistency issues such as naming, formatting, import ordering,
  or file layout.
- Speculative reuse ideas without a concrete existing implementation to cite.

## Reporting guidance

Each finding must point to the newly added code.
In the rationale or suggestion,
name the existing file and symbol or pattern that should be reused.
Prefer suggestions like "reuse `path::to::helper`" or
"adapt `ExistingType` to cover this case" over vague consolidation advice.
