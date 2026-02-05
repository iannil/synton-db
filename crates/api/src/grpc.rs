// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{
        AddEdgeRequest as ApiAddEdgeRequest, AddNodeRequest as ApiAddNodeRequest,
        DeleteNodeRequest, GetNodeRequest, QueryRequest as ApiQueryRequest,
        TraverseRequest as ApiTraverseRequest,
    },
    SyntonDbService,
};
use synton_core::{Edge as CoreEdge, Node as CoreNode, NodeType as CoreNodeType, Relation as CoreRelation};
use synton_graph::{Graph, TraverseDirection as GraphTraverseDirection};

// Include the generated proto code
pub mod synton {
    tonic::include_proto!("synton.v1");
}

use synton::synton_db_server::{SyntonDb as SyntonDbTrait, SyntonDbServer};

/// gRPC service implementation.
pub struct GrpcService {
    inner: Arc<SyntonDbService>,
}

impl GrpcService {
    /// Create a new gRPC service.
    pub fn new(service: Arc<SyntonDbService>) -> Self {
        Self { inner: service }
    }
}

#[tonic::async_trait]
impl SyntonDbTrait for GrpcService {
    async fn health(
        &self,
        _request: tonic::Request<synton::HealthRequest>,
    ) -> Result<tonic::Response<synton::HealthResponse>, tonic::Status> {
        let health = self.inner.health();
        Ok(tonic::Response::new(synton::HealthResponse {
            status: health.status,
            version: health.version,
            uptime_secs: health.uptime_secs,
        }))
    }

