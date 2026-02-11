/**
 * Traverse page for graph traversal with animation visualization.
 */

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, Edge, TraverseDirection } from '@/types/api';
import { Select, SelectInput, Button } from '@/components/ui';
import { GraphViewer } from '@/components/graph/GraphViewer';

const DIRECTIONS = [
  { value: 'forward', label: 'Forward (outgoing)' },
  { value: 'backward', label: 'Backward (incoming)' },
  { value: 'both', label: 'Both directions' },
];

export function Traverse(): JSX.Element {
  const navigate = useNavigate();

  const [nodes, setNodes] = useState<Node[]>([]);
  const [startNodeId, setStartNodeId] = useState<string>('');
  const [direction, setDirection] = useState<TraverseDirection>('forward');
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
      <div className="card space-y-4">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* Start Node */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Start Node
            </label>
            {isLoadingNodes ? (
              <p className="text-gray-500 text-sm">Loading nodes...</p>
            ) : (
              <SelectInput
                value={startNodeId}
                onChange={(e) => setStartNodeId(e.target.value)}
                fullWidth
              >
                {nodes.map((node) => (
                  <option key={node.id} value={node.id}>
                    [{node.node_type}] {node.content.slice(0, 40)}
                  </option>
                ))}
              </SelectInput>
            )}
          </div>

          {/* Direction */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Direction
            </label>
            <Select
              options={DIRECTIONS}
              value={direction}
              onChange={(e) => setDirection(e.target.value as TraverseDirection)}
            />
          </div>

          {/* Max Depth */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Max Depth: {maxDepth}
            </label>
            <input
              type="range"
              min="1"
              max="5"
              value={maxDepth}
              onChange={(e) => setMaxDepth(parseInt(e.target.value))}
              className="w-full mt-2"
            />
          </div>

          {/* Max Nodes */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Max Nodes: {maxNodes}
            </label>
            <input
              type="range"
              min="10"
              max="200"
              step="10"
              value={maxNodes}
              onChange={(e) => setMaxNodes(parseInt(e.target.value))}
              className="w-full mt-2"
            />
          </div>
        </div>

        <div className="flex gap-3">
          <Button
            onClick={handleTraverse}
            isLoading={isTraversing}
            disabled={!startNodeId || isAnimating}
          >
            Start Traversal
          </Button>
          {traversedNodes.length > 0 && (
            <Button
              variant="ghost"
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
      </div>

      {/* Error */}
      {error && (
        <div className="p-4 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400">
          {error}
        </div>
      )}

      {/* Results */}
      {traversedNodes.length > 0 && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Graph Visualization */}
          <div className="lg:col-span-2">
            <GraphViewer
              nodes={traversedNodes}
              edges={traversedEdges}
              height="500px"
            />
          </div>

          {/* Stats & Node List */}
          <div className="space-y-4">
            {/* Stats */}
            <div className="card">
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

              {/* Depth Indicator */}
              <div className="mt-4">
                <div className="flex gap-1">
                  {Array.from({ length: maxDepth }).map((_, i) => (
                    <div
                      key={i}
                      className={clsx(
                        'flex-1 h-2 rounded-full transition-all duration-300',
                        i < currentDepth ? 'bg-[#e94560]' : 'bg-gray-700'
                      )}
                    />
                  ))}
                </div>
              </div>
            </div>

            {/* Start Node */}
            {startNode && (
              <div className="card">
                <h2 className="text-lg font-semibold text-white mb-3">Starting Node</h2>
                <div className="flex items-center gap-3">
                  <div className={clsx('w-10 h-10 rounded-full flex items-center justify-center', {
                    'bg-blue-500/20': startNode.node_type === 'entity',
                    'bg-purple-500/20': startNode.node_type === 'concept',
                    'bg-green-500/20': startNode.node_type === 'fact',
                    'bg-gray-500/20': startNode.node_type === 'raw_chunk',
                  })}>
                    <span className="text-xl">
                      {startNode.node_type === 'entity' && '🏢'}
                      {startNode.node_type === 'concept' && '💡'}
                      {startNode.node_type === 'fact' && '✓'}
                      {startNode.node_type === 'raw_chunk' && '📄'}
                    </span>
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-white line-clamp-2">{startNode.content}</p>
                    <button
                      onClick={() => navigate(`/nodes/${startNode.id}`)}
                      className="text-xs text-[#e94560] hover:underline mt-1"
                    >
                      View details →
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Discovered Nodes */}
            <div className="card">
              <h2 className="text-lg font-semibold text-white mb-3">
                Discovered Nodes ({traversedNodes.length})
              </h2>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {traversedNodes.map((node) => (
                  <div
                    key={node.id}
                    className="flex items-center gap-2 p-2 rounded bg-white/5 hover:bg-white/10 cursor-pointer transition-colors"
                    onClick={() => navigate(`/nodes/${node.id}`)}
                  >
                    <span className={clsx('w-2 h-2 rounded-full', {
                      'bg-blue-500': node.node_type === 'entity',
                      'bg-purple-500': node.node_type === 'concept',
                      'bg-green-500': node.node_type === 'fact',
                      'bg-gray-500': node.node_type === 'raw_chunk',
                    })} />
                    <span className="text-xs text-white truncate flex-1">
                      {node.content.slice(0, 30)}{node.content.length > 30 ? '...' : ''}
                    </span>
                    {node.id === startNodeId && (
                      <span className="text-xs text-[#e94560]">Start</span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
