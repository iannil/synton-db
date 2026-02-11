/**
 * Graph visualization page with filtering and layout controls.
 */

import { useEffect, useState } from 'react';
import { api } from '@/services/api';
import type { Node, Edge, NodeType } from '@/types/api';
import { GraphViewer } from '@/components/graph/GraphViewer';
import { NodeInspector } from '@/components/graph/NodeInspector';
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
  Skeleton,
} from '@/components/ui';
import { cn } from '@/lib/utils';

const NODE_TYPES: Array<{ value: NodeType | 'all'; label: string }> = [
  { value: 'all', label: 'All Types' },
  { value: 'entity', label: 'Entity' },
  { value: 'concept', label: 'Concept' },
  { value: 'fact', label: 'Fact' },
  { value: 'raw_chunk', label: 'Raw Chunk' },
];

const MAX_NODES_TO_DISPLAY = 200;

export function Graph(): JSX.Element {
  const [allNodes, setAllNodes] = useState<Node[]>([]);
  const [allEdges, setAllEdges] = useState<Edge[]>([]);
  const [filteredNodes, setFilteredNodes] = useState<Node[]>([]);
  const [filteredEdges, setFilteredEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  // Filters
  const [typeFilter, setTypeFilter] = useState<NodeType | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [maxDepth, setMaxDepth] = useState(2);

  // Load all data
  useEffect(() => {
    const loadData = async () => {
      try {
        setIsLoading(true);
        const nodesData = await api.getAllNodes();
        setAllNodes(nodesData);

        // Get edges by traversing from nodes
        const edgesData: Edge[] = [];
        const limitNodes = nodesData.slice(0, 50); // Limit to prevent too many requests

        for (const node of limitNodes) {
          try {
            const traverseData = await api.traverse({
              start_id: node.id,
              max_depth: 1,
              max_nodes: 100,
              direction: 'Forward',
            });
            edgesData.push(...traverseData.edges);
          } catch {
            // Skip errors
          }
        }

        setAllEdges(edgesData);
      } catch (err) {
        console.error('Failed to load graph data:', err);
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, []);

  // Apply filters
  useEffect(() => {
    let filtered = allNodes;

    // Type filter
    if (typeFilter !== 'all') {
      filtered = filtered.filter((n) => n.node_type === typeFilter);
    }

    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((n) => n.content.toLowerCase().includes(query));
    }

    // Limit nodes for performance
    if (filtered.length > MAX_NODES_TO_DISPLAY) {
      filtered = filtered.slice(0, MAX_NODES_TO_DISPLAY);
    }

    setFilteredNodes(filtered);

    // Get edges for filtered nodes
    const nodeIds = new Set(filtered.map((n) => n.id));
    const relatedEdges = allEdges.filter(
      (e) => nodeIds.has(e.source) && nodeIds.has(e.target)
    );
    setFilteredEdges(relatedEdges);
  }, [allNodes, allEdges, typeFilter, searchQuery]);

  const handleNodeClick = (node: Node) => {
    setSelectedNodeId(node.id);
  };

  const handleLoadFromNode = async (nodeId: string) => {
    try {
      const traverseData = await api.traverse({
        start_id: nodeId,
        max_depth: maxDepth,
        max_nodes: 100,
        direction: 'Both',
      });

      // Add new nodes and edges
      const newNodes = traverseData.nodes.filter(
        (n) => !allNodes.some((existing) => existing.id === n.id)
      );
      const newEdges = traverseData.edges.filter(
        (e) => !allEdges.some((existing) =>
          existing.source === e.source && existing.target === e.target && existing.relation === e.relation
        )
      );

      setAllNodes([...allNodes, ...newNodes]);
      setAllEdges([...allEdges, ...newEdges]);
    } catch (err) {
      console.error('Failed to load from node:', err);
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Graph Visualization</h1>
          <p className="text-gray-400 mt-1">
            {filteredNodes.length} nodes, {filteredEdges.length} edges
            {filteredNodes.length >= MAX_NODES_TO_DISPLAY && ' (limited)'}
          </p>
        </div>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap gap-4 items-end">
            <div className="flex-1 min-w-[200px]">
              <Label htmlFor="search">Search nodes</Label>
              <Input
                id="search"
                type="text"
                placeholder="Search by content..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
            <div className="w-48">
              <Label htmlFor="type">Filter by type</Label>
              <Select value={typeFilter} onValueChange={(value) => setTypeFilter(value as NodeType | 'all')}>
                <SelectTrigger id="type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {NODE_TYPES.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="w-48">
              <Label htmlFor="depth">Traverse depth</Label>
              <Select value={maxDepth.toString()} onValueChange={(value) => setMaxDepth(parseInt(value))}>
                <SelectTrigger id="depth">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">1 hop</SelectItem>
                  <SelectItem value="2">2 hops</SelectItem>
                  <SelectItem value="3">3 hops</SelectItem>
                  <SelectItem value="4">4 hops</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Graph */}
      {isLoading ? (
        <div className="flex justify-center items-center" style={{ height: '600px' }}>
          <Skeleton className="h-64 w-96" />
        </div>
      ) : (
        <Card>
          <GraphViewer
            nodes={filteredNodes}
            edges={filteredEdges}
            onNodeClick={handleNodeClick}
            selectedNodeId={selectedNodeId ?? undefined}
            height="calc(100vh - 320px)"
          />
        </Card>
      )}

      {/* Node Inspector Sidebar */}
      <NodeInspector
        nodeId={selectedNodeId}
        onClose={() => setSelectedNodeId(null)}
      />
    </div>
  );
}
