---
name: ultrareview
description: >
  Use when reviewing code changes with multiple specialized agents,
  independent verification, and optional ultrareview-bridge diagnostics.
argument-hint: "--reviewer=<model> --verifier=<model> <base-ref>"
---

# Ultrareview

Deep, multi-agent code review that runs locally.
Specialized reviewer agents analyze your diff in parallel
across multiple dimensions (logic bugs, concurrency, security, etc.).
A verifier agent using a different model
independently confirms each finding before it is reported.
When ultrareview-bridge MCP tools are available,
confirmed findings are also posted to the bridge
so editors can show them as diagnostics.

## Invocation

The skill accepts optional arguments:

- `--reviewer=<model>` -- model for reviewer agents
  (default: `claude-sonnet-4`)
- `--verifier=<model>` -- model for verifier agents
  (default: `gpt-4o`)
- `<base-ref>` -- git ref to diff against
  (default: default branch)

Examples:

```text
/ultrareview
/ultrareview --reviewer=claude-sonnet-4 --verifier=o3
/ultrareview main
/ultrareview --reviewer=claude-opus-4 origin/develop
```

## Configuration

If `.ultrareview.yml` exists at the project root,
it is read for defaults.
CLI arguments override config file values.
See [config-schema.yml](config-schema.yml) for the full schema.

## Execution

When this skill is invoked,
follow the orchestrator instructions in [orchestrator.md](orchestrator.md).
The orchestrator document contains the complete step-by-step procedure
for running the review pipeline.

Do not skip any phase.
Do not modify the pipeline order.
Follow orchestrator.md exactly.
