/**
 * Query page for natural language (PaQL) and GraphRAG hybrid search.
 */

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, HybridSearchResponse, QueryResponse } from '@/types/api';
import { Button, Textarea, Select } from '@/components/ui';

const QUERY_HISTORY_KEY = 'syntondb_query_history';

const SEARCH_TYPES = [
  { value: 'hybrid', label: 'GraphRAG Hybrid Search' },
  { value: 'text', label: 'Text Query' },
];

export function Query(): JSX.Element {
  const navigate = useNavigate();

  const [query, setQuery] = useState('');
  const [searchType, setSearchType] = useState<'hybrid' | 'text'>('hybrid');
  const [limit, setLimit] = useState(10);
  const [results, setResults] = useState<Node[]>([]);
  const [executionTime, setExecutionTime] = useState<number | null>(null);
  const [totalCount, setTotalCount] = useState<number | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [queryHistory, setQueryHistory] = useState<string[]>([]);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      // Update history
      const newHistory = [query, ...queryHistory.filter((q) => q !== query)].slice(0, 10);
      setQueryHistory(newHistory);
      localStorage.setItem(QUERY_HISTORY_KEY, JSON.stringify(newHistory));

      if (searchType === 'hybrid') {
        const response = await api.hybridSearch({ query, k: limit });
        setResults(response.nodes);
        setTotalCount(response.count);
        setExecutionTime(null);
      } else {
        const response = await api.query({
          query,
          limit,
          include_metadata: true,
        });
        setResults(response.nodes);
        setTotalCount(response.total_count);
        setExecutionTime(response.execution_time_ms);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Query failed');
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      handleSearch();
    }
  };

  const loadHistory = () => {
    const saved = localStorage.getItem(QUERY_HISTORY_KEY);
    if (saved) {
      try {
        setQueryHistory(JSON.parse(saved));
      } catch {
        // Ignore
      }
    }
  };

  useState(() => {
    loadHistory();
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Query Database</h1>
        <p className="text-gray-400 mt-1">
          Search using natural language or GraphRAG hybrid search
        </p>
      </div>

      {/* Search Form */}
      <div className="card space-y-4">
        <div className="flex flex-wrap gap-4">
          <div className="flex-1 min-w-[200px]">
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Search Type
            </label>
            <Select
              options={SEARCH_TYPES}
              value={searchType}
              onChange={(e) => setSearchType(e.target.value as 'hybrid' | 'text')}
              fullWidth
            />
          </div>
          <div className="w-32">
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Results
            </label>
            <Select
              options={[
                { value: '5', label: '5' },
                { value: '10', label: '10' },
                { value: '20', label: '20' },
                { value: '50', label: '50' },
                { value: '100', label: '100' },
              ]}
              value={limit.toString()}
              onChange={(e) => setLimit(parseInt(e.target.value))}
              fullWidth
            />
          </div>
        </div>

        <div>
          <Textarea
            placeholder="Enter your query... (Cmd/Ctrl + Enter to search)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyPress}
            rows={3}
          />
          <p className="text-xs text-gray-500 mt-1">
            Example: "Find all nodes related to artificial intelligence"
          </p>
        </div>

        <div className="flex justify-between items-center">
          <Button onClick={handleSearch} isLoading={isLoading}>
            Search
          </Button>
          {queryHistory.length > 0 && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-500">Recent:</span>
              <div className="flex flex-wrap gap-2">
                {queryHistory.slice(0, 3).map((h, i) => (
                  <button
                    key={i}
                    onClick={() => setQuery(h)}
                    className="text-xs px-2 py-1 rounded bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
                  >
                    {h.length > 30 ? h.slice(0, 30) + '...' : h}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="p-4 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400">
          {error}
        </div>
      )}

      {/* Results Summary */}
      {results.length > 0 && (
        <div className="card">
          <div className="flex items-center justify-between">
            <p className="text-gray-300">
              Found <span className="font-semibold text-white">{totalCount}</span> results
              {executionTime && (
                <span className="ml-2 text-gray-500">
                  in {executionTime}ms
                </span>
              )}
            </p>
          </div>
        </div>
      )}

      {/* Results List */}
      {results.length === 0 && !isLoading && !error && (
        <div className="card text-center py-12">
          <p className="text-gray-500 text-lg">
            Enter a query above to search the database.
          </p>
        </div>
      )}

      {results.length > 0 && (
        <div className="space-y-3">
          {results.map((node) => (
            <div
              key={node.id}
              className="card card-hover cursor-pointer"
              onClick={() => navigate(`/nodes/${node.id}`)}
            >
              <div className="flex items-start gap-4">
                <div className={clsx('w-12 h-12 rounded-full flex items-center justify-center flex-shrink-0', {
                  'bg-blue-500/20': node.node_type === 'entity',
                  'bg-purple-500/20': node.node_type === 'concept',
                  'bg-green-500/20': node.node_type === 'fact',
                  'bg-gray-500/20': node.node_type === 'raw_chunk',
                })}>
                  <span className="text-2xl">
                    {node.node_type === 'entity' && '🏢'}
                    {node.node_type === 'concept' && '💡'}
                    {node.node_type === 'fact' && '✓'}
                    {node.node_type === 'raw_chunk' && '📄'}
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className={clsx('badge', `badge-${node.node_type}`)}>
                      {node.node_type}
                    </span>
                    <span className="text-xs text-gray-500">
                      {(node.meta.confidence * 100).toFixed(0)}% confidence
                    </span>
                  </div>
                  <p className="text-white line-clamp-2">{node.content}</p>
                  {node.meta.access_score > 0 && (
                    <p className="text-xs text-gray-500 mt-1">
                      Access score: {node.meta.access_score.toFixed(2)}
                    </p>
                  )}
                </div>
                <svg
                  className="w-5 h-5 text-gray-500 flex-shrink-0"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                </svg>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
