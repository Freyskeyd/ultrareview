# Code Reuse Reviewer Design

## Goal

Add a new Open Ultrareview reviewer dimension that catches cases where a diff adds
new code instead of reusing or adapting code that already exists elsewhere in the project.

The reviewer should help keep the codebase coherent over time.
It should prevent unnecessary parallel helpers, types, traits, and implementation patterns
from accumulating when a specific existing implementation can be reused or extended.

## Naming

The dimension should be named `code-reuse`.

This name is more precise than `code-consistency`.
Consistency can include naming style, formatting, module layout, error handling conventions,
and many other concerns that would make the reviewer noisy.
The selected scope is narrower: identify new code that duplicates or bypasses reusable code
that already exists in the broader codebase.

`code-reuse` also matches the existing dimension naming style:
short, hyphenated, and behavior-oriented.

## Current Context

Open Ultrareview already has several language-agnostic reviewer dimensions.
The closest existing one is `arch-improvements`,
which looks for structural improvement opportunities in the changed code.
That prompt already mentions consolidation of duplicated logic,
but its practical scope is the diff and the changed files that the orchestrator provides.

The new reviewer fills a different gap.
It should actively compare changed code against the rest of the repository.
Its main job is not to find duplicated code inside the diff,
but to find places where the diff introduces code that should have reused pre-existing code.

## Recommended Approach

Add `code-reuse` as a new language-agnostic review dimension.
It should be enabled by the existing `auto` dimension resolution,
the same way other `any` dimensions are enabled.

The dimension should use the improvement reviewer preamble,
because most findings describe maintainability issues rather than correctness bugs.
However, `code-reuse` findings should use `warning` severity instead of `info`.
This reflects that unnecessary reimplementation can create real future maintenance risk,
even when the current behavior is correct.

The prompt should make the severity rule explicit.
It should also explain that findings are stronger than optional cleanup,
and should only be emitted when there is a concrete existing implementation to reference.

## Reviewer Behavior

The reviewer should start from the full combined changeset supplied by the orchestrator.
That changeset covers `git diff <base-ref>..HEAD` plus staged and uncommitted changes.
It is not limited to the latest commit unless the user explicitly chooses such a base ref.

The reviewer should identify non-trivial new or changed code anywhere in that changeset,
especially helper functions, type definitions, traits, parsing or validation logic,
and repeated workflow patterns.

For each candidate, it should search the broader repository for related code.
The search should use names, signatures, nearby domain terms, and observable behavior.
Promising matches should be read and compared before reporting a finding.

Each finding must point to the newly added code in the diff.
Its rationale or suggestion must also name the existing file, symbol, or pattern that should be reused.

The reviewer should report these cases:

- A new helper duplicates an existing helper or utility.
- A new type overlaps with an existing type that could be reused.
- A new trait or abstraction mirrors an existing trait or interface.
- A new function is a small generalization of an existing function,
  where adapting the existing function would be clearer than adding a parallel one.
- A new workflow pattern bypasses an established project-level implementation for the same concern.

The reviewer should ignore these cases:

- Duplication that exists only inside the current diff.
  `arch-improvements` already covers that area.
- Trivial code where reuse would be more complex than direct implementation.
- Similar-looking code with different semantics, layering requirements, or ownership boundaries.
- Style-only consistency issues such as naming, formatting, or import organization.
- Speculative reuse opportunities without a concrete existing implementation to cite.

## Severity

`code-reuse` should be a warning-level dimension.

The finding should use:

```json
"severity": "warning"
```

This intentionally differs from the existing blanket rule that improvement findings are `info`.
The prompt and output instructions should describe `code-reuse` as a maintainability warning.
It is not a correctness `error`,
but it is more important than optional polish.

The bridge mapping should preserve the reviewer-provided `warning` severity for this dimension,
instead of forcing all improvement dimensions to `info`.

## Config And Schema Changes

`config-schema.yml` should include `code-reuse` in the valid dimension list.
Its language map entry should be:

```yaml
code-reuse: any
```

`schemas/finding.json` should include `code-reuse` in the allowed `dimension` enum.
The existing `severity` enum already supports `warning`,
so no new severity value is required.

## Output Changes

The orchestrator currently documents all improvement findings as `info`.
That rule needs a narrow exception for `code-reuse`.

Confirmed `code-reuse` findings should be emitted as compiler-style warning diagnostics.
Bridge output should also use `warning` severity.

This change should remain dimension-specific.
Other improvement dimensions should continue to emit `info` findings.

## Non-Goals

This feature does not add a full repository index.
The reviewer can use normal repository search and file-reading tools.

This feature does not introduce a general style-consistency reviewer.
It should not flag naming, formatting, import layout, or aesthetic convention differences.

This feature does not change the verifier schema.
Verifier prompts can continue treating this as an improvement-type finding,
while the final output preserves warning severity.

## Testing

This repository primarily stores the Open Ultrareview workflow as Markdown,
JSON schema, and YAML schema documentation.

Verification should include:

- Checking the new prompt for clear scope boundaries and false-positive controls.
- Checking `config-schema.yml` for dimension list and language map consistency.
- Checking `schemas/finding.json` for the new dimension enum value.
- Checking orchestrator output instructions so `code-reuse` can emit `warning` diagnostics.
- Checking README references to reviewer dimensions.
