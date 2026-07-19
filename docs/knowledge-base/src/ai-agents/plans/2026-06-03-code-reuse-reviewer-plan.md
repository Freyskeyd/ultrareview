# Code Reuse Reviewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `code-reuse` Open Ultrareview dimension that flags unnecessary
new code when a concrete pre-existing implementation can be reused or adapted.

**Architecture:** Register `code-reuse` as a language-agnostic review dimension,
add a focused reviewer prompt, and update output rules so confirmed findings are
shown as `warning` diagnostics while still being verifier improvement-type findings.
Keep the change prompt-and-schema driven; do not add a repository index or runtime code.

**Tech Stack:** Markdown prompt files, YAML schema documentation, JSON schema,
mdbook documentation, existing Open Ultrareview orchestrator workflow.

---

## File Structure

- Create `skills/open-ultrareview/prompts/code-reuse.md`.
  This file defines the new reviewer dimension.
- Modify `skills/open-ultrareview/config-schema.yml`.
  This registers the dimension and updates output documentation.
- Modify `skills/open-ultrareview/schemas/finding.json`.
  This allows reviewer subagents to return `dimension: "code-reuse"`.
- Modify `skills/open-ultrareview/orchestrator.md`.
  This updates dimension counts and preserves warning severity for `code-reuse`.
- Modify `skills/open-ultrareview/templates/report.md`.
  This shows severity for improvement-type findings,
  so warning-level `code-reuse` results are visible in reports.
- Modify `README.md`.
  This documents the new review area.
- Modify `docs/knowledge-base/src/SUMMARY.md`.
  This links the new spec and plan in the mdbook navigation.

Do not modify `~/.agents/skills/open-ultrareview/` directly.
The normal setup uses a symlink.

Do not commit unless the user explicitly requests a commit.

---

### Task 1: Baseline Checks

**Files:**
- Inspect: `skills/open-ultrareview/config-schema.yml`
- Inspect: `skills/open-ultrareview/schemas/finding.json`
- Inspect: `skills/open-ultrareview/orchestrator.md`

- [ ] **Step 1: Confirm `code-reuse` is not already registered**

Run:

```bash
rg "code-reuse" skills/open-ultrareview README.md
```

Expected before implementation:

```text
No matches in runtime skill files or README.
```

- [ ] **Step 2: Confirm existing dimension count references**

Run:

```bash
rg "10|dimensions|include_improvements|Bridge field|improvement" \
  skills/open-ultrareview/orchestrator.md \
  skills/open-ultrareview/config-schema.yml \
  skills/open-ultrareview/templates/report.md
```

Expected before implementation:

```text
The files contain the existing 10-dimension defaults,
the improvement output rules,
and the bridge severity mapping that must be updated.
```

---

### Task 2: Register The Dimension

**Files:**
- Modify: `skills/open-ultrareview/config-schema.yml`
- Modify: `skills/open-ultrareview/schemas/finding.json`

- [ ] **Step 1: Add `code-reuse` to the config schema dimension list**

In `skills/open-ultrareview/config-schema.yml`, add this valid value under the
language-agnostic dimensions:

```yaml
    - code-reuse          # language: any
```

Add this language map entry:

```yaml
    code-reuse: any
```

Update nearby comments so they no longer describe only 10 dimensions.

- [ ] **Step 2: Update output configuration wording**

In `skills/open-ultrareview/config-schema.yml`, change the `include_improvements`
description from info-only wording to maintainability wording:

```yaml
      Include improvement and maintainability findings.
      Most improvement dimensions emit info diagnostics;
      code-reuse emits warning diagnostics.
      Set false to only see correctness bug findings.
```

- [ ] **Step 3: Add `code-reuse` to the finding schema enum**

In `skills/open-ultrareview/schemas/finding.json`, add the dimension to the enum:

```json
"code-reuse"
```

Place it near the other improvement dimensions.

- [ ] **Step 4: Verify registration strings**

Run:

```bash
rg "code-reuse" skills/open-ultrareview/config-schema.yml skills/open-ultrareview/schemas/finding.json
```

Expected after this task:

```text
`config-schema.yml` contains code-reuse in valid_values and language_map.
`finding.json` contains code-reuse in the dimension enum.
```

---

### Task 3: Add The Reviewer Prompt

**Files:**
- Create: `skills/open-ultrareview/prompts/code-reuse.md`

- [ ] **Step 1: Create the prompt file**

Create `skills/open-ultrareview/prompts/code-reuse.md` with this content:

```markdown
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
```

- [ ] **Step 2: Verify prompt exists and uses warning severity**

Run:

```bash
rg "warning|reuse|adapt" skills/open-ultrareview/prompts/code-reuse.md
```

Expected:

```text
The prompt explicitly instructs the reviewer to use warning severity
and to cite concrete existing code for reuse or adaptation.
```

---

### Task 4: Update Orchestrator Output Rules

