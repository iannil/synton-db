// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Server management for SYNTON-DB.
//!
//! Handles the lifecycle of both gRPC and REST API servers.

use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

use crate::config::Config;
use synton_api::SyntonDbService;

#[cfg(feature = "ml")]
use synton_ml::{BackendType, EmbeddingConfig, EmbeddingService};

/// Server handle for managing running servers.
pub struct ServerHandle {
    /// The gRPC server task handle.
    grpc_handle: Option<tokio::task::JoinHandle<()>>,

    /// The REST server task handle.
    rest_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ServerHandle {
    /// Create a new server handle.
    pub fn new(
        grpc_handle: Option<tokio::task::JoinHandle<()>>,
        rest_handle: Option<tokio::task::JoinHandle<()>>,
    ) -> Self {
        Self {
            grpc_handle,
            rest_handle,
        }
    }

    /// Signal all servers to shut down.
    pub async fn shutdown(mut self) {
        // Wait for servers to finish
        if let Some(handle) = self.grpc_handle.take() {
            handle.abort();
        }

        if let Some(handle) = self.rest_handle.take() {
            handle.abort();
        }

        info!("All servers have shut down");
    }
}

/// Start both gRPC and REST API servers.
///
/// # Errors
///
/// Returns an error if servers fail to start.
pub fn start_servers(
    config: &Config,
) -> Result<(ServerHandle, oneshot::Sender<()>), Box<dyn std::error::Error>> {
    // Initialize service with optional ML support
    #[cfg(feature = "ml")]
    let service = {
        if config.ml.enabled {
            match init_embedding_service(config) {
                Ok(embedding) => {
                    info!(
                        "ML embedding service initialized: backend={}, dimension={}",
                        embedding.backend_type(),
                        embedding.dimension()
                    );
                    Arc::new(SyntonDbService::with_embedding(embedding))
                }
                Err(e) => {
                    warn!("Failed to initialize ML service: {}. Running without embeddings.", e);
                    Arc::new(SyntonDbService::new())
                }
            }
        } else {
            info!("ML features disabled. Running without embeddings.");
            Arc::new(SyntonDbService::new())
        }
    };

    #[cfg(not(feature = "ml"))]
    let service = {
        if config.ml.enabled {
            info!("ML features requested but ML feature is not enabled. Recompile with --features ml to enable.");
        }
        Arc::new(SyntonDbService::new())
    };

    let grpc_handle = maybe_start_grpc(config, service.clone())?;
    let rest_handle = maybe_start_rest(config, service)?;
    let (shutdown_tx, _shutdown_rx) = oneshot::channel();

    let handle = ServerHandle::new(grpc_handle, rest_handle);

    Ok((handle, shutdown_tx))
}

/// Initialize the embedding service from configuration.
#[cfg(feature = "ml")]
fn init_embedding_service(config: &Config) -> Result<Arc<EmbeddingService>, Box<dyn std::error::Error>> {
    use synton_ml::{ApiConfig, LocalModelConfig};

    let backend_type = match config.ml.backend.to_lowercase().as_str() {
        "openai" => BackendType::OpenAi,
        "ollama" => BackendType::Ollama,
        _ => BackendType::Local,
    };

    let ml_config = EmbeddingConfig {
        backend: backend_type,
        local: LocalModelConfig {
            model_name: config.ml.local_model.clone(),
            ..Default::default()
        },
        api: ApiConfig {
            endpoint: config.ml.api_endpoint.clone(),
            api_key: config.ml.api_key.clone(),
            model: config.ml.api_model.clone(),
            timeout_secs: config.ml.timeout_secs,
            ..Default::default()
        },
        cache_enabled: config.ml.cache_enabled,
        cache_size: config.ml.cache_size,
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new()?;
    let service = rt.block_on(EmbeddingService::from_config(ml_config))?;
    Ok(Arc::new(service))
}

/// Start the gRPC server if enabled.
fn maybe_start_grpc(
    config: &Config,
    service: Arc<SyntonDbService>,
) -> Result<Option<tokio::task::JoinHandle<()>>, Box<dyn std::error::Error>> {
    if !config.server.grpc_enabled {
        return Ok(None);
    }

    let grpc_addr = format!("{}:{}", config.server.host, config.server.grpc_port);
    let grpc_addr: std::net::SocketAddr = grpc_addr.parse()?;

    let handle = tokio::spawn(async move {
        info!("Starting gRPC server on {}", grpc_addr);

        // Create the gRPC router
        let grpc_router = synton_api::create_grpc_router(service);

        let listener = tokio::net::TcpListener::bind(grpc_addr).await;

        let listener = match listener {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind gRPC server: {}", e);
                return;
            }
        };

        let local_addr = listener.local_addr();
        match local_addr {
            Ok(addr) => info!("gRPC server listening on {}", addr),
            Err(e) => error!("Failed to get gRPC server address: {}", e),
        }

        // Serve
        let result = tonic::transport::Server::builder()
            .add_service(grpc_router)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await;

        if let Err(e) = result {
            error!("gRPC server error: {}", e);
        }

        info!("gRPC server shut down");
    });

    Ok(Some(handle))
}

/// Start the REST API server if enabled.
fn maybe_start_rest(
    config: &Config,
    service: Arc<SyntonDbService>,
) -> Result<Option<tokio::task::JoinHandle<()>>, Box<dyn std::error::Error>> {
    if !config.server.rest_enabled {
        return Ok(None);
    }

    let rest_addr = format!("{}:{}", config.server.host, config.server.rest_port);

    let handle = tokio::spawn(async move {
        info!("Starting REST API server on {}", rest_addr);

        let state = synton_api::AppState::new(service);

        let app = axum::Router::new()
            .route("/health", axum::routing::get(synton_api::rest::health_check))
            .route("/stats", axum::routing::get(synton_api::rest::stats))
            .route("/nodes", axum::routing::post(synton_api::rest::add_node))
            .route("/nodes", axum::routing::get(synton_api::rest::get_all_nodes))
            .route("/nodes/:id", axum::routing::get(synton_api::rest::get_node))
            .route("/nodes/:id", axum::routing::delete(synton_api::rest::delete_node))
            .route("/edges", axum::routing::post(synton_api::rest::add_edge))
            .route("/query", axum::routing::post(synton_api::rest::query))
            .route("/traverse", axum::routing::post(synton_api::rest::traverse))
            .route("/bulk", axum::routing::post(synton_api::rest::bulk_operation))
            .with_state(state)
            .layer(
                tower_http::cors::CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods(tower_http::cors::Any)
                    .allow_headers(tower_http::cors::Any),
            )
            .layer(tower_http::trace::TraceLayer::new_for_http());

        let listener = tokio::net::TcpListener::bind(&rest_addr).await;

        let listener = match listener {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind REST server: {}", e);
                return;
            }
        };

        let local_addr = listener.local_addr();
        match local_addr {
            Ok(addr) => info!("REST API server listening on {}", addr),
            Err(e) => error!("Failed to get REST server address: {}", e),
        }

        // Serve
        let result = axum::serve(listener, app).await;

        if let Err(e) = result {
            error!("REST server error: {}", e);
        }

        info!("REST API server shut down");
    });

    Ok(Some(handle))
}
