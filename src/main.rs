use clap::{Parser, Subcommand};
use tokio_util::sync::CancellationToken;
use tracing::info;
use ultrareview_bridge::bridge::BridgeState;
use ultrareview_bridge::lsp_server::LspBackend;
use ultrareview_bridge::mcp_server;
use ultrareview_bridge::store::FindingsStore;

#[derive(Parser)]
#[command(name = "ultrareview-bridge")]
#[command(version)]
#[command(about = "Bridge AI code findings to editor diagnostics")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start LSP server on stdio and MCP server on localhost HTTP.
    Lsp {
        /// TCP port for the MCP HTTP server.
        #[arg(long, default_value_t = 19999)]
        port: u16,
    },
    /// Start MCP server only. Without --port, uses stdio transport.
    Mcp {
        /// TCP port for the MCP HTTP server.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Print version.
    Version,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ultrareview_bridge=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().command {
        Commands::Lsp { port } => run_lsp_and_mcp(port).await?,
        Commands::Mcp { port } => run_mcp_only(port).await?,
        Commands::Version => println!("ultrareview-bridge {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}

async fn run_lsp_and_mcp(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bridge = BridgeState::new(FindingsStore::load_from_disk());
    let cancellation = CancellationToken::new();
    let mcp_bridge = bridge.clone();
    let mcp_cancellation = cancellation.clone();

    let mcp_handle = tokio::spawn(async move {
        info!(port, "starting MCP server on http://127.0.0.1:{port}/mcp");
        mcp_server::serve_http(mcp_bridge, port, mcp_cancellation).await
    });

    info!("starting LSP server on stdio");
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = tower_lsp::LspService::new(|client| LspBackend::new(client, bridge));
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;

    cancellation.cancel();
    mcp_handle.abort();
    Ok(())
}

async fn run_mcp_only(port: Option<u16>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bridge = BridgeState::new(FindingsStore::load_from_disk());
    match port {
        Some(port) => {
            info!(port, "starting MCP server on http://127.0.0.1:{port}/mcp");
            mcp_server::serve_http(bridge, port, CancellationToken::new()).await?;
        }
        None => {
            info!("starting MCP server on stdio");
            mcp_server::serve_stdio(bridge).await?;
        }
    }

    Ok(())
}
