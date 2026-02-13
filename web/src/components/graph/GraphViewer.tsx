/**
 * Graph visualization component using AntV G6.
 */

import { useEffect, useRef, useState, useMemo, useCallback } from 'react';
import { Graph } from '@antv/g6';
import type { Node, Edge, NodeType } from '@/types/api';
import { Button } from '@/components/ui';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui';
import { Plus, Minus, Maximize2 } from 'lucide-react';

interface GraphViewerProps {
  nodes: Node[];
  edges: Edge[];
  onNodeClick?: (node: Node) => void;
  selectedNodeId?: string;
  height?: string;
}

const NODE_TYPE_COLORS: Record<NodeType, string> = {
  entity: '#3498db',
  concept: '#9b59b6',
  fact: '#2ecc71',
  raw_chunk: '#95a5a6',
};

const NODE_TYPE_LABELS: Record<NodeType, string> = {
  entity: '🏢',
  concept: '💡',
  fact: '✓',
  raw_chunk: '📄',
};

export function GraphViewer({
  nodes,
  edges,
  onNodeClick,
  selectedNodeId,
  height = '600px',
}: GraphViewerProps): JSX.Element | null {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);
  const previousSelectedNodeIdRef = useRef<string | undefined>();
  const [zoom, setZoom] = useState(1);
  const [graphInitialized, setGraphInitialized] = useState(false);

  // Keep onNodeClick ref up to date
  const onNodeClickRef = useRef(onNodeClick);
  useEffect(() => {
    onNodeClickRef.current = onNodeClick;
  }, [onNodeClick]);

  // Check if we have data ready to display
  const hasData = nodes.length > 0 && edges.length > 0;

  // Transform data for G6 (memoized to prevent infinite loops)
  const graphData = useMemo(() => ({
    nodes: nodes.map((node) => ({
      id: node.id,
      data: {
        label: NODE_TYPE_LABELS[node.node_type],
        color: NODE_TYPE_COLORS[node.node_type],
        nodeData: node,
      },
    })),
    edges: edges.map((edge, index) => ({
      id: `${edge.source}-${edge.target}-${index}`,
      source: edge.source,
      target: edge.target,
      data: {
        weight: Math.max(1, edge.weight * 5),
        edgeData: edge,
      },
    })),
  }), [nodes, edges]);

  // Initialize G6 graph (only once when container is ready)
  useEffect(() => {
    if (!containerRef.current || graphInitialized) return;

    console.log('Initializing G6 graph container');

    const graph = new Graph({
      container: containerRef.current,
      autoFit: 'view',
      autoResize: true,
      padding: 50,
      node: {
        style: {
          size: 40,
          fill: (datum) => datum.data?.color || '#3498db',
          stroke: '#fff',
          lineWidth: 2,
        },
      },
      edge: {
        style: {
          stroke: '#666',
          lineWidth: 2,
          endArrow: true,
          arrowSize: 8,
        },
      },
      layout: {
        type: 'force',
        preventOverlap: true,
        nodeStrength: -300,
        edgeStrength: 0.1,
        linkDistance: 100,
      },
      behaviors: [
        'zoom-canvas',
        'drag-canvas',
        'drag-element',
      ],
    });

    graphRef.current = graph;
    setGraphInitialized(true);

    // Handle node click
    graph.on('node:click', (event) => {
      const nodeId = event.itemId;
      const nodeData = graph.getNodeData(nodeId);
      if (nodeData?.data?.nodeData && onNodeClickRef.current) {
        onNodeClickRef.current(nodeData.data.nodeData as Node);
      }
    });

    // Handle zoom changes
    graph.on('viewport-change', () => {
      const currentZoom = graph.getZoom();
      setZoom(currentZoom);
    });

    return () => {
      graph.destroy();
      setGraphInitialized(false);
    };
  }, [graphInitialized]);

  // Update graph data when data changes
  useEffect(() => {
    const graph = graphRef.current;
    if (!graph || !hasData) return;

    console.log('Rendering graph data:', { nodes: graphData.nodes.length, edges: graphData.edges.length });
    graph.setData(graphData);
    graph.render();

    // Log container dimensions after render
    setTimeout(() => {
      const container = containerRef.current;
      if (container) {
        const rect = container.getBoundingClientRect();
        console.log('Container dimensions:', { width: rect.width, height: rect.height });
      }
    }, 100);
  }, [graphData, hasData]);

  // Handle selection
  useEffect(() => {
    const graph = graphRef.current;
    if (!graph) return;

    // Clear previous selection
    const previousId = previousSelectedNodeIdRef.current;
    if (previousId) {
      graph.setItemState(previousId, 'selected', false);
    }

    if (selectedNodeId) {
      graph.setItemState(selectedNodeId, 'selected', true);
      // Focus on the selected node
      graph.focusItem(selectedNodeId, true, {
        duration: 300,
        easing: 'ease-cubic-in-out',
      });
      previousSelectedNodeIdRef.current = selectedNodeId;
    } else {
      previousSelectedNodeIdRef.current = undefined;
    }
  }, [selectedNodeId]);

  const handleFit = useCallback(() => {
    const graph = graphRef.current;
    if (graph) {
      graph.fitCenter();
    }
  }, []);

  const handleZoomIn = useCallback(() => {
    const graph = graphRef.current;
    if (graph) {
      const newZoom = (graph.getZoom() || 1) * 1.2;
      graph.zoomTo(newZoom);
    }
  }, []);

  const handleZoomOut = useCallback(() => {
    const graph = graphRef.current;
    if (graph) {
      const newZoom = (graph.getZoom() || 1) * 0.8;
      graph.zoomTo(newZoom);
    }
  }, []);

  // Show loading state while waiting for data
  if (!hasData) {
    return (
      <div className="card p-0 overflow-hidden">
        <div className="flex items-center justify-between px-4 py-2 border-b border-white/10 bg-[#0f3460]">
          <div className="flex items-center gap-4 text-sm text-gray-400">
            <span>Nodes: {nodes.length}</span>
            <span>Edges: {edges.length}</span>
          </div>
        </div>
        <div
          className="bg-[#1a1a2e] flex items-center justify-center"
          style={{ height }}
        >
          <p className="text-gray-400">Loading graph data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="card p-0 overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-white/10 bg-[#0f3460]">
        <div className="flex items-center gap-4 text-sm text-gray-400">
          <span>Nodes: {nodes.length}</span>
          <span>Edges: {edges.length}</span>
          <span>Zoom: {(zoom * 100).toFixed(0)}%</span>
        </div>
        <div className="flex gap-2">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={handleZoomIn}
                  className="h-8 w-8 text-white hover:bg-white/10"
                >
                  <Plus className="w-4 h-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <p>Zoom In</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={handleZoomOut}
                  className="h-8 w-8 text-white hover:bg-white/10"
                >
                  <Minus className="w-4 h-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <p>Zoom Out</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={handleFit}
                  className="h-8 w-8 text-white hover:bg-white/10"
                >
                  <Maximize2 className="w-4 h-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <p>Fit to Screen</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>

      {/* Graph Container */}
      <div
        ref={containerRef}
        style={{ height, minHeight: '400px' }}
        className="bg-[#1a1a2e]"
      />

      {/* Legend */}
      <div className="px-4 py-2 border-t border-white/10 bg-[#0f3460] flex items-center justify-center gap-6 text-sm">
        {Object.entries(NODE_TYPE_LABELS).map(([type, emoji]) => (
          <div key={type} className="flex items-center gap-2">
            <span
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: NODE_TYPE_COLORS[type as NodeType] }}
            />
            <span className="text-gray-300 capitalize">
              {emoji} {type.replace('_', ' ')}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
