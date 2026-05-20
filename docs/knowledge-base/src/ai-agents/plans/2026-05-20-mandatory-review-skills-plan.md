# Mandatory Review Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `.ultrareview.yml` support for mandatory project and language skills.

**Architecture:** Extend the existing Ultrareview skill configuration documentation and orchestrator phases.
The feature remains in the review skill layer and does not require Rust bridge changes.

**Tech Stack:** Markdown skill docs, YAML schema documentation, Ultrareview agent workflow.

---

## File Structure

- Modify `skills/ultrareview/config-schema.yml` to document the `mandatory_skills` section.
- Modify `skills/ultrareview/orchestrator.md` to load configured mandatory skills after language detection.
- Modify `README.md` to show the new `.ultrareview.yml` usage.

### Task 1: Document Config Schema

**Files:**

- Modify: `skills/ultrareview/config-schema.yml`

- [ ] **Step 1: Add the schema block**

Add this block after the `dimensions.language_map` section and before `output:`:

```yaml
mandatory_skills:
  type: object
  default:
    project: []
    languages: {}
  description: >
    Mandatory agent skills to load before dispatching reviewers.
    Project skills are always loaded.
    Language skills are loaded when their key matches the detected primary language.
    If a mandatory skill cannot be loaded, the review fails fast.
  project:
    type: array
    default: []
    items: string
    description: >
      Skill names to load for every review run in this project.
  languages:
    type: object
    default: {}
    additional_properties:
      type: array
      items: string
    description: >
      Map of detected language name to skill names.
      Supported keys are the same language names used by primary language detection,
      such as rust, python, typescript, javascript, go, java, cpp, and any.
```

- [ ] **Step 2: Review formatting**

Run: `git diff -- skills/ultrareview/config-schema.yml`

Expected: the new schema block appears between `dimensions` and `output`.

### Task 2: Update Orchestrator Flow

**Files:**

- Modify: `skills/ultrareview/orchestrator.md`

- [ ] **Step 1: Add defaults to Phase 1**

In the final defaults list, add:

```markdown
     - `mandatory_skills.project`: `[]`
     - `mandatory_skills.languages`: `{}`
```

- [ ] **Step 2: Insert mandatory skill loading after language detection**

Add this as a new step after primary language detection and before dimension resolution:

```markdown
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
```

- [ ] **Step 3: Renumber following steps**

Renumber `Resolve dimensions`, `Prepare the review context bundle`, and `Report to the user`
so Phase 2 remains sequential.
Add the mandatory skill guidance bundle to the review context bundle.
Add the mandatory skill guidance bundle to each reviewer prompt in Phase 3.

- [ ] **Step 4: Review diff**

Run: `git diff -- skills/ultrareview/orchestrator.md`

Expected: mandatory skill loading happens after detected language is known and before reviewers are prepared.

### Task 3: Add README Example

**Files:**

- Modify: `README.md`

- [ ] **Step 1: Extend the Ultrareview config paragraph**

Replace the paragraph after the invocation examples with:

````markdown
If `.ultrareview.yml` exists at the project root, the skill reads it for defaults.
Command-line arguments override config values.
Projects can also require review skills to be loaded before reviewers run:

```yaml
mandatory_skills:
  project:
    - tech-rust
  languages:
    rust:
      - tech-rust
    typescript:
      - shadcn
```

Mandatory skills fail fast if they are not installed in the active agent environment.
````

- [ ] **Step 2: Review diff**

Run: `git diff -- README.md`

Expected: README documents the new config with skill names only.

### Task 4: Verify Documentation Consistency

**Files:**

- Verify: `skills/ultrareview/config-schema.yml`
- Verify: `skills/ultrareview/orchestrator.md`
- Verify: `README.md`

- [ ] **Step 1: Search for mandatory skill references**

Run: `rg "mandatory_skills|Mandatory skills|mandatory skill" skills/ultrareview README.md`

Expected: matches appear in schema, orchestrator, and README.

- [ ] **Step 2: Check repository diff**

Run: `git diff -- skills/ultrareview/config-schema.yml skills/ultrareview/orchestrator.md README.md`

Expected: only documentation and workflow instructions changed.
