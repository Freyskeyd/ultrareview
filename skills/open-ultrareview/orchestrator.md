# Open Ultrareview Orchestrator

Follow these phases exactly, in order.
Do not skip phases. Do not reorder.

## Phase 1: Parse Arguments & Load Config

1. Parse `$ARGUMENTS` for:
   - `--reviewer=<model>` (optional)
   - `--verifier=<model>` (optional)
   - Remaining positional argument as `<base-ref>` (optional)

2. Check if `.open-ultrareview.yml` exists at the project root.
   If it exists, read it and use its values as defaults.

3. Apply CLI overrides on top of config file values.
   Final defaults if nothing is specified:
   - `reviewer.model`: `claude-opus-4-6`
   - `reviewer.max_parallel`: `10`
   - `verifier.model`: `gpt-5.5`
   - `dimensions`: all 10 enabled
   - `mandatory_skills.project`: `[]`
   - `mandatory_skills.languages`: `{}`
   - `output.format`: `diagnostics`
   - `output.report`: `true`
   - `output.include_improvements`: `true`
   - `output.verbose`: `false`
   - `output.bridge`: `auto`

4. If `reviewer.model` equals `verifier.model`, emit a warning:

   ```
   warning: reviewer and verifier use the same model (<model>).
   Cross-model verification is more effective with different models.
   Proceeding anyway.
   ```

## Phase 2: Diff Extraction & Context Gathering

1. Determine the base ref:
   - If `<base-ref>` was provided, use it.
   - Otherwise, detect the default branch:
     `git symbolic-ref refs/remotes/origin/HEAD | sed 's@^refs/remotes/origin/@@'`
   - If that fails, try `main`, then `master`.

2. Compute the diff:

   ```bash
   git diff <base-ref>..HEAD
   ```

   Also include uncommitted and staged changes:

   ```bash
   git diff HEAD
   ```

   Merge both diffs into a combined changeset.

3. If the combined diff is empty, exit early:

   ```
   No changes to review.
   ```

4. Extract the list of changed files from the diff.

5. For each changed file, read its full content
   (the current working tree version).
   This gives reviewers surrounding context beyond the diff hunks.

6. **Detect primary language** from changed file extensions:
   - `.rs` → rust
   - `.py` → python
   - `.ts`, `.tsx` → typescript
   - `.js`, `.jsx` → javascript
   - `.go` → go
   - `.java` → java
   - `.cpp`, `.cc`, `.h`, `.hpp` → cpp
   - Other or mixed → any

   If the diff contains multiple languages,
   use the language with the most changed files.
   If tied, treat as "any".

7. **Load mandatory skills:**
   - Build the ordered skill list from:
     - `mandatory_skills.project`
     - `mandatory_skills.languages.<detected-language>`
     - `mandatory_skills.languages.any`
   - De-duplicate skill names while preserving the first configured order.
   - Load each mandatory skill before resolving dimensions or dispatching reviewers.
   - Extract the review-relevant guidance from each loaded mandatory skill
     into a **mandatory skill guidance bundle**.
     This bundle is the propagation mechanism for reviewer subagents,
     which start with fresh context and do not automatically inherit
     the orchestrator's loaded skill context.
   - If a skill cannot be loaded, stop the review and print:

     ```
     error: mandatory skill '<skill>' could not be loaded.
     Configure .ultrareview.yml with installed skill names only.
     ```

   - Report loaded mandatory skills:

     ```
     Mandatory skills: tech-rust, shadcn
     ```

     If no mandatory skills are configured, omit this line.

8. **Resolve dimensions:**
   - If `dimensions` is set to `auto` (the default),
     enable all dimensions whose `language` in the language_map
     is either `any` or matches the detected language.
   - If `dimensions` is an explicit list in config,
     use that list as-is (no language filtering).
   - Report which dimensions were auto-selected and why:

     ```
     Detected language: rust (from 12 .rs files)
     Auto-enabled dimensions: logic-bugs, concurrency,
       error-handling, security, api-contracts, performance,
       architecture, best-practices, rust-improvements,
       arch-improvements (10/10)
     ```

     Or for a non-Rust project:

     ```
     Detected language: typescript (from 8 .ts files)
     Auto-enabled dimensions: logic-bugs, concurrency,
       error-handling, security, api-contracts, performance,
       architecture, arch-improvements (8/10)
     Skipped: best-practices (rust-only), rust-improvements (rust-only)
     ```

9. Prepare the **review context bundle**:
   - The combined diff (full text)
   - For each changed file: path, full content, and language/extension
   - The mandatory skill guidance bundle from Phase 2, step 7
   - Total line count and file count (for progress reporting)

10. Report to the user:

   ```
   Open Ultrareview: reviewing N files, ~M lines changed
   Reviewer model: <model> | Verifier model: <model>
   Dimensions: <comma-separated list> (K enabled)
   ```

## Phase 3: Parallel Review

1. For each enabled dimension,
   prepare a reviewer prompt by concatenating:
   - The appropriate preamble
     (`prompts/_preamble.md` for bug dimensions,
      `prompts/_preamble-improvements.md` for improvement dimensions)
   - The dimension-specific prompt (`prompts/<dimension>.md`)
   - The mandatory skill guidance bundle, when any mandatory skills were loaded
   - The finding JSON schema from `schemas/finding.json`
   - The review context bundle from Phase 2

2. Dispatch up to `max_parallel` reviewer subagents
   using the Task tool.
   Each subagent receives its concatenated prompt
   and must return a JSON object matching the finding schema.

   Dispatch all independent reviewers in a single message
   with multiple Task tool calls to maximize parallelism.

