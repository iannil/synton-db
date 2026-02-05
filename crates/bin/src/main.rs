// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! SYNTON-DB Server Binary
//!
//! The main entry point for the SYNTON-DB cognitive database server.
//! Runs both gRPC and REST API servers with configurable storage and memory management.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod config;
mod server;

use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use clap::Parser;

use crate::config::Config;

/// SYNTON-DB - A cognitive database for LLMs
#[derive(Parser, Debug)]
#[command(name = "synton-db-server")]
#[command(author = "SYNTON-DB Team")]
#[command(version)]
#[command(about = "A cognitive database for Large Language Models", long_about = None)]
struct Args {
    /// Path to the configuration file (TOML format)
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Override the server host
    #[arg(long)]
    host: Option<String>,

    /// Override the gRPC port
    #[arg(long)]
    grpc_port: Option<u16>,

    /// Override the REST port
    #[arg(long)]
    rest_port: Option<u16>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long)]
    log_level: Option<String>,

    /// Enable JSON formatted logs
    #[arg(long)]
    json_logs: bool,

    /// Validate configuration and exit
    #[arg(long)]
    validate: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration
    let mut config = load_or_create_config(&args.config)?;

    // Apply command-line overrides
    apply_cli_overrides(&mut config, &args);

    // Validate configuration
    config.validate()?;

    if args.validate {
        println!("Configuration is valid");
        return Ok(());
    }

    // Apply JSON logs override from CLI
    if args.json_logs {
        config.logging.json_format = true;
    }

    // Initialize logging
    init_logging(&config);

    info!("Starting SYNTON-DB server v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from: {}", args.config);

    // Log key configuration values
    log_config(&config);

    // Start the server
    let start_time = Instant::now();

    // Ensure data directories exist
    ensure_data_dirs(&config)?;

    // Start servers
    let (server_handle, _shutdown_tx) = server::start_servers(&config)?;

    info!("SYNTON-DB server started in {:.2}s", start_time.elapsed().as_secs_f64());
    info!("Press Ctrl+C to shut down");

    // Wait for shutdown signal
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received");
        }
        Err(e) => {
            eprintln!("Error waiting for shutdown signal: {}", e);
        }
    }

    // Graceful shutdown
    info!("Shutting down...");
    server_handle.shutdown().await;

    info!("SYNTON-DB server stopped");
    Ok(())
}

/// Load configuration from file, or create default if file doesn't exist.
fn load_or_create_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    match std::fs::metadata(path) {
        Ok(_) => Config::from_file(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "Config file '{}' not found, using default configuration",
                path
            );
            eprintln!("To create a config file, run with --config flag and redirect output:");
            eprintln!(
                "  synton-db-server --config {} 2>/dev/null | head -n 100 > {}.example",
                path, path
            );
            Ok(Config::default())
        }
        Err(e) => Err(e.into()),
    }
}

/// Apply command-line overrides to configuration.
fn apply_cli_overrides(config: &mut Config, args: &Args) {
    if let Some(host) = &args.host {
        config.server.host = host.clone();
    }

    if let Some(port) = args.grpc_port {
        config.server.grpc_port = port;
    }

    if let Some(port) = args.rest_port {
        config.server.rest_port = port;
    }

    if let Some(level) = &args.log_level {
        config.logging.level = level.clone();
    }
}

/// Initialize logging based on configuration.
fn init_logging(config: &Config) {
    let level = match config.logging.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    if config.logging.json_format {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }
}

/// Log key configuration values.
fn log_config(config: &Config) {
    info!("Server configuration:");
    info!(
        "  gRPC port: {} ({})",
        config.server.grpc_port,
        if config.server.grpc_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    info!(
        "  REST port: {} ({})",
        config.server.rest_port,
        if config.server.rest_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    info!("  Log level: {}", config.logging.level);
    info!("Storage configuration:");
    info!("  RocksDB path: {}", config.storage.rocksdb_path.display());
    info!("  Lance path: {}", config.storage.lance_path.display());
    info!("  Max open files: {}", config.storage.max_open_files);
    info!("  Cache size: {} MB", config.storage.cache_size_mb);
    info!("Memory configuration:");
    info!("  Decay scale: {}", config.memory.decay_scale);
    info!(
        "  Initial access score: {}",
        config.memory.initial_access_score
    );
    info!("  Access boost: {}", config.memory.access_boost);
    info!(
        "  Retention threshold: {}",
        config.memory.retention_threshold
    );
}

/// Ensure data directories exist.
fn ensure_data_dirs(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&config.storage.rocksdb_path)?;
    std::fs::create_dir_all(&config.storage.lance_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_application() {
        let mut config = Config::default();
        let args = Args {
            config: "test.toml".to_string(),
            host: Some("127.0.0.1".to_string()),
            grpc_port: Some(9090),
            rest_port: Some(8081),
            log_level: Some("debug".to_string()),
            json_logs: false,
            validate: false,
        };

        apply_cli_overrides(&mut config, &args);

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.grpc_port, 9090);
        assert_eq!(config.server.rest_port, 8081);
        assert_eq!(config.logging.level, "debug");
    }
}
