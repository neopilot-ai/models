use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use neopilot_models_core::{Engine, Result};
use neopilot_models_web::{run_server, Config};
use std::path::PathBuf;
use tracing::{info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};
use console::Style;

#[derive(Parser)]
#[command(
    name = "neopilot-models",
    about = "Neopilot Models CLI - High-performance AI model database",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: Verbosity,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the web server
    Serve {
        /// Bind address (default: 0.0.0.0)
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,

        /// Port (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Path to providers directory
        #[arg(long, default_value = "providers")]
        providers: PathBuf,

        /// Path to static files directory
        #[arg(long)]
        static_dir: Option<PathBuf>,
    },
    /// Validate the model database
    Validate {
        /// Path to providers directory
        #[arg(long, default_value = "providers")]
        providers: PathBuf,

        /// Show detailed validation errors
        #[arg(long)]
        detailed: bool,
    },
    /// Generate statistics about the database
    Stats {
        /// Path to providers directory
        #[arg(long, default_value = "providers")]
        providers: PathBuf,

        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Search for models
    Search {
        /// Search query
        query: String,

        /// Path to providers directory
        #[arg(long, default_value = "providers")]
        providers: PathBuf,

        /// Search by family
        #[arg(long)]
        family: Option<String>,

        /// Search by capability (reasoning, tool_call, attachment, structured_output)
        #[arg(long)]
        capability: Option<String>,

        /// Search by modality (text, image, audio, video, pdf)
        #[arg(long)]
        modality: Option<String>,

        /// Limit results
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Export database to JSON
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to providers directory
        #[arg(long, default_value = "providers")]
        providers: PathBuf,

        /// Pretty print JSON
        #[arg(long)]
        pretty: bool,
    },
    /// Generate model schema
    Schema {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.verbose.into_env_filter()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Serve { bind, port, providers, static_dir } => {
            let config = Config {
                bind_address: bind,
                port,
                static_dir,
                providers_dir: providers,
                log_level: cli.verbose.to_string(),
            };
            
            info!("Starting server on {}:{}", bind, port);
            run_server(config).await?;
        }
        Commands::Validate { providers, detailed } => {
            validate_database(&providers, detailed).await?;
        }
        Commands::Stats { providers, format } => {
            show_statistics(&providers, &format).await?;
        }
        Commands::Search { query, providers, family, capability, modality, limit } => {
            search_models(&query, &providers, family, capability, modality, limit).await?;
        }
        Commands::Export { output, providers, pretty } => {
            export_database(&providers, output, pretty).await?;
        }
        Commands::Schema { output } => {
            generate_schema(output).await?;
        }
    }

    Ok(())
}

async fn validate_database(providers_dir: &PathBuf, detailed: bool) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    pb.set_message("Loading and validating database...");
    
    let mut engine = Engine::new();
    
    match engine.load_from_directory(providers_dir).await {
        Ok(_) => {
            pb.finish_with_message("✅ Database validation passed");
            println!();
            
            let stats = engine.get_statistics();
            let providers_count = stats["providers"].as_u64().unwrap_or(0);
            let models_count = stats["models"].as_u64().unwrap_or(0);
            
            let green = Style::new().green();
            let bold = Style::new().bold();
            
            println!("{} {}", green.paint("✓"), bold.apply_to("Validation successful"));
            println!("  {} providers", providers_count);
            println!("  {} models", models_count);
            println!("  No errors found");
        }
        Err(e) => {
            pb.finish_with_message("❌ Database validation failed");
            println!();
            
            let red = Style::new().red();
            let bold = Style::new().bold();
            
            println!("{} {}", red.paint("✗"), bold.apply_to("Validation failed"));
            
            if detailed {
                println!();
                println!("Error details:");
                println!("{}", e);
            } else {
                println!("  Use --detailed for full error information");
            }
            
            return Err(anyhow::anyhow!("Validation failed: {}", e));
        }
    }
    
    Ok(())
}

async fn show_statistics(providers_dir: &PathBuf, format: &str) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    pb.set_message("Loading database...");
    
    let mut engine = Engine::new();
    engine.load_from_directory(providers_dir).await?;
    pb.finish_with_message("Database loaded");
    
    let stats = engine.get_statistics();
    
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        "table" => {
            print_table_stats(&stats);
        }
        _ => {
            warn!("Unknown format: {}, using table", format);
            print_table_stats(&stats);
        }
    }
    
    Ok(())
}