3. For each subagent that returns successfully,
   parse its JSON response.
   If a subagent fails or times out, log a warning:

   ```
   warning: <dimension> reviewer failed: <error>. Skipping.
   ```

   Continue with remaining results.

4. Collect all findings into a single list.
   Tag each finding with its source dimension.

## Phase 4: Deduplication

This phase uses your judgment as the orchestrator.
You are an LLM reading structured findings --
use semantic understanding, not string matching.

1. Sort findings by (file, line).

2. For each cluster of findings at the same file
   and same or adjacent lines (within 3 lines),
   determine whether they describe the same underlying issue
   or distinct issues. Use the following judgment criteria:

   **Same issue (merge):**
   - Two findings point to the same root cause
     even if described differently
     (e.g., "unchecked Result" from error-handling
     and "silent failure" from logic-bugs).
   - When merging: keep the finding with higher severity,
     preserve the richer evidence/rationale,
     add an `also_flagged_by` field listing the other dimensions.

   **Distinct issues (keep both):**
   - Two findings at the same line describe
     genuinely different problems
     (e.g., "race condition" and "missing error handling"
     on the same function call).
   - Two findings at adjacent lines in the same dimension
     are likely distinct issues -- keep both.

   **When uncertain:** keep both. False negatives in dedup
   (showing a duplicate) are less harmful
   than false positives (hiding a real finding).

3. Assign each deduplicated finding a unique ID:
   `<dimension>-<index>`
   (e.g., `concurrency-0`, `logic-bugs-1`).

## Phase 5: Verification

1. Group deduplicated findings by file.

2. For each file group, prepare a verifier prompt by concatenating:
   - The verifier prompt template (`prompts/verifier.md`)
   - The full file content
   - For each finding in the group:
     only the finding ID, line, column, and finding type
     (bug or improvement).
     Do NOT include the dimension, title, evidence,
     rationale, or suggestion.
     The verifier must not know which dimension flagged the location.
     This ensures truly independent verification.
   - The verdict JSON schema from `schemas/verdict.json`

3. Dispatch verifier subagents using the Task tool,
   one per file group.
   Use the verifier model (not the reviewer model).

   Dispatch all independent verifier calls in a single message
   with multiple Task tool calls to maximize parallelism.

4. For each verifier response, parse the verdict JSON.
   If a verifier fails, mark all findings in that batch
   as `unverified`:

   ```
   warning: verifier failed for <file>: <error>.
   Findings marked as unverified.
   ```

## Phase 6: Filtering & Output

1. Apply filtering rules to each finding based on verdict:

   | Verdict   | Confidence | Action                              |
   |-----------|------------|-------------------------------------|
   | confirmed | high       | Emit as diagnostic                  |
   | confirmed | medium     | Emit with `[medium confidence]` tag |
   | confirmed | low        | Report only                         |
   | uncertain | any        | Report only, flagged as unverified  |
   | rejected  | any        | Report only if `verbose: true`      |
   | unverified| n/a        | Report only, flagged as unverified  |

2. If `include_improvements` is `false`,
   suppress all findings from improvement dimensions
   regardless of verdict.

3. If `output.bridge` is `auto`
   and open-ultrareview-bridge MCP tools are available,
   publish confirmed findings to the bridge before printing diagnostics:

   - Call `clear_findings` with:
     - `project`: the absolute project root from Phase 2
     - `source`: `open-ultrareview`
   - Call `post_findings` with:
     - `project`: the same absolute project root
     - `source`: `open-ultrareview`
     - `findings`: all confirmed high-confidence and medium-confidence findings
       that will be emitted as diagnostics

   Map each open-ultrareview finding to a bridge finding as follows:

   | Bridge field | Open Ultrareview value |
   |--------------|-------------------|
   | `file` | relative file path |
   | `line` | 1-indexed line |
   | `col` | 1-indexed column, defaulting to `1` when missing |
   | `severity` | `error` for bug findings, `info` for improvement findings |
   | `category` | review dimension |
   | `title` | finding title, including `[medium confidence]` when applicable |
   | `evidence` | bug evidence, or `null` for improvements |
   | `rationale` | improvement rationale, or `null` for bugs |
   | `suggestion` | suggested fix, if present |
   | `verified_by` | verifier model |

   If bridge posting fails,
   print a warning and continue with textual diagnostics and report generation.
   Do not fail the review solely because editor diagnostic publishing failed.

4. Emit confirmed findings as compiler-style diagnostics:

   For bug findings:

   ```
   <file>:<line>:<col>: <severity>[<dimension>]: <title>
     = evidence: <evidence>
     = suggestion: <suggestion>
     = verified-by: <verifier-model> (independent confirmation)
   ```

   For improvement findings:

   ```
   <file>:<line>:<col>: info[<dimension>]: <title>
     = rationale: <rationale>
     = suggestion: <suggestion>
     = verified-by: <verifier-model> (independent confirmation)
   ```

   For medium-confidence findings, append `[medium confidence]`
   after the dimension tag.

5. After diagnostics, print a summary line:

   ```
   Open Ultrareview complete: X confirmed findings
   (Y bugs, Z improvements) across N dimensions.
   W findings rejected by verifier.
   ```

6. If `output.report` is `true`,
   generate a Markdown report using `templates/report.md`
   and save it to `.open-ultrareview-report.md` in the project root.
   Report to the user:

   ```
   Full report saved to .open-ultrareview-report.md
   ```

7. If all findings were rejected or no findings were reported:

   ```
   Open Ultrareview complete: no issues found across N dimensions.
   ```
