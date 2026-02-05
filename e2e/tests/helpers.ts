/**
 * E2E Test Helpers
 */

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:8080';

export interface Node {
  id: string;
  content: string;
  node_type: string;
  embedding?: number[] | null;
  meta: {
    created_at: string;
    updated_at: string;
    accessed_at: string | null;
    access_score: number;
    confidence: number;
    source: string;
    document_id: string | null;
    chunk_index: number | null;
  };
  attributes: Record<string, any>;
}

export interface Edge {
  source: string;
  target: string;
  relation: string;
  weight: number;
  vector: number[] | null;
  created_at: string;
  expired: boolean;
  replaced_by: string | null;
  attributes: Record<string, any>;
}

export interface CreateNodeRequest {
  content: string;
  node_type: string;
}

export interface CreateEdgeRequest {
  source: string;
  target: string;
  relation: string;
  weight: number;
}

export interface QueryRequest {
  query: string;
  limit?: number;
}

export interface QueryResponse {
  nodes: Node[];
  total_count: number;
  execution_time_ms: number;
  truncated: boolean;
}

export interface StatsResponse {
  node_count: number;
  edge_count: number;
  embedded_count: number;
}

/**
 * API Client for SYNTON-DB
 */
export class SyntonApiClient {
  constructor(private baseUrl: string = BASE_URL) {}

  async health(): Promise<{ status: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/health`);
    if (!response.ok) throw new Error('Health check failed');
    return response.json();
  }

  async createNode(content: string, nodeType: string = 'Concept'): Promise<Node> {
    // API expects lowercase node types, with special handling for RawChunk -> raw_chunk
    let nodeTypeLower = nodeType.toLowerCase();
    if (nodeTypeLower === 'rawchunk') {
      nodeTypeLower = 'raw_chunk';
    }
    const response = await fetch(`${this.baseUrl}/nodes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content, node_type: nodeTypeLower }),
    });
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to create node: ${response.status} ${errorText}`);
    }
    const data = await response.json();
    return data.node;
  }

  async getNode(id: string): Promise<Node | null> {
    const response = await fetch(`${this.baseUrl}/nodes/${id}`);
    if (response.status === 404) return null;
    if (!response.ok) throw new Error('Failed to get node');
    const data = await response.json();
    return data.node;
  }

  async deleteNode(id: string): Promise<boolean> {
    const response = await fetch(`${this.baseUrl}/nodes/${id}`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id }),
    });
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to delete node: ${response.status} ${errorText}`);
    }
    const data = await response.json();
    return data.deleted;
  }

  async listNodes(): Promise<Node[]> {
    const response = await fetch(`${this.baseUrl}/nodes`);
    if (!response.ok) throw new Error('Failed to list nodes');
    return response.json();
  }

  async createEdge(source: string, target: string, relation: string, weight: number = 1.0): Promise<Edge> {
    // Convert hyphenated relations to underscore format (e.g., "is-part-of" -> "is_part_of")
    const relationFormatted = relation.replace(/-/g, '_');
    const response = await fetch(`${this.baseUrl}/edges`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source, target, relation: relationFormatted, weight }),
    });
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to create edge: ${response.status} ${errorText}`);
    }
    const data = await response.json();
    return data.edge;
  }

  async query(query: string, limit?: number): Promise<QueryResponse> {
    const response = await fetch(`${this.baseUrl}/query`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, limit, include_metadata: false }),
    });
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to execute query: ${response.status} ${errorText}`);
    }
    return response.json();
  }

  async stats(): Promise<StatsResponse> {
    const response = await fetch(`${this.baseUrl}/stats`);
    if (!response.ok) throw new Error('Failed to get stats');
    return response.json();
  }

  /**
   * Clear all test data
   */
  async clearAll(): Promise<void> {
    const nodes = await this.listNodes();
    for (const node of nodes) {
      await this.deleteNode(node.id);
    }
  }
}

/**
 * Generate a random test name
 */
export function randomTestName(prefix: string = 'test'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substring(7)}`;
}

/**
 * Wait for a condition to be true
 */
export async function waitFor(
  condition: () => boolean | Promise<boolean>,
  timeout: number = 5000,
  interval: number = 100
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (await condition()) return;
    await new Promise(resolve => setTimeout(resolve, interval));
  }
  throw new Error(`Condition not met within ${timeout}ms`);
}
