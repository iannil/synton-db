/**
 * API client for SYNTON-DB
 *
 * Handles all communication with the backend REST API.
 */

import type {
  AddEdgeRequest,
  AddEdgeResponse,
  AddNodeRequest,
  AddNodeResponse,
  DatabaseStats,
  DeleteNodeRequest,
  DeleteNodeResponse,
  Edge,
  GetNodeRequest,
  GetNodeResponse,
  HealthResponse,
  HybridSearchRequest,
  HybridSearchResponse,
  Node,
  QueryRequest,
  QueryResponse,
  TraverseRequest,
  TraverseResponse,
  BulkOperationRequest,
  BulkOperationResponse,
} from '@/types/api';

/**
 * Base URL for the API.
 * In development, this proxied through Vite.
 */
const API_BASE = import.meta.env.VITE_API_URL || '';

/**
 * Custom error class for API errors.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public response?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Helper to handle fetch responses.
 */
async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let message = `HTTP ${response.status}`;
    try {
      const error = (await response.json()) as { error?: string; message?: string };
      message = error.error || error.message || message;
    } catch {
      // Use default message
    }
    throw new ApiError(message, response.status);
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

/**
 * API client class.
 */
export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = baseUrl;
  }

  /**
   * Make a GET request.
   */
  private async get<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });
    return handleResponse<T>(response);
  }

  /**
   * Make a POST request.
   */
  private async post<T>(path: string, data?: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: data ? JSON.stringify(data) : undefined,
    });
    return handleResponse<T>(response);
  }

  /**
   * Make a DELETE request.
   */
  private async delete<T>(path: string, data?: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
      },
      body: data ? JSON.stringify(data) : undefined,
    });
    return handleResponse<T>(response);
  }

  /**
   * Health check.
   */
  async health(): Promise<HealthResponse> {
    return this.get<HealthResponse>('/health');
  }

  /**
   * Get database statistics.
   */
  async stats(): Promise<DatabaseStats> {
    return this.get<DatabaseStats>('/stats');
  }

  /**
   * Get all nodes.
   */
  async getAllNodes(): Promise<Node[]> {
    return this.get<Node[]>('/nodes');
  }

  /**
   * Get a node by ID.
   */
  async getNode(request: GetNodeRequest): Promise<GetNodeResponse> {
    return this.get<GetNodeResponse>(`/nodes/${request.id}`);
  }

  /**
   * Add a node.
   */
  async addNode(request: AddNodeRequest): Promise<AddNodeResponse> {
    return this.post<AddNodeResponse>('/nodes', request);
  }

  /**
   * Delete a node.
   */
  async deleteNode(request: DeleteNodeRequest): Promise<DeleteNodeResponse> {
    return this.delete<DeleteNodeResponse>(`/nodes/${request.id}`, request);
  }

  /**
   * Add an edge.
   */
  async addEdge(request: AddEdgeRequest): Promise<AddEdgeResponse> {
    return this.post<AddEdgeResponse>('/edges', request);
  }

  /**
   * Query the database.
   */
  async query(request: QueryRequest): Promise<QueryResponse> {
    return this.post<QueryResponse>('/query', request);
  }

  /**
   * Hybrid search (GraphRAG).
   */
  async hybridSearch(request: HybridSearchRequest): Promise<HybridSearchResponse> {
    return this.post<HybridSearchResponse>('/hybrid_search', request);
  }

  /**
   * Traverse the graph.
   */
  async traverse(request: TraverseRequest): Promise<TraverseResponse> {
    return this.post<TraverseResponse>('/traverse', request);
  }

  /**
   * Bulk operations.
   */
  async bulk(request: BulkOperationRequest): Promise<BulkOperationResponse> {
    return this.post<BulkOperationResponse>('/bulk', request);
  }
}

/**
 * Singleton API client instance.
 */
export const api = new ApiClient();

/**
 * React hook for API calls with loading and error states.
 */
export interface ApiState<T> {
  data: T | null;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

export function createApiHook<T>(
  fetcher: () => Promise<T>
): () => ApiState<T> {
  return () => {
    const [data, setData] = React.useState<T | null>(null);
    const [isLoading, setIsLoading] = React.useState(true);
    const [error, setError] = React.useState<string | null>(null);

    const refetch = React.useCallback(async () => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await fetcher();
        setData(result);
      } catch (err) {
        setError(err instanceof ApiError ? err.message : 'Unknown error');
      } finally {
        setIsLoading(false);
      }
    }, [fetcher]);

    React.useEffect(() => {
      refetch();
    }, [refetch]);

    return { data, isLoading, error, refetch };
  };
}

import React from 'react';
