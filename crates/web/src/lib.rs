use neopilot_models_core::Engine;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

mod server;

pub use server::{Server, AppState};

pub struct WebServer {
    engine: Arc<Engine>,
    static_dir: Option<PathBuf>,
}

impl WebServer {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self {
            engine,
            static_dir: None,
        }
    }

    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    pub fn build(self) -> Server {
        Server::new(self.engine, self.static_dir)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub port: u16,
    pub static_dir: Option<PathBuf>,
    pub providers_dir: PathBuf,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 3000,
            static_dir: None,
            providers_dir: PathBuf::from("providers"),
            log_level: "info".to_string(),
        }
    }
}

pub async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Neopilot Models web server");

    // Initialize engine
    let mut engine = Engine::new();
    engine.load_from_directory(&config.providers_dir).await?;
    
    let engine = Arc::new(engine);
    
    info!("Loaded {} providers with {} models", 
          engine.get_database().provider_count(),
          engine.get_database().model_count());

    // Build and run server
    let server = WebServer::new(engine)
        .with_static_dir(config.static_dir.clone())
        .build();

    let addr = format!("{}:{}", config.bind_address, config.port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    server.run(addr).await?;
    
    Ok(())
}
