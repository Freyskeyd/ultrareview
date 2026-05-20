# Mandatory Review Skills Design

## Goal

Allow projects to require specific agent skills during Ultrareview runs.
The configuration should support project-wide mandatory skills and language-specific mandatory skills.

## Current Context

Ultrareview already reads `.ultrareview.yml` from the project root.
The orchestrator detects the primary language from changed file extensions,
then uses that language to select review dimensions.

The new behavior should extend this existing config flow rather than add a second prompt system.

## Recommended Approach

Use skill names in `.ultrareview.yml`.
Skill names match installed agent runtime skills,
which keeps the feature explicit and portable across projects that already depend on skills.

The supported config shape is:

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

## Behavior

The orchestrator loads mandatory skills after parsing config and detecting the primary language.
It loads all entries under `mandatory_skills.project`.
It then loads entries under `mandatory_skills.languages.<detected-language>`.
It also loads entries under `mandatory_skills.languages.any`.

The orchestrator de-duplicates skill names while preserving their first configured order.
After loading skills,
it extracts their review-relevant guidance into a mandatory skill guidance bundle.
That bundle is included in reviewer subagent prompts,
because subagents start with fresh context and do not automatically inherit
the orchestrator's loaded skill context.

If a mandatory skill is unavailable or fails to load,
the review fails fast with a clear message.
The failure is intentional because the user marked the skill mandatory.

## Non-Goals

This feature does not add inline prompt files.
It does not make bootstrap process skills like `using-superpowers` language-specific review guidance.
It does not change review dimensions,
verification,
deduplication,
or bridge output.

## Testing

This repository primarily stores the Ultrareview workflow as Markdown and schema files.
The change should be verified by reviewing the modified orchestrator instructions,
schema documentation,
and README examples for consistency.