fn print_table_stats(stats: &serde_json::Value) {
    let bold = Style::new().bold();
    let cyan = Style::new().cyan();
    let green = Style::new().green();
    
    println!();
    println!("{}", bold.apply_to("📊 Database Statistics"));
    println!();
    
    println!("{}: {}", 
             cyan.paint("Providers"), 
             stats["providers"].as_u64().unwrap_or(0));
    println!("{}: {}", 
             cyan.paint("Models"), 
             stats["models"].as_u64().unwrap_or(0));
    println!();
    
    if let Some(families) = stats["families"].as_object() {
        println!("{}", bold.apply_to("🏷️  Model Families"));
        let mut families_vec: Vec<_> = families.iter().collect();
        families_vec.sort_by(|a, b| b.1.as_u64().unwrap_or(0).cmp(&a.1.as_u64().unwrap_or(0)));
        
        for (family, count) in families_vec.iter().take(10) {
            println!("  {}: {}", family, count.as_u64().unwrap_or(0));
        }
        println!();
    }
    
    if let Some(capabilities) = stats["capabilities"].as_object() {
        println!("{}", bold.apply_to("⚡ Capabilities"));
        for (capability, count) in capabilities {
            let emoji = match capability.as_str() {
                "reasoning" => "🧠",
                "tool_call" => "🔧",
                "attachment" => "📎",
                "structured_output" => "📋",
                _ => "•",
            };
            println!("  {} {}: {}", emoji, capability, count.as_u64().unwrap_or(0));
        }
        println!();
    }
    
    if let Some(modalities) = stats["modalities"].as_object() {
        println!("{}", bold.apply_to("🎨 Modalities"));
        for (modality, count) in modalities {
            let emoji = match modality.as_str() {
                "text" => "📝",
                "image" => "🖼️",
                "audio" => "🎵",
                "video" => "🎬",
                "pdf" => "📄",
                _ => "•",
            };
            println!("  {} {}: {}", emoji, modality, count.as_u64().unwrap_or(0));
        }
    }
}

async fn search_models(
    query: &str,
    providers_dir: &PathBuf,
    family: Option<String>,
    capability: Option<String>,
    modality: Option<String>,
    limit: usize,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.yellow} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    pb.set_message("Loading database...");
    
    let mut engine = Engine::new();
    engine.load_from_directory(providers_dir).await?;
    pb.finish_with_message("Database loaded");
    
    let models = if let Some(family) = family {
        engine.get_models_by_family(&family)
    } else if let Some(capability) = capability {
        engine.get_models_by_capability(&capability)
    } else if let Some(modality) = modality {
        engine.get_models_by_modality(&modality)
    } else {
        engine.search_models(query)
    };
    
    let models: Vec<_> = models.into_iter().take(limit).collect();
    
    let bold = Style::new().bold();
    let cyan = Style::new().cyan();
    let green = Style::new().green();
    
    println!();
    println!("{} {} {}", 
             bold.apply_to("🔍"),
             green.paint(format!("{} models found", models.len())),
             cyan.paint(format!("showing first {}", models.len().min(limit))));
    println!();
    
    for model in models {
        println!("{} {}", green.paint("•"), bold.apply_to(&model.name));
        println!("  {} {}", cyan.paint("ID:"), model.id);
        if let Some(ref family) = model.family {
            println!("  {} {}", cyan.paint("Family:"), family);
        }
        println!("  {} {}", cyan.paint("Provider:"), model.id.split('/').next().unwrap_or("unknown"));
        println!("  {} {}", cyan.paint("Capabilities:"), format_capabilities(model));
        println!();
    }
    
    Ok(())
}

fn format_capabilities(model: &neopilot_models_core::Model) -> String {
    let mut caps = Vec::new();
    
    if model.reasoning { caps.push("reasoning"); }
    if model.tool_call { caps.push("tool_call"); }
    if model.attachment { caps.push("attachment"); }
    if model.structured_output.unwrap_or(false) { caps.push("structured_output"); }
    
    caps.join(", ")
}

async fn export_database(providers_dir: &PathBuf, output: Option<PathBuf>, pretty: bool) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    pb.set_message("Loading database for export...");
    
    let mut engine = Engine::new();
    engine.load_from_directory(providers_dir).await?;
    pb.finish_with_message("Database loaded");
    
    let json_data = engine.get_providers_json()?;
    
    let output_string = if pretty {
        serde_json::to_string_pretty(&json_data)?
    } else {
        serde_json::to_string(&json_data)?
    };
    
    match output {
        Some(path) => {
            tokio::fs::write(&path, output_string).await?;
            println!("✅ Exported database to: {:?}", path);
        }
        None => {
            println!("{}", output_string);
        }
    }
    
    Ok(())
}

async fn generate_schema(output: Option<PathBuf>) -> Result<()> {
    let engine = Engine::new();
    let schema = engine.get_model_schema()?;
    
    let schema_json = serde_json::to_string_pretty(&schema)?;
    
    match output {
        Some(path) => {
            tokio::fs::write(&path, schema_json).await?;
            println!("✅ Generated schema: {:?}", path);
        }
        None => {
            println!("{}", schema_json);
        }
    }
    
    Ok(())
}
