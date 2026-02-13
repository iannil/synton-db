/**
 * Node detail page.
 */

import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, Edge } from '@/types/api';
import {
  Button,
  Card,
  CardContent,
  Badge,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Skeleton,
} from '@/components/ui';
import { cn } from '@/lib/utils';
import { ArrowLeft, Trash2, Building, Lightbulb, CheckCircle, FileText } from 'lucide-react';

const NODE_TYPE_ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  entity: Building,
  concept: Lightbulb,
  fact: CheckCircle,
  raw_chunk: FileText,
};

const NODE_TYPE_COLORS: Record<string, string> = {
  entity: 'bg-blue-500/20 text-blue-400 border border-blue-500/30',
  concept: 'bg-purple-500/20 text-purple-400 border border-purple-500/30',
  fact: 'bg-green-500/20 text-green-400 border border-green-500/30',
  raw_chunk: 'bg-gray-500/20 text-gray-400 border border-gray-500/30',
};

export function NodeDetail(): JSX.Element {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [node, setNode] = useState<Node | null>(null);
  const [incomingEdges, setIncomingEdges] = useState<Edge[]>([]);
  const [outgoingEdges, setOutgoingEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  useEffect(() => {
    const loadData = async () => {
      if (!id) return;

      try {
        setIsLoading(true);
        const response = await api.getNode({ id });

        if (!response.node) {
          setError('Node not found');
          return;
        }

        setNode(response.node);

        // Load related edges through traverse
        const traverseResponse = await api.traverse({
          start_id: id,
          max_depth: 1,
          max_nodes: 100,
          direction: 'Both',
        });

        setOutgoingEdges(
          traverseResponse.edges.filter((e) => e.source === id)
        );
        setIncomingEdges(
          traverseResponse.edges.filter((e) => e.target === id)
        );
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load node');
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [id]);

  const handleDelete = async () => {
    if (!node) return;

    try {
      await api.deleteNode({ id: node.id });
      navigate('/nodes');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete node');
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Skeleton className="h-64 w-96" />
      </div>
    );
  }

  if (error || !node) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-destructive text-lg">{error || 'Node not found'}</p>
          <Button variant="outline" className="mt-4" onClick={() => navigate('/nodes')}>
            Back to Nodes
          </Button>
        </div>
      </div>
    );
  }

  const NodeTypeIcon = NODE_TYPE_ICONS[node.node_type] || FileText;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="sm" onClick={() => navigate('/nodes')}>
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back
          </Button>
          <h1 className="text-2xl font-bold text-white">Node Details</h1>
        </div>
        <Button
          variant="destructive"
          onClick={() => setDeleteDialogOpen(true)}
        >
          <Trash2 className="w-4 h-4 mr-2" />
          Delete Node
        </Button>
      </div>

      {/* Node Info */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-start gap-4">
            <div className={cn(
              'w-16 h-16 rounded-full flex items-center justify-center flex-shrink-0 border',
              NODE_TYPE_COLORS[node.node_type] || NODE_TYPE_COLORS.raw_chunk
            )}>
              <NodeTypeIcon className="w-8 h-8" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 mb-2">
                <Badge
                  className={NODE_TYPE_COLORS[node.node_type] || NODE_TYPE_COLORS.raw_chunk}
                >
                  {node.node_type}
                </Badge>
                <span className="text-sm text-gray-500">
                  ID: {node.id}
                </span>
              </div>
              <p className="text-lg text-white">{node.content}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Metadata */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardContent className="p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Metadata</h2>
            <dl className="space-y-3">
              <div className="flex justify-between">
                <dt className="text-gray-400">Created</dt>
                <dd className="text-white">
                  {new Date(node.meta.created_at).toLocaleString()}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Updated</dt>
                <dd className="text-white">
                  {new Date(node.meta.updated_at).toLocaleString()}
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Source</dt>
                <dd className="text-white capitalize">{node.meta.source.replace('_', ' ')}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Confidence</dt>
                <dd className="text-white">
                  {(node.meta.confidence * 100).toFixed(0)}%
                </dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Access Score</dt>
                <dd className="text-white">{node.meta.access_score.toFixed(2)}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Has Embedding</dt>
                <dd className="text-white">{node.embedding ? 'Yes' : 'No'}</dd>
              </div>
            </dl>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Connections</h2>
            <dl className="space-y-3">
              <div className="flex justify-between">
                <dt className="text-gray-400">Incoming Edges</dt>
                <dd className="text-white">{incomingEdges.length}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-400">Outgoing Edges</dt>
                <dd className="text-white">{outgoingEdges.length}</dd>
              </div>
            </dl>
          </CardContent>
        </Card>
      </div>

      {/* Outgoing Edges */}
      {outgoingEdges.length > 0 && (
        <Card>
          <CardContent className="p-6">
            <h2 className="text-lg font-semibold text-white mb-4">
              Outgoing Connections ({outgoingEdges.length})
            </h2>
            <div className="space-y-2">
              {outgoingEdges.map((edge, index) => (
                <div
                  key={`${edge.source}-${edge.target}-${edge.relation}-${index}`}
                  className="flex items-center gap-3 p-3 rounded-lg bg-muted/50"
                >
                  <span className="text-primary">→</span>
                  <span className="text-purple-400">{edge.relation}</span>
                  <span className="text-gray-400">→</span>
                  <button
                    onClick={() => navigate(`/nodes/${edge.target}`)}
                    className="text-blue-400 hover:text-blue-300 truncate flex-1 text-left"
                  >
                    {edge.target}
                  </button>
                  <span className="text-sm text-gray-500">
                    w: {edge.weight.toFixed(2)}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Incoming Edges */}
      {incomingEdges.length > 0 && (
        <Card>
          <CardContent className="p-6">
            <h2 className="text-lg font-semibold text-white mb-4">
              Incoming Connections ({incomingEdges.length})
            </h2>
            <div className="space-y-2">
              {incomingEdges.map((edge, index) => (
                <div
                  key={`${edge.source}-${edge.target}-${edge.relation}-${index}`}
                  className="flex items-center gap-3 p-3 rounded-lg bg-muted/50"
                >
                  <button
                    onClick={() => navigate(`/nodes/${edge.source}`)}
                    className="text-blue-400 hover:text-blue-300 truncate flex-1 text-right"
                  >
                    {edge.source}
                  </button>
                  <span className="text-gray-400">→</span>
                  <span className="text-purple-400">{edge.relation}</span>
                  <span className="text-primary">→</span>
                  <span className="text-sm text-gray-500">
                    w: {edge.weight.toFixed(2)}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Delete Confirmation */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Node</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{node.content}"? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDelete}>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
