# ultrareview

Ultrareview combines a multi-agent code review skill with an editor diagnostics bridge.

This repository contains:

- `skills/ultrareview/`, the agent skill that runs the review workflow
- `ultrareview-bridge`, a binary that publishes confirmed findings to editors

## Bridge

The bridge turns AI-generated code findings into editor diagnostics.

A single binary that runs:

- An MCP server for AI agents to post findings
- An LSP server over stdio for editors to display diagnostics

Findings appear as inline squiggles in your editor in real time.

## Install

```bash
cargo install --path .
```

## Usage

### Start With Your Editor

The editor launches the binary as a language server:

```bash
ultrareview-bridge lsp --port 19999
```

This starts the LSP server on stdio for the editor and the MCP server on `http://127.0.0.1:19999/mcp` for AI agents.

### Zed Configuration

Zed does not allow arbitrary language server names under the top-level `lsp` setting.
That setting only configures language server adapters already registered by Zed
or by an installed Zed extension.

This means the following settings-only configuration is invalid and Zed will reject it:

```json
{
  "lsp": {
    "ultrareview-bridge": {
      "binary": {
        "path": "ultrareview-bridge",
        "arguments": ["lsp"]
      }
    }
  }
}
```

To use this bridge with Zed, install a Zed extension that registers
`ultrareview-bridge` as a language server adapter.
The extension should return this command from `language_server_command`:

```text
ultrareview-bridge lsp --port 19999
```

This repository includes that extension in `zed-extension/`.
Install it as a local dev extension from Zed:

1. Run `cargo install --path .` from this repository.
2. In Zed, run `zed: install dev extension` from the command palette.
3. Select this repository's `zed-extension/` directory.
4. Add `ultrareview-bridge` to each language where you want diagnostics.
   For Rust and Markdown:

```json
{
  "languages": {
    "Rust": {
      "language_servers": ["rust-analyzer", "ultrareview-bridge", "..."]
    },
    "Markdown": {
      "language_servers": ["ultrareview-bridge", "..."]
    }
  }
}
```

If the binary is not on Zed's `PATH`, point the extension at it explicitly:

```json
{
  "lsp": {
    "ultrareview-bridge": {
      "binary": {
        "path": "/absolute/path/to/ultrareview-bridge",
        "arguments": ["lsp", "--port", "19999"],
        "env": {
          "RUST_LOG": "ultrareview_bridge=debug"
        }
      }
    }
  }
}
```

After startup, `Zed.log` should contain both a process launch line and an LSP message:

```text
starting language server process. binary path: "ultrareview-bridge"
ultrareview-bridge LSP initialized
```

You can also run the MCP side manually for agents without Zed:

```bash
ultrareview-bridge mcp --port 19999
```

### Post Findings From AI Agents

Any MCP-compatible AI tool can call:

- `post_findings` to append findings incrementally
- `clear_findings` to clear findings before a new run
- `restore_findings` to undismiss all findings

### Dismiss Findings

Click on a diagnostic in your editor and use the code action "Dismiss this finding" to hide it.

Dismissed findings stay dismissed across re-runs.

## Architecture

```text
AI agents -> MCP HTTP :19999/mcp -> FindingsStore -> LSP stdio -> Editor
```

## Ultrareview Skill

The bridge-aware `ultrareview` agent skill lives in `skills/ultrareview/`.
The skill runs the multi-agent review workflow and,
when ultrareview-bridge MCP tools are available,
posts confirmed findings back to the bridge with `source: "ultrareview"`.

Use it as the producer side of the bridge workflow:

```text
/ultrareview
```
