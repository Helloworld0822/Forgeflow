use autoforge::{App, Config};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "autoforge", about = "AI 외주 자동화 프로그램", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Actix-web 서버 시작 (기본)
    Serve {
        #[arg(long, env = "HOST")]
        host: Option<String>,
        #[arg(long, env = "PORT")]
        port: Option<u16>,
    },
    /// 파이프라인 워커 (향후 Redis 연동)
    Worker {
        #[arg(long, env = "STAGE_FILTER")]
        stage_filter: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Serve {
        host: None,
        port: None,
    }) {
        Commands::Serve { host, port } => {
            let mut config = Config::from_env();
            if let Some(h) = host {
                config.host = h;
            }
            if let Some(p) = port {
                config.port = p;
            }

            let app = App::new(config)?.shared();
            autoforge::web::serve(app).await?;
        }
        Commands::Worker { stage_filter } => {
            tracing::info!(?stage_filter, "worker mode — Redis consumer coming soon");
            tokio::signal::ctrl_c().await?;
        }
    }

    Ok(())
}
