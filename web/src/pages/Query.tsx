/**
 * Query page for natural language (PaQL) and GraphRAG hybrid search.
 */

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node } from '@/types/api';
import { Button, Textarea, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Card, CardContent, Alert, AlertDescription, Badge, Label } from '@/components/ui';
import { cn } from '@/lib/utils';
import { ChevronRight, Building, Lightbulb, CheckCircle, FileText } from 'lucide-react';

const QUERY_HISTORY_KEY = 'syntondb_query_history';

const SEARCH_TYPES = [
  { value: 'hybrid', label: 'GraphRAG Hybrid Search' },
  { value: 'text', label: 'Text Query' },
];

const NODE_TYPE_ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  entity: Building,
  concept: Lightbulb,
  fact: CheckCircle,
  raw_chunk: FileText,
};

const NODE_TYPE_COLORS: Record<string, string> = {
  entity: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  concept: 'bg-purple-500/20 text-purple-400 border border-purple-500/30',
  fact: 'bg-green-500/20 text-green-400 border border-green-500/30',
  raw_chunk: 'bg-gray-500/20 text-gray-400 border border-gray-500/30',
};

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
      <Card className="p-6">
        <div className="space-y-4">
          <div className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-[200px]">
              <Label htmlFor="search-type">Search Type</Label>
              <Select value={searchType} onValueChange={(value) => setSearchType(value as 'hybrid' | 'text')}>
                <SelectTrigger id="search-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SEARCH_TYPES.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="w-32">
              <Label htmlFor="limit">Results</Label>
              <Select value={limit.toString()} onValueChange={(value) => setLimit(parseInt(value))}>
                <SelectTrigger id="limit">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {['5', '10', '20', '50', '100'].map((value) => (
                    <SelectItem key={value} value={value}>
                      {value}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div>
            <Label htmlFor="query">Query</Label>
            <Textarea
              id="query"
              placeholder="Enter your query... (Cmd/Ctrl + Enter to search)"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyPress}
              rows={3}
              className="mt-1.5"
            />
            <p className="text-xs text-gray-500 mt-1">
              Example: "Find all nodes related to artificial intelligence"
            </p>
          </div>

          <div className="flex justify-between items-center">
            <Button onClick={handleSearch} disabled={isLoading}>
              {isLoading ? 'Searching...' : 'Search'}
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
      </Card>

      {/* Error */}
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Results Summary */}
      {results.length > 0 && (
        <Card>
          <CardContent className="py-4">
            <p className="text-gray-300">
              Found <span className="font-semibold text-white">{totalCount}</span> results
              {executionTime && (
                <span className="ml-2 text-gray-500">
                  in {executionTime}ms
                </span>
              )}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Results List */}
      {results.length === 0 && !isLoading && !error && (
        <Card>
          <CardContent className="text-center py-12">
            <p className="text-gray-500 text-lg">
              Enter a query above to search the database.
            </p>
          </CardContent>
        </Card>
      )}

      {results.length > 0 && (
        <div className="space-y-3">
          {results.map((node) => {
            const NodeTypeIcon = NODE_TYPE_ICONS[node.node_type] || FileText;
            return (
              <Card
                key={node.id}
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => navigate(`/nodes/${node.id}`)}
              >
                <CardContent className="p-4">
                  <div className="flex items-start gap-4">
                    <div className={cn(
                      'w-12 h-12 rounded-full flex items-center justify-center flex-shrink-0 border',
                      NODE_TYPE_COLORS[node.node_type] || NODE_TYPE_COLORS.raw_chunk
                    )}>
                      <NodeTypeIcon className="w-6 h-6" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <Badge variant="secondary" className="text-xs">
                          {node.node_type}
                        </Badge>
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
                    <ChevronRight className="w-5 h-5 text-gray-500 flex-shrink-0" />
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
