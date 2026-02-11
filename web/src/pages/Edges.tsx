/**
 * Edges management page with filtering and creation.
 */

import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { api } from '@/services/api';
import type { Edge, Node, Relation } from '@/types/api';
import { Button, Input, Select, SelectInput, Modal } from '@/components/ui';

const RELATION_TYPES: Relation[] = [
  'is_part_of',
  'causes',
  'contradicts',
  'happened_after',
  'similar_to',
  'is_a',
  'located_at',
  'belongs_to',
];

const ITEMS_PER_PAGE = 20;

export function Edges(): JSX.Element {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [relationFilter, setRelationFilter] = useState<string>('all');
  const [page, setPage] = useState(1);

  // Create edge form
  const [createModalOpen, setCreateModalOpen] = useState(
    searchParams.get('action') === 'create'
  );
  const [sourceId, setSourceId] = useState('');
  const [targetId, setTargetId] = useState('');
  const [relation, setRelation] = useState<Relation>('similar_to');
  const [weight, setWeight] = useState(1.0);
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    const loadData = async () => {
      try {
        setIsLoading(true);
        const nodesData = await api.getAllNodes();
        setNodes(nodesData);

        // Get edges by traversing from all nodes (simplified approach)
        const allEdges: Edge[] = [];
        for (const node of nodesData.slice(0, 50)) {
          // Limit to prevent too many requests
          try {
            const traverseData = await api.traverse({
              start_id: node.id,
              max_depth: 1,
              max_nodes: 100,
              direction: 'forward',
            });
            allEdges.push(...traverseData.edges);
          } catch {
            // Skip errors
          }
        }
        setEdges(allEdges);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, []);

  const filteredEdges = edges.filter((edge) =>
    relationFilter === 'all' || edge.relation === relationFilter
  );

  const paginatedEdges = filteredEdges.slice(
    (page - 1) * ITEMS_PER_PAGE,
    page * ITEMS_PER_PAGE
  );

  const totalPages = Math.ceil(filteredEdges.length / ITEMS_PER_PAGE);

  const handleCreate = async () => {
    if (!sourceId || !targetId) return;

    try {
      setIsCreating(true);
      const response = await api.addEdge({
        source: sourceId,
        target: targetId,
        relation,
        weight,
      });

      setEdges([...edges, response.edge]);
      setCreateModalOpen(false);
      setSourceId('');
      setTargetId('');
      setRelation('similar_to');
      setWeight(1.0);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create edge');
    } finally {
      setIsCreating(false);
    }
  };

  const getNodeContent = (nodeId: string): string => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return nodeId;
    return node.content.length > 30
      ? node.content.slice(0, 30) + '...'
      : node.content;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Edges</h1>
          <p className="text-gray-400 mt-1">
            {filteredEdges.length} {filteredEdges.length === 1 ? 'edge' : 'edges'}
          </p>
        </div>
        <Button onClick={() => setCreateModalOpen(true)}>+ Add Edge</Button>
      </div>

      {/* Filters */}
      <div className="card">
        <div className="flex flex-wrap gap-4">
          <div className="w-48">
            <Select
              options={[
                { value: 'all', label: 'All Relations' },
                ...RELATION_TYPES.map((r) => ({ value: r, label: r })),
              ]}
              value={relationFilter}
              onChange={(e) => setRelationFilter(e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="p-4 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400">
          {error}
        </div>
      )}

      {/* Loading */}
      {isLoading ? (
        <div className="flex justify-center py-12">
          <div className="spinner" />
        </div>
      ) : (
        <>
          {/* Edges List */}
          {paginatedEdges.length === 0 ? (
            <div className="card text-center py-12">
              <p className="text-gray-500 text-lg">
                {relationFilter !== 'all'
                  ? 'No edges match your filters.'
                  : 'No edges yet. Create your first edge!'}
              </p>
            </div>
          ) : (
            <div className="card overflow-hidden p-0">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-white/5">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Source
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Relation
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Target
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Weight
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Created
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/10">
                    {paginatedEdges.map((edge, index) => (
                      <tr
                        key={`${edge.source}-${edge.target}-${edge.relation}-${index}`}
                        className="hover:bg-white/5 transition-colors"
                      >
                        <td className="px-6 py-4">
                          <button
                            onClick={() => navigate(`/nodes/${edge.source}`)}
                            className="text-blue-400 hover:text-blue-300 truncate max-w-xs block"
                          >
                            {getNodeContent(edge.source)}
                          </button>
                        </td>
                        <td className="px-6 py-4">
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-500/20 text-purple-400">
                            {edge.relation}
                          </span>
                        </td>
                        <td className="px-6 py-4">
                          <button
                            onClick={() => navigate(`/nodes/${edge.target}`)}
                            className="text-blue-400 hover:text-blue-300 truncate max-w-xs block"
                          >
                            {getNodeContent(edge.target)}
                          </button>
                        </td>
                        <td className="px-6 py-4">
                          <div className="flex items-center gap-2">
                            <div className="w-16 h-2 bg-gray-700 rounded-full overflow-hidden">
                              <div
                                className="h-full bg-[#e94560]"
                                style={{ width: `${edge.weight * 100}%` }}
                              />
                            </div>
                            <span className="text-sm text-gray-400">
                              {edge.weight.toFixed(2)}
                            </span>
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-400">
                          {new Date(edge.created_at).toLocaleDateString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-400">
                Showing {(page - 1) * ITEMS_PER_PAGE + 1} to{' '}
                {Math.min(page * ITEMS_PER_PAGE, filteredEdges.length)} of{' '}
                {filteredEdges.length} edges
              </p>
              <div className="flex gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={page === 1}
                  onClick={() => setPage(page - 1)}
                >
                  Previous
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={page === totalPages}
                  onClick={() => setPage(page + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          )}
        </>
      )}

      {/* Create Modal */}
      <Modal
        isOpen={createModalOpen}
        onClose={() => setCreateModalOpen(false)}
        title="Create Edge"
        footer={
          <>
            <Button variant="ghost" onClick={() => setCreateModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} isLoading={isCreating}>
              Create
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Source Node
            </label>
            <SelectInput
              value={sourceId}
              onChange={(e) => setSourceId(e.target.value)}
              fullWidth
            >
              <option value="">Select a node...</option>
              {nodes.map((node) => (
                <option key={node.id} value={node.id}>
                  [{node.node_type}] {node.content.slice(0, 50)}
                </option>
              ))}
            </SelectInput>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Target Node
            </label>
            <SelectInput
              value={targetId}
              onChange={(e) => setTargetId(e.target.value)}
              fullWidth
            >
              <option value="">Select a node...</option>
              {nodes.map((node) => (
                <option key={node.id} value={node.id}>
                  [{node.node_type}] {node.content.slice(0, 50)}
                </option>
              ))}
            </SelectInput>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Relation
            </label>
            <SelectInput
              value={relation}
              onChange={(e) => setRelation(e.target.value as Relation)}
              fullWidth
            >
              {RELATION_TYPES.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </SelectInput>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Weight: {weight.toFixed(2)}
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={weight}
              onChange={(e) => setWeight(parseFloat(e.target.value))}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>0.0</span>
              <span>0.5</span>
              <span>1.0</span>
            </div>
          </div>
        </div>
      </Modal>
    </div>
  );
}
