/**
 * API types for SYNTON-DB
 *
 * These types match the backend Rust API models.
 */

/**
 * Node types in the Tensor-Graph.
 */
export type NodeType = 'entity' | 'concept' | 'fact' | 'raw_chunk';

/**
 * Relation types between nodes.
 */
export type Relation =
  | 'is_part_of'
  | 'causes'
  | 'contradicts'
  | 'happened_after'
  | 'similar_to'
  | 'is_a'
  | 'located_at'
  | 'belongs_to'
  | string;

/**
 * Data source for a node.
 */
export type Source = 'manual' | 'api_import' | 'document_ingestion' | 'system_generated';

/**
 * Node metadata.
 */
export interface NodeMeta {
  created_at: string;
  updated_at: string;
  accessed_at: string | null;
  access_score: number;
  confidence: number;
  source: Source;
  document_id: string | null;
  chunk_index: number | null;
}

/**
 * A node in the Tensor-Graph.
 */
export interface Node {
  id: string;
  content: string;
  embedding: number[] | null;
  meta: NodeMeta;
  node_type: NodeType;
  attributes: Record<string, unknown>;
}

/**
 * An edge in the Tensor-Graph.
 */
export interface Edge {
  source: string;
  target: string;
  relation: Relation;
  weight: number;
  vector: number[] | null;
  created_at: string;
  expired: boolean;
  replaced_by: string | null;
  attributes: Record<string, unknown>;
}

/**
 * Memory statistics.
 */
export interface MemoryStats {
  total_nodes: number;
  active_nodes: number;
  decayed_nodes: number;
  average_score: number;
  load_factor: number;
}

/**
 * Database statistics.
 */
export interface DatabaseStats {
  node_count: number;
  edge_count: number;
  embedded_count: number;
  memory_stats: MemoryStats;
}

/**
 * Health check response.
 */
export interface HealthResponse {
  status: string;
  version: string;
  uptime_secs: number;
}

/**
 * Request to add a node.
 */
export interface AddNodeRequest {
  content: string;
  node_type: NodeType;
  embedding?: number[];
  attributes?: Record<string, unknown>;
}

/**
 * Response from adding a node.
 */
export interface AddNodeResponse {
  node: Node;
  created: boolean;
}

/**
 * Request to get a node by ID.
 */
export interface GetNodeRequest {
  id: string;
}

/**
 * Response with a node.
 */
export interface GetNodeResponse {
  node: Node | null;
}

/**
 * Request to delete a node.
 */
export interface DeleteNodeRequest {
  id: string;
}

/**
 * Response from deleting a node.
 */
export interface DeleteNodeResponse {
  deleted: boolean;
  id: string;
}

/**
 * Request to add an edge.
 */
export interface AddEdgeRequest {
  source: string;
  target: string;
  relation: Relation;
  weight?: number;
  vector?: number[];
}

/**
 * Response from adding an edge.
 */
export interface AddEdgeResponse {
  edge: Edge;
}

/**
 * Request to query the database.
 */
export interface QueryRequest {
  query: string;
  limit?: number;
  include_metadata?: boolean;
}

/**
 * Response from a database query.
 */
export interface QueryResponse {
  nodes: Node[];
  total_count: number;
  execution_time_ms: number;
  truncated: boolean;
}

/**
 * Request for hybrid search (GraphRAG).
 */
export interface HybridSearchRequest {
  query: string;
  k?: number;
}

/**
 * Response from hybrid search.
 */
export interface HybridSearchResponse {
  nodes: Node[];
  count: number;
}

/**
 * Direction for graph traversal.
 */
export type TraverseDirection = 'forward' | 'backward' | 'both';

/**
 * Request for graph traversal.
 */
export interface TraverseRequest {
  start_id: string;
  max_depth: number;
  max_nodes: number;
  direction: TraverseDirection;
}

/**
 * Response from graph traversal.
 */
export interface TraverseResponse {
  nodes: Node[];
  edges: Edge[];
  depth: number;
  truncated: boolean;
}

/**
 * Bulk operation request.
 */
export interface BulkOperationRequest {
  nodes: AddNodeRequest[];
  edges: AddEdgeRequest[];
}

/**
 * Bulk operation response.
 */
export interface BulkOperationResponse {
  node_ids: string[];
  edge_ids: string[];
  success_count: number;
  failure_count: number;
  errors: string[];
}

/**
 * API error response.
 */
export interface ApiError {
  error: string;
  message?: string;
}

/**
 * Node info for display (simplified).
 */
export interface NodeInfo {
  id: string;
  content: string;
  node_type: NodeType;
  created_at: string;
}

/**
 * Edge info for display (simplified).
 */
export interface EdgeInfo {
  id: string;
  source: string;
  target: string;
  relation: Relation;
  weight: number;
}

/**
 * Graph data for visualization.
 */
export interface GraphData {
  nodes: Array<{
    id: string;
    label: string;
    type: NodeType;
    data: Node;
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
    label: Relation;
    weight: number;
    data: Edge;
  }>;
}

/**
 * Paginated response.
 */
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  has_more: boolean;
}

/**
 * Filter options for nodes.
 */
export interface NodeFilterOptions {
  type?: NodeType;
  search?: string;
  has_embedding?: boolean;
  limit?: number;
  offset?: number;
}

/**
 * Filter options for edges.
 */
export interface EdgeFilterOptions {
  relation?: Relation;
  source?: string;
  target?: string;
  min_weight?: number;
  limit?: number;
  offset?: number;
}
