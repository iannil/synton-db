/**
 * Traverse page for graph traversal with animation visualization.
 */

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, Edge, TraverseDirection } from '@/types/api';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Button,
  Card,
  CardContent,
  Input,
  Label,
  Slider,
  Alert,
  AlertDescription,
  Skeleton,
  Badge,
  Progress,
} from '@/components/ui';
import { GraphViewer } from '@/components/graph/GraphViewer';
import { cn } from '@/lib/utils';

const DIRECTIONS = [
  { value: 'Forward', label: 'Forward (outgoing)' },
  { value: 'Backward', label: 'Backward (incoming)' },
  { value: 'Both', label: 'Both directions' },
];

const NODE_TYPE_ICONS: Record<string, string> = {
  entity: '🏢',
  concept: '💡',
  fact: '✓',
  raw_chunk: '📄',
};

const NODE_TYPE_COLORS: Record<string, string> = {
  entity: 'bg-blue-500',
  concept: 'bg-purple-500',
  fact: 'bg-green-500',
  raw_chunk: 'bg-gray-500',
};

export function Traverse(): JSX.Element {
  const navigate = useNavigate();

  const [nodes, setNodes] = useState<Node[]>([]);
  const [startNodeId, setStartNodeId] = useState<string>('');
  const [direction, setDirection] = useState<TraverseDirection>('Forward');
  const [maxDepth, setMaxDepth] = useState(2);
  const [maxNodes, setMaxNodes] = useState(50);

  // Traversal results
  const [traversedNodes, setTraversedNodes] = useState<Node[]>([]);
  const [traversedEdges, setTraversedEdges] = useState<Edge[]>([]);
  const [currentDepth, setCurrentDepth] = useState(0);
  const [isAnimating, setIsAnimating] = useState(false);

  const [isLoadingNodes, setIsLoadingNodes] = useState(true);
  const [isTraversing, setIsTraversing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load all nodes on mount
  useEffect(() => {
    const loadNodes = async () => {
      try {
        setIsLoadingNodes(true);
        const data = await api.getAllNodes();
        setNodes(data);
        if (data.length > 0) {
          setStartNodeId(data[0].id);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load nodes');
      } finally {
        setIsLoadingNodes(false);
      }
    };

    loadNodes();
  }, []);

  const handleTraverse = async () => {
    if (!startNodeId) return;

    setIsTraversing(true);
    setError(null);
    setTraversedNodes([]);
    setTraversedEdges([]);
    setCurrentDepth(0);

    try {
      // Animate traversal step by step
      for (let depth = 1; depth <= maxDepth; depth++) {
        setIsAnimating(true);

        const response = await api.traverse({
          start_id: startNodeId,
          max_depth: depth,
          max_nodes: maxNodes,
          direction,
        });

        setTraversedNodes(response.nodes);
        setTraversedEdges(response.edges);
        setCurrentDepth(depth);

        // Add a small delay for animation effect
        if (depth < maxDepth) {
          await new Promise(resolve => setTimeout(resolve, 500));
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Traversal failed');
    } finally {
      setIsTraversing(false);
      setIsAnimating(false);
    }
  };

  const getNodeLabel = (nodeId: string): string => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return nodeId;
    return node.content.length > 30 ? node.content.slice(0, 30) + '...' : node.content;
  };

  const startNode = nodes.find((n) => n.id === startNodeId);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Graph Traversal</h1>
        <p className="text-gray-400 mt-1">
          Visualize BFS traversal from a starting node
        </p>
      </div>

      {/* Controls */}
      <Card>
        <CardContent className="p-6 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Start Node */}
            <div className="space-y-2">
              <Label htmlFor="start-node">Start Node</Label>
              {isLoadingNodes ? (
                <p className="text-gray-500 text-sm">Loading nodes...</p>
              ) : (
                <select
                  id="start-node"
                  value={startNodeId}
                  onChange={(e) => setStartNodeId(e.target.value)}
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  {nodes.map((node) => (
                    <option key={node.id} value={node.id}>
                      [{node.node_type}] {node.content.slice(0, 40)}
                    </option>
                  ))}
                </select>
              )}
            </div>

            {/* Direction */}
            <div className="space-y-2">
              <Label htmlFor="direction">Direction</Label>
              <Select value={direction} onValueChange={(value) => setDirection(value as TraverseDirection)}>
                <SelectTrigger id="direction">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {DIRECTIONS.map((dir) => (
                    <SelectItem key={dir.value} value={dir.value}>
                      {dir.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Max Depth */}
            <div className="space-y-2">
              <Label>Max Depth: {maxDepth}</Label>
              <Slider
                value={[maxDepth]}
                onValueChange={(value) => setMaxDepth(value[0])}
                min={1}
                max={5}
                step={1}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-gray-500">
                <span>1</span>
                <span>5</span>
              </div>
            </div>

            {/* Max Nodes */}
            <div className="space-y-2">
              <Label>Max Nodes: {maxNodes}</Label>
              <Slider
                value={[maxNodes]}
                onValueChange={(value) => setMaxNodes(value[0])}
                min={10}
                max={200}
                step={10}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-gray-500">
                <span>10</span>
                <span>200</span>
              </div>
            </div>
          </div>

          <div className="flex gap-3">
            <Button
              onClick={handleTraverse}
              disabled={!startNodeId || isAnimating}
            >
              {isTraversing ? 'Traversing...' : 'Start Traversal'}
            </Button>
            {traversedNodes.length > 0 && (
              <Button
                variant="outline"
                onClick={() => {
                  setTraversedNodes([]);
                  setTraversedEdges([]);
                  setCurrentDepth(0);
                }}
              >
                Clear
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Error */}
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Results */}
      {traversedNodes.length > 0 && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Graph Visualization */}
          <div className="lg:col-span-2">
            <Card>
              <GraphViewer
                nodes={traversedNodes}
                edges={traversedEdges}
                height="500px"
              />
            </Card>
          </div>

          {/* Stats & Node List */}
          <div className="space-y-4">
            {/* Stats */}
            <Card>
              <CardContent className="p-6">
                <h2 className="text-lg font-semibold text-white mb-4">Traversal Stats</h2>
                <dl className="space-y-3">
                  <div className="flex justify-between">
                    <dt className="text-gray-400">Nodes Found</dt>
                    <dd className="text-white font-semibold">{traversedNodes.length}</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-gray-400">Edges Found</dt>
                    <dd className="text-white font-semibold">{traversedEdges.length}</dd>
                  </div>
                  <div className="flex justify-between">
                    <dt className="text-gray-400">Current Depth</dt>
                    <dd className="text-white font-semibold">
                      {currentDepth} / {maxDepth}
                    </dd>
                  </div>
                </dl>

                {/* Depth Progress */}
                <div className="mt-4">
                  <Progress value={(currentDepth / maxDepth) * 100} />
                  <div className="flex justify-between text-xs text-gray-500 mt-1">
                    <span>Depth {currentDepth}</span>
                    <span>of {maxDepth}</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Start Node */}
            {startNode && (
              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold text-white mb-3">Starting Node</h2>
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      'w-10 h-10 rounded-full flex items-center justify-center',
                      NODE_TYPE_COLORS[startNode.node_type] || NODE_TYPE_COLORS.raw_chunk
                    )}>
                      <span className="text-xl">
                        {NODE_TYPE_ICONS[startNode.node_type] || '📄'}
                      </span>
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-white line-clamp-2">{startNode.content}</p>
                      <button
                        onClick={() => navigate(`/nodes/${startNode.id}`)}
                        className="text-xs text-primary hover:underline mt-1"
                      >
                        View details →
                      </button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            )}

            {/* Discovered Nodes */}
            <Card>
              <CardContent className="p-6">
                <h2 className="text-lg font-semibold text-white mb-3">
                  Discovered Nodes ({traversedNodes.length})
                </h2>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {traversedNodes.map((node) => (
                    <div
                      key={node.id}
                      className="flex items-center gap-2 p-2 rounded bg-muted/50 hover:bg-muted/80 cursor-pointer transition-colors"
                      onClick={() => navigate(`/nodes/${node.id}`)}
                    >
                      <span className={cn(
                        'w-2 h-2 rounded-full',
                        NODE_TYPE_COLORS[node.node_type] || NODE_TYPE_COLORS.raw_chunk
                      )} />
                      <span className="text-xs text-white truncate flex-1">
                        {node.content.slice(0, 30)}{node.content.length > 30 ? '...' : ''}
                      </span>
                      {node.id === startNodeId && (
                        <Badge variant="default" className="text-[10px] h-5">
                          Start
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}
    </div>
  );
}
