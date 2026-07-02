use autoforge_api::{router, ApiState};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "autoforge", about = "AI 외주 자동화 오케스트레이터")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// REST API 서버 시작
    Serve {
        #[arg(long, env = "PORT", default_value = "8080")]
        port: u16,
    },
    /// 워커 프로세스 시작 (Redis consumer)
    Worker {
        #[arg(long, env = "STAGE_FILTER")]
        stage_filter: Option<String>,
    },
    /// 오케스트레이터 (단일 리더)
    Orchestrate,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => {
            let app = router(ApiState {});
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            tracing::info!(%addr, "starting API server");
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Commands::Worker { stage_filter } => {
            tracing::info!(?stage_filter, "starting worker");
            // TODO: Redis consumer loop
            tokio::signal::ctrl_c().await.unwrap();
        }
        Commands::Orchestrate => {
            tracing::info!("starting orchestrator");
            // TODO: advisory lock + event loop
            tokio::signal::ctrl_c().await.unwrap();
        }
    }
}
