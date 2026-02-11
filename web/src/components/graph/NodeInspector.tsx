/**
 * Node inspector component for displaying node details in a sidebar.
 */

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, Edge } from '@/types/api';
import { Button } from '@/components/ui';

interface NodeInspectorProps {
  nodeId: string | null;
  onClose: () => void;
}

export function NodeInspector({ nodeId, onClose }: NodeInspectorProps): JSX.Element | null {
  const navigate = useNavigate();
  const [node, setNode] = useState<Node | null>(null);
  const [incomingEdges, setIncomingEdges] = useState<Edge[]>([]);
  const [outgoingEdges, setOutgoingEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!nodeId) {
      setNode(null);
      setIncomingEdges([]);
      setOutgoingEdges([]);
      return;
    }

    const loadNodeData = async () => {
      setIsLoading(true);
      try {
        const response = await api.getNode({ id: nodeId });
        setNode(response.node || null);

        if (response.node) {
          const traverseResponse = await api.traverse({
            start_id: nodeId,
            max_depth: 1,
            max_nodes: 100,
            direction: 'Both',
          });

          setOutgoingEdges(traverseResponse.edges.filter((e) => e.source === nodeId));
          setIncomingEdges(traverseResponse.edges.filter((e) => e.target === nodeId));
        }
      } catch (err) {
        console.error('Failed to load node:', err);
      } finally {
        setIsLoading(false);
      }
    };

    loadNodeData();
  }, [nodeId]);

  if (!nodeId) return null;

  return (
    <div className="fixed right-0 top-0 h-full w-80 bg-[#16213e] border-l border-white/10 shadow-2xl z-40 overflow-y-auto">
      <div className="sticky top-0 bg-[#16213e] border-b border-white/10 p-4 flex items-center justify-between z-10">
        <h2 className="text-lg font-semibold text-white">Node Details</h2>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-white transition-colors"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="p-4 space-y-4">
        {isLoading ? (
          <div className="flex justify-center py-8">
            <div className="spinner" />
          </div>
        ) : node ? (
          <>
            {/* Node Type Badge */}
            <span className={clsx('badge', `badge-${node.node_type}`)}>
              {node.node_type}
            </span>

            {/* Content */}
            <div>
              <h3 className="text-sm font-medium text-gray-400 mb-1">Content</h3>
              <p className="text-white">{node.content}</p>
            </div>

            {/* ID */}
            <div>
              <h3 className="text-sm font-medium text-gray-400 mb-1">ID</h3>
              <p className="text-sm text-gray-300 font-mono break-all">{node.id}</p>
            </div>

            {/* Metadata */}
            <div className="space-y-2">
              <h3 className="text-sm font-medium text-gray-400">Metadata</h3>
              <div className="text-sm space-y-1">
                <div className="flex justify-between">
                  <span className="text-gray-500">Created</span>
                  <span className="text-white">{new Date(node.meta.created_at).toLocaleDateString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">Confidence</span>
                  <span className="text-white">{(node.meta.confidence * 100).toFixed(0)}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">Access Score</span>
                  <span className="text-white">{node.meta.access_score.toFixed(2)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">Embedding</span>
                  <span className="text-white">{node.embedding ? 'Yes' : 'No'}</span>
                </div>
              </div>
            </div>

            {/* Outgoing Edges */}
            {outgoingEdges.length > 0 && (
              <div>
                <h3 className="text-sm font-medium text-gray-400 mb-2">
                  Outgoing ({outgoingEdges.length})
                </h3>
                <div className="space-y-1">
                  {outgoingEdges.map((edge, index) => (
                    <div
                      key={`${edge.target}-${index}`}
                      className="text-xs p-2 rounded bg-white/5 flex items-center gap-2"
                    >
                      <span className="text-[#e94560]">→</span>
                      <span className="text-purple-400">{edge.relation}</span>
                      <span className="text-gray-500 truncate">{edge.target.slice(0, 8)}...</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Incoming Edges */}
            {incomingEdges.length > 0 && (
              <div>
                <h3 className="text-sm font-medium text-gray-400 mb-2">
                  Incoming ({incomingEdges.length})
                </h3>
                <div className="space-y-1">
                  {incomingEdges.map((edge, index) => (
                    <div
                      key={`${edge.source}-${index}`}
                      className="text-xs p-2 rounded bg-white/5 flex items-center gap-2"
                    >
                      <span className="text-gray-500 truncate">{edge.source.slice(0, 8)}...</span>
                      <span className="text-purple-400">{edge.relation}</span>
                      <span className="text-[#e94560]">→</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Actions */}
            <div className="pt-4 space-y-2">
              <Button
                fullWidth
                onClick={() => navigate(`/nodes/${node.id}`)}
              >
                View Full Details
              </Button>
            </div>
          </>
        ) : (
          <p className="text-gray-500 text-center">Node not found</p>
        )}
      </div>
    </div>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