**Files:**
- Modify: `skills/open-ultrareview/orchestrator.md`
- Modify: `skills/open-ultrareview/templates/report.md`

- [ ] **Step 1: Update dimension counts and examples**

In `skills/open-ultrareview/orchestrator.md`, update default dimension counts:

```markdown
- `dimensions`: all 11 enabled
```

In the Rust auto-enabled example,
include `code-reuse` and update the count:

```text
Auto-enabled dimensions: logic-bugs, concurrency,
  error-handling, security, api-contracts, performance,
  architecture, best-practices, rust-improvements,
  arch-improvements, code-reuse (11/11)
```

In the non-Rust auto-enabled example,
include `code-reuse` and update the count:

```text
Auto-enabled dimensions: logic-bugs, concurrency,
  error-handling, security, api-contracts, performance,
  architecture, arch-improvements, code-reuse (9/11)
```

- [ ] **Step 2: Preserve `warning` output for `code-reuse`**

Update the bridge mapping table so severity is no longer forced to `info`
for all improvement findings:

```markdown
| `severity` | `error` for bug findings, `warning` for `code-reuse`, `info` for other improvement findings |
```

Add an explicit diagnostic format for `code-reuse` findings:

````markdown
For code reuse findings:

```text
<file>:<line>:<col>: warning[code-reuse]: <title>
  = rationale: <rationale>
  = suggestion: <suggestion>
  = verified-by: <verifier-model> (independent confirmation)
```
````

Keep other improvement dimensions as `info`.

- [ ] **Step 3: Keep verifier classification improvement-type**

In the verification instructions,
add a short note that `code-reuse` is verified as an improvement-type finding
even though final output preserves warning severity:

```markdown
      Treat `code-reuse` as an improvement-type finding for verification;
      its final diagnostic severity remains `warning`.
```

- [ ] **Step 4: Show severity in improvement report entries**

In `skills/open-ultrareview/templates/report.md`, add severity to the
confirmed improvement entry:

```markdown
- **Severity:** {severity}
```

Place it after the file line.

- [ ] **Step 5: Verify orchestrator wording**

Run:

```bash
rg "all 11|code-reuse|warning\[code-reuse\]|error` for bug" \
  skills/open-ultrareview/orchestrator.md \
  skills/open-ultrareview/templates/report.md
```

Expected:

```text
The orchestrator documents 11 dimensions,
the code-reuse warning diagnostic format,
and the bridge warning severity exception.
```

---

### Task 5: Update Documentation Navigation And README

**Files:**
- Modify: `README.md`
- Modify: `docs/knowledge-base/src/SUMMARY.md`

- [ ] **Step 1: Update README dimension summary**

In `README.md`, update the reviewer-area sentence so it includes code reuse:

```markdown
- Reviewer agents inspect the diff in parallel across areas such as logic bugs,
  concurrency, security, error handling, performance, API contracts,
  architecture, and code reuse.
```

- [ ] **Step 2: Add the new spec and plan to mdbook navigation**

In `docs/knowledge-base/src/SUMMARY.md`, add links for the new files:

```markdown
- [AI Agents](ai-agents/README.md)
  - [Specs](ai-agents/specs/2026-05-20-mandatory-review-skills-design.md)
  - [Code Reuse Reviewer Spec](ai-agents/specs/2026-06-03-code-reuse-reviewer-design.md)
  - [Plans](ai-agents/plans/2026-05-20-mandatory-review-skills-plan.md)
  - [Code Reuse Reviewer Plan](ai-agents/plans/2026-06-03-code-reuse-reviewer-plan.md)
```

- [ ] **Step 3: Verify docs links**

Run:

```bash
rg "code reuse|Code Reuse Reviewer" README.md docs/knowledge-base/src/SUMMARY.md
```

Expected:

```text
README mentions code reuse in the reviewer area list.
SUMMARY.md links both the new spec and the new plan.
```

---

### Task 6: Final Verification

**Files:**
- Verify: all modified files

- [ ] **Step 1: Check repository diff**

Run:

```bash
git diff -- skills/open-ultrareview README.md docs/knowledge-base/src
```

Expected:

```text
Diff is limited to the new prompt,
schema/config registration,
orchestrator/report wording,
README,
and docs navigation/spec/plan files.
```

- [ ] **Step 2: Build Rust tests**

Run:

```bash
cargo test
```

Expected:

```text
All Rust tests pass.
```

- [ ] **Step 3: Build the knowledge base**

Run:

```bash
mdbook build docs/knowledge-base/
```

Expected:

```text
The mdbook build completes successfully.
```

- [ ] **Step 4: Run formatting/lint checks if available**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
```

Expected:

```text
Formatting is unchanged and clippy reports no new issues.
```

- [ ] **Step 5: Confirm no direct installed-copy edits**

Run:

```bash
git status --short
```

Expected:

```text
No files under ~/.agents are listed.
Repository changes are limited to the intended files.
```