    async fn add_node(
        &self,
        request: tonic::Request<synton::AddNodeRequest>,
    ) -> Result<tonic::Response<synton::AddNodeResponse>, tonic::Status> {
        let req = request.into_inner();
        let node_type = match req.node_type {
            1 => CoreNodeType::Entity,   // NODE_TYPE_ENTITY
            2 => CoreNodeType::Concept,  // NODE_TYPE_CONCEPT
            3 => CoreNodeType::Fact,     // NODE_TYPE_FACT
            4 => CoreNodeType::RawChunk, // NODE_TYPE_RAW_CHUNK
            _ => CoreNodeType::Concept,
        };

        let api_request = ApiAddNodeRequest {
            content: req.content,
            node_type,
            embedding: if req.embedding.is_empty() {
                None
            } else {
                Some(req.embedding)
            },
            attributes: if req.attributes.is_empty() {
                None
            } else {
                Some(serde_json::to_value(req.attributes).unwrap_or_default())
            },
        };

        match self.inner.add_node(api_request).await {
            Ok(response) => {
                let proto_node = core_node_to_proto(response.node);
                Ok(tonic::Response::new(synton::AddNodeResponse {
                    node: Some(proto_node),
                    created: response.created,
                }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn get_node(
        &self,
        request: tonic::Request<synton::GetNodeRequest>,
    ) -> Result<tonic::Response<synton::GetNodeResponse>, tonic::Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let api_request = GetNodeRequest { id };
        match self.inner.get_node(api_request).await {
            Ok(response) => {
                let proto_node = response.node.map(core_node_to_proto);
                Ok(tonic::Response::new(synton::GetNodeResponse { node: proto_node }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn delete_node(
        &self,
        request: tonic::Request<synton::DeleteNodeRequest>,
    ) -> Result<tonic::Response<synton::DeleteNodeResponse>, tonic::Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let api_request = DeleteNodeRequest { id };
        match self.inner.delete_node(api_request).await {
            Ok(response) => Ok(tonic::Response::new(synton::DeleteNodeResponse {
                deleted: response.deleted,
                id: response.id.to_string(),
            })),
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn list_nodes(
        &self,
        request: tonic::Request<synton::ListNodesRequest>,
    ) -> Result<tonic::Response<synton::ListNodesResponse>, tonic::Status> {
        let req = request.into_inner();
        let mut nodes = self.inner.all_nodes().await;

        let total_count = nodes.len();
        let offset = req.offset as usize;
        let limit = req.limit as usize;

        if offset < nodes.len() {
            nodes = nodes.into_iter().skip(offset).take(limit).collect();
        } else {
            nodes.clear();
        }

        let proto_nodes: Vec<synton::Node> = nodes.into_iter().map(core_node_to_proto).collect();

        Ok(tonic::Response::new(synton::ListNodesResponse {
            nodes: proto_nodes,
            total_count: total_count as u32,
        }))
    }

    async fn add_edge(
        &self,
        request: tonic::Request<synton::AddEdgeRequest>,
    ) -> Result<tonic::Response<synton::AddEdgeResponse>, tonic::Status> {
        let req = request.into_inner();
        let relation = match req.relation {
            1 => CoreRelation::IsA,           // IS_A
            2 => CoreRelation::IsPartOf,       // PART_OF
            3 => CoreRelation::Causes,         // CAUSES
            4 => CoreRelation::SimilarTo,      // SIMILAR_TO
            5 => CoreRelation::Contradicts,    // CONTRADICTS
            6 => CoreRelation::HappenedAfter,  // HAPPENED_AFTER
            7 => CoreRelation::BelongsTo,     // BELONGS_TO
            _ => CoreRelation::SimilarTo,
        };

        let source = parse_uuid(&req.source)?;
        let target = parse_uuid(&req.target)?;

        let api_request = ApiAddEdgeRequest {
            source,
            target,
            relation,
            weight: req.weight,
            vector: if req.vector.is_empty() {
                None
            } else {
                Some(req.vector)
            },
        };

        match self.inner.add_edge(api_request).await {
            Ok(response) => {
                let proto_edge = core_edge_to_proto(response.edge);
                Ok(tonic::Response::new(synton::AddEdgeResponse { edge: Some(proto_edge) }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn get_edges(
        &self,
        request: tonic::Request<synton::GetEdgesRequest>,
    ) -> Result<tonic::Response<synton::GetEdgesResponse>, tonic::Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.node_id)?;

        // Get edges from the graph
        let graph = self.inner.graph().await;
        let graph_guard = graph.read().await;
        let edges = match Graph::edges(&*graph_guard, id, GraphTraverseDirection::Forward).await {
            Ok(edges) => edges,
            Err(_) => return Ok(tonic::Response::new(synton::GetEdgesResponse { edges: vec![] })),
        };

        let proto_edges: Vec<synton::Edge> = edges.into_iter().map(core_edge_to_proto).collect();
        Ok(tonic::Response::new(synton::GetEdgesResponse { edges: proto_edges }))
    }

    async fn query(
        &self,
        request: tonic::Request<synton::QueryRequest>,
    ) -> Result<tonic::Response<synton::QueryResponse>, tonic::Status> {
        let req = request.into_inner();

        let api_request = ApiQueryRequest {
            query: req.query,
            limit: if req.limit == 0 { None } else { Some(req.limit as usize) },
            include_metadata: req.include_metadata,
        };

        match self.inner.query(api_request).await {
            Ok(response) => {
                let nodes: Vec<synton::Node> = response.nodes.into_iter().map(core_node_to_proto).collect();
                Ok(tonic::Response::new(synton::QueryResponse {
                    nodes,
                    total_count: response.total_count as u32,
                    execution_time_ms: response.execution_time_ms,
                    truncated: response.truncated,
                }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn traverse(
        &self,
        request: tonic::Request<synton::TraverseRequest>,
    ) -> Result<tonic::Response<synton::TraverseResponse>, tonic::Status> {
        let req = request.into_inner();
        let start_id = parse_uuid(&req.start_id)?;

        let direction = match req.direction {
            1 => crate::models::TraverseDirection::Forward,  // FORWARD
            2 => crate::models::TraverseDirection::Backward, // BACKWARD
            3 => crate::models::TraverseDirection::Both,     // BOTH
            _ => crate::models::TraverseDirection::Forward,
        };

        let api_request = ApiTraverseRequest {
            start_id,
            max_depth: req.max_depth as usize,
            max_nodes: req.max_nodes as usize,
            direction,
        };

        match self.inner.traverse(api_request).await {
            Ok(response) => {
                let nodes: Vec<synton::Node> = response.nodes.into_iter().map(core_node_to_proto).collect();
                let edges: Vec<synton::Edge> = response.edges.into_iter().map(core_edge_to_proto).collect();
                Ok(tonic::Response::new(synton::TraverseResponse {
                    nodes,
                    edges,
                    depth: response.depth as u32,
                    truncated: response.truncated,
                }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn stats(
        &self,
        _request: tonic::Request<synton::StatsRequest>,
    ) -> Result<tonic::Response<synton::StatsResponse>, tonic::Status> {
        match self.inner.stats().await {
            Ok(stats) => Ok(tonic::Response::new(synton::StatsResponse {
                node_count: stats.node_count as u32,
                edge_count: stats.edge_count as u32,
                embedded_count: stats.embedded_count as u32,
                memory_stats: Some(synton::MemoryStats {
                    total_nodes: stats.memory_stats.total_nodes as u32,
                    active_nodes: stats.memory_stats.active_nodes as u32,
                    decayed_nodes: stats.memory_stats.decayed_nodes as u32,
                    average_score: stats.memory_stats.average_score,
                    load_factor: stats.memory_stats.load_factor,
                }),
            })),
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn bulk_operation(
        &self,
        request: tonic::Request<synton::BulkOperationRequest>,
    ) -> Result<tonic::Response<synton::BulkOperationResponse>, tonic::Status> {
        let req = request.into_inner();

        let mut node_requests = Vec::new();
        for node_req in req.nodes {
            let node_type = match node_req.node_type {
                1 => CoreNodeType::Entity,   // NODE_TYPE_ENTITY
                2 => CoreNodeType::Concept,  // NODE_TYPE_CONCEPT
                3 => CoreNodeType::Fact,     // NODE_TYPE_FACT
                4 => CoreNodeType::RawChunk, // NODE_TYPE_RAW_CHUNK
                _ => CoreNodeType::Concept,
            };
            node_requests.push(ApiAddNodeRequest {
                content: node_req.content,
                node_type,
                embedding: if node_req.embedding.is_empty() {
                    None
                } else {
                    Some(node_req.embedding)
                },
                attributes: if node_req.attributes.is_empty() {
                    None
                } else {
                    Some(serde_json::to_value(node_req.attributes).unwrap_or_default())
                },
            });
        }

        let mut edge_requests = Vec::new();
        for edge_req in req.edges {
            let relation = match edge_req.relation {
                1 => CoreRelation::IsA,           // IS_A
                2 => CoreRelation::IsPartOf,       // PART_OF
                3 => CoreRelation::Causes,         // CAUSES
                4 => CoreRelation::SimilarTo,      // SIMILAR_TO
                5 => CoreRelation::Contradicts,    // CONTRADICTS
                6 => CoreRelation::HappenedAfter,  // HAPPENED_AFTER
                7 => CoreRelation::BelongsTo,     // BELONGS_TO
                _ => CoreRelation::SimilarTo,
            };
            let source = match parse_uuid(&edge_req.source) {
                Ok(id) => id,
                Err(_) => continue,
            };
            let target = match parse_uuid(&edge_req.target) {
                Ok(id) => id,
                Err(_) => continue,
            };
            edge_requests.push(ApiAddEdgeRequest {
                source,
                target,
                relation,
                weight: edge_req.weight,
                vector: None,
            });
        }

        let bulk_request = crate::models::BulkOperationRequest {
            nodes: node_requests,
            edges: edge_requests,
        };

        // Process bulk operation
        let mut node_ids = Vec::new();
        let mut edge_ids = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut errors = Vec::new();

        for node_req in bulk_request.nodes {
            match self.inner.add_node(node_req).await {
                Ok(resp) => {
                    node_ids.push(resp.node.id.to_string());
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(format!("Node creation failed: {}", e));
                    failure_count += 1;
                }
            }
        }

        for edge_req in bulk_request.edges {
            match self.inner.add_edge(edge_req).await {
                Ok(resp) => {
                    edge_ids.push(resp.edge.id());
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(format!("Edge creation failed: {}", e));
                    failure_count += 1;
                }
            }
        }

        Ok(tonic::Response::new(synton::BulkOperationResponse {
            node_ids,
            edge_ids,
            success_count: success_count as u32,
            failure_count: failure_count as u32,
            errors,
        }))
    }
}

/// Parse UUID from string.
fn parse_uuid(s: &str) -> Result<Uuid, tonic::Status> {
    Uuid::parse_str(s).map_err(|_| tonic::Status::invalid_argument("Invalid UUID format"))
}

/// Convert core node to proto node.
fn core_node_to_proto(node: CoreNode) -> synton::Node {
    synton::Node {
        id: node.id.to_string(),
        content: node.content().to_string(),
        node_type: match node.node_type {
            CoreNodeType::Entity => synton::NodeType::Entity as i32,
            CoreNodeType::Concept => synton::NodeType::Concept as i32,
            CoreNodeType::Fact => synton::NodeType::Fact as i32,
            CoreNodeType::RawChunk => synton::NodeType::RawChunk as i32,
        },
        embedding: node.embedding.unwrap_or_default(),
        created_at: node.meta.created_at.timestamp(),
        updated_at: node.meta.updated_at.timestamp(),
        access_score: node.meta.access_score,
        source: format!("{:?}", node.meta.source),
        attributes: node
            .attributes
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Convert core edge to proto edge.
fn core_edge_to_proto(edge: CoreEdge) -> synton::Edge {
    synton::Edge {
        source: edge.source.to_string(),
        target: edge.target.to_string(),
        relation: match edge.relation {
            CoreRelation::IsA => synton::Relation::IsA as i32,
            CoreRelation::IsPartOf => synton::Relation::PartOf as i32,
            CoreRelation::Causes => synton::Relation::Causes as i32,
            CoreRelation::SimilarTo => synton::Relation::SimilarTo as i32,
            CoreRelation::Contradicts => synton::Relation::Contradicts as i32,
            CoreRelation::HappenedAfter => synton::Relation::HappenedAfter as i32,
            CoreRelation::BelongsTo => synton::Relation::BelongsTo as i32,
            CoreRelation::LocatedAt => synton::Relation::BelongsTo as i32, // Map to similar
            CoreRelation::Custom(_) => synton::Relation::SimilarTo as i32, // Map custom to similar
        },
        weight: edge.weight,
        vector: edge.vector.unwrap_or_default(),
        created_at: edge.created_at.timestamp(),
        expired: edge.expired,
        replaced_by: edge.replaced_by.map(|id| id.to_string()).unwrap_or_default(),
        attributes: edge
            .attributes
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Create the gRPC router.
pub fn create_grpc_router(service: Arc<SyntonDbService>) -> SyntonDbServer<GrpcService> {
    let grpc_service = GrpcService::new(service);
    SyntonDbServer::new(grpc_service)
}

/// Start the gRPC server.
pub async fn run_grpc_server(
    addr: impl tokio::net::ToSocketAddrs,
    service: Arc<SyntonDbService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_service = GrpcService::new(service);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("gRPC server listening on {}", listener.local_addr()?);

    tonic::transport::Server::builder()
        .add_service(SyntonDbServer::new(grpc_service))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid() {
        let uuid_str = "00000000-0000-0000-0000-000000000000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("invalid");
        assert!(result.is_err());
    }
}
