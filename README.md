# Ultrareview

> AI code review that shows up as editor squiggles.

Ultrareview turns multi-agent AI code review into diagnostics you can see where you
already work: your editor.
Specialized reviewer agents inspect a change, a second model verifies their findings,
and confirmed issues flow through MCP and LSP into inline diagnostics.

The core promise is simple:
reviewed by multiple agents, independently verified, and displayed where developers
already work.

This repository contains three pieces that work together:

- `skills/ultrareview/`, the multi-agent review skill for AI coding tools.
- `ultrareview-bridge`, a Rust binary that exposes MCP tools and an LSP server.
- `zed-extension/`, a local Zed extension that registers the bridge as a language server.

## Why Ultrareview?

Ordinary AI code review often stops at a chat transcript or pull request comment.
That makes findings easy to miss, hard to triage, and disconnected from the code you
are currently editing.

Ultrareview is different because it is built around the review loop, not just the model
response:

- Multiple specialized reviewer agents inspect the same diff from different angles.
- A verifier agent, using a different model, confirms findings before they are reported.
- Confirmed findings are published through MCP so compatible AI tools can update them.
- The bridge serves those findings through LSP so editors can render diagnostics.
- Dismissed findings stay dismissed across re-runs, so review noise stays controlled.

Use it when you want AI review to feel less like a one-off prompt and more like an
editor-native development tool.

## How It Works

Ultrareview separates review, transport, storage, and display.
The skill produces verified findings.
The bridge stores them and exposes them to editors.

```text
+----------------------+        +------------------------+
| AI coding tool       |        | Editor                 |
| /ultrareview skill   |        | Zed or any LSP client  |
+----------+-----------+        +-----------+------------+
           |                                ^
           | MCP tools                      | LSP diagnostics
           v                                |
+----------+--------------------------------+------------+
| ultrareview-bridge                                     |
|                                                        |
|  MCP HTTP :19999/mcp  ->  FindingsStore  ->  LSP stdio |
+--------------------------------------------------------+
```

In practice, this means a verified issue can move from an AI review agent to a red
squiggle without copying text between tools.

## Quickstart

Install the bridge from this repository:

```bash
cargo install --path .
```

Configure your editor or extension to launch the bridge as a stdio LSP server:

```bash
ultrareview-bridge lsp --port 19999
```

When launched this way, the process starts the LSP server on stdio for the editor and
the MCP server at
`http://127.0.0.1:19999/mcp` for AI agents.

You can also run only the MCP side for agents without launching an editor language
server:

```bash
ultrareview-bridge mcp --port 19999
```

The bridge is useful once an MCP-compatible AI tool connects to it and posts findings.
The included `ultrareview` skill is the intended producer.

## Zed Setup

Zed does not allow arbitrary language server names under the top-level `lsp` setting.
That setting only configures language server adapters already registered by Zed or by
an installed Zed extension.

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

To use Ultrareview with Zed, install the local extension in `zed-extension/`.
The extension registers `ultrareview-bridge` as a language server adapter and returns
this command from `language_server_command`:

```text
ultrareview-bridge lsp --port 19999
```

Install and enable it locally:

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

Once findings are posted through MCP, they appear as editor diagnostics.
Click a diagnostic and use the available code action to dismiss it when it is no
longer useful.

## Ultrareview Skill

The bridge-aware `ultrareview` agent skill lives in `skills/ultrareview/`.
It runs a multi-agent review workflow locally:

- Reviewer agents inspect the diff in parallel across areas such as logic bugs,
  concurrency, security, error handling, performance, API contracts, and architecture.
- A verifier agent independently checks each candidate finding.
- Confirmed findings are reported to the user.
- When the bridge MCP tools are available, confirmed findings are also posted with
  `source: "ultrareview"`.

Invoke it from a compatible agent environment:

```text
/ultrareview
```

The skill also accepts optional arguments for reviewer model, verifier model, and base
git ref:

```text
/ultrareview --reviewer=claude-sonnet-4 --verifier=o3
/ultrareview main
```

If `.ultrareview.yml` exists at the project root, the skill reads it for defaults.
Command-line arguments override config values.

## MCP Tools

Any MCP-compatible AI tool can connect to the bridge and call these tools:

- `post_findings` appends findings incrementally.
- `clear_findings` clears findings for a project/source pair before a new run.
- `restore_findings` undismisses all findings for a project.

Findings posted through MCP are stored by the bridge and served as LSP diagnostics to
the connected editor.

## Dismiss Findings

Ultrareview is designed for repeated review runs, so findings need lifecycle controls.

Click on a diagnostic in your editor and use the code action `Dismiss this finding` to
hide it.
Dismissed findings stay dismissed across re-runs.

If you want to bring dismissed findings back, call `restore_findings` through MCP.

## For Maintainers

Ultrareview is aimed at maintainers who want stronger review coverage without turning
every contribution into a long prompt-management session.

It helps with:

- Catching issues before they become pull request review churn.
- Keeping findings close to the changed files.
- Separating candidate findings from independently verified findings.
- Giving contributors editor-native feedback instead of detached review text.

Star the repository if you want editor-native AI review to become easier to install,
share, and standardize across projects.
Watch it if you care about MCP, LSP, Zed integration, or multi-agent review workflows.

## For Power Users

Ultrareview is for AI coding power users who already run agents locally and want those
agents to leave useful traces in the editor.

The bridge is intentionally small:

- MCP in, so agents can publish findings.
- LSP out, so editors can display diagnostics.
- Local storage, so dismissals survive repeated runs.
- Plain Rust binary, so it is easy to install and inspect.

The current editor path is Zed through the included local extension.
Other LSP-capable editors can integrate by configuring
`ultrareview-bridge lsp --port 19999` as a stdio language server, then connecting
agents to the MCP endpoint started by that process.

## Roadmap

Near-term areas worth contributing to:

- Smoother editor setup for more LSP-capable editors.
- Better finding lifecycle controls for repeated review runs.
- Richer diagnostic metadata and code actions.
- Packaged releases for easier installation.
- More examples of agent integrations that publish findings through MCP.

## Contributing

Contributions are welcome, especially from people who use AI coding tools daily and
want better review ergonomics.

Good first contribution areas include:

- Trying the Zed setup and reporting rough edges.
- Improving MCP client compatibility.
- Hardening LSP diagnostic behavior across languages.
- Improving the `ultrareview` skill prompts and verifier workflow.
- Packaging the bridge for easier installation.

Please keep changes focused.
The bridge should stay understandable, local-first, and easy to run.

## Repository Metadata

Suggested GitHub description:

```text
Multi-agent AI code review, independently verified and displayed as editor diagnostics through MCP and LSP.
```

Suggested topics:

```text
ai-code-review, mcp, lsp, zed-extension, developer-tools, code-review,
multi-agent, rust, editor-diagnostics, ai-agents
```

## License

MIT
