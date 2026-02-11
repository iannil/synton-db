/**
 * Graph visualization component using Cytoscape.js.
 */

import { useEffect, useRef, useState } from 'react';
import cytoscape, { Core, ElementDefinition } from 'cytoscape';
import type { Node, Edge, NodeType } from '@/types/api';

// Register the layout extension
import coseBilkent from 'cytoscape-cose-bilkent';
cytoscape.use(coseBilkent);

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
  const cyRef = useRef<Core | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });

  // Initialize Cytoscape
  useEffect(() => {
    if (!containerRef.current) return;

    const cy = cytoscape({
      container: containerRef.current,
      elements: [],
      style: [
        {
          selector: 'node',
          style: {
            'background-color': 'data(color)',
            'label': 'data(label)',
            'text-valign': 'center',
            'text-halign': 'center',
            'width': '40px',
            'height': '40px',
            'font-size': '10px',
            'color': '#fff',
            'text-outline-color': '#000',
            'text-outline-width': '2px',
            'border-width': 2,
            'border-color': '#fff',
          },
        },
        {
          selector: 'node:selected',
          style: {
            'border-width': 4,
            'border-color': '#e94560',
          },
        },
        {
          selector: 'edge',
          style: {
            'width': 'data(width)',
            'line-color': '#666',
            'target-arrow-color': '#666',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            'arrow-scale': 0.8,
          },
        },
        {
          selector: 'edge:selected',
          style: {
            'line-color': '#e94560',
            'target-arrow-color': '#e94560',
          },
        },
      ],
      layout: {
        name: 'cose-bilkent',
        animate: true,
        animationDuration: 500,
        fit: true,
        padding: 50,
        nodeRepulsion: 4500,
        idealEdgeLength: 50,
        edgeElasticity: 0.45,
        nestingFactor: 0.1,
      },
      minZoom: 0.1,
      maxZoom: 3,
    });

    cyRef.current = cy;

    // Event handlers
    cy.on('tap', 'node', (evt) => {
      const node = evt.target;
      const nodeId = node.id();
      const clickedNode = nodes.find((n) => n.id === nodeId);
      if (clickedNode && onNodeClick) {
        onNodeClick(clickedNode);
      }
    });

    cy.on('zoom', () => {
      setZoom(cy.zoom());
      setPan({ x: cy.pan().x, y: cy.pan().y });
    });

    cy.on('pan', () => {
      setPan({ x: cy.pan().x, y: cy.pan().y });
    });

    return () => {
      cy.destroy();
    };
  }, []);

  // Update graph data
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;

    const elements: ElementDefinition[] = [
      ...nodes.map((node) => ({
        data: {
          id: node.id,
          label: NODE_TYPE_LABELS[node.node_type],
          color: NODE_TYPE_COLORS[node.node_type],
          nodeData: node,
        },
      })),
      ...edges.map((edge, index) => ({
        data: {
          id: `${edge.source}-${edge.target}-${index}`,
          source: edge.source,
          target: edge.target,
          width: Math.max(1, edge.weight * 5),
          edgeData: edge,
        },
      })),
    ];

    cy.json({ elements });
    cy.layout({
      name: 'cose-bilkent',
      animate: true,
      animationDuration: 500,
      fit: true,
      padding: 50,
    }).run();
  }, [nodes, edges]);

  // Handle selection
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;

    cy.elements().unselect();
    if (selectedNodeId) {
      const node = cy.getElementById(selectedNodeId);
      if (node.length > 0) {
        node.select();
        cy.animate({
          center: { eles: node },
          zoom: 1.5,
        }, {
          duration: 300
        });
      }
    }
  }, [selectedNodeId]);

  const handleFit = () => {
    cyRef.current?.fit(undefined, 50);
  };

  const handleZoomIn = () => {
    cyRef.current?.zoom({
      level: (cyRef.current.zoom() || 1) * 1.2,
      renderedPosition: { x: containerRef.current?.offsetWidth || 0, y: containerRef.current?.offsetHeight || 0 }
    });
  };

  const handleZoomOut = () => {
    cyRef.current?.zoom({
      level: (cyRef.current.zoom() || 1) * 0.8,
      renderedPosition: { x: containerRef.current?.offsetWidth || 0, y: containerRef.current?.offsetHeight || 0 }
    });
  };

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
          <button
            onClick={handleZoomIn}
            className="p-1.5 rounded hover:bg-white/10 text-white"
            title="Zoom In"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
          </button>
          <button
            onClick={handleZoomOut}
            className="p-1.5 rounded hover:bg-white/10 text-white"
            title="Zoom Out"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
            </svg>
          </button>
          <button
            onClick={handleFit}
            className="p-1.5 rounded hover:bg-white/10 text-white"
            title="Fit to Screen"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
            </svg>
          </button>
        </div>
      </div>

      {/* Graph Container */}
      <div
        ref={containerRef}
        style={{ height }}
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
