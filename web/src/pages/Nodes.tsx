/**
 * Nodes list page with filtering and pagination.
 */

import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, NodeType } from '@/types/api';
import { Button, Input, Select, Modal, ConfirmModal } from '@/components/ui';

const NODE_TYPES: Array<{ value: NodeType; label: string }> = [
  { value: 'entity', label: 'Entity' },
  { value: 'concept', label: 'Concept' },
  { value: 'fact', label: 'Fact' },
  { value: 'raw_chunk', label: 'Raw Chunk' },
];

const ITEMS_PER_PAGE = 20;

export function Nodes(): JSX.Element {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const [nodes, setNodes] = useState<Node[]>([]);
  const [filteredNodes, setFilteredNodes] = useState<Node[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<string>('all');
  const [page, setPage] = useState(1);

  // Modals
  const [createModalOpen, setCreateModalOpen] = useState(
    searchParams.get('action') === 'create'
  );
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [nodeToDelete, setNodeToDelete] = useState<Node | null>(null);

  // New node form
  const [newNodeContent, setNewNodeContent] = useState('');
  const [newNodeType, setNewNodeType] = useState<NodeType>('concept');
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    const loadNodes = async () => {
      try {
        setIsLoading(true);
        const data = await api.getAllNodes();
        setNodes(data);
        setFilteredNodes(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load nodes');
      } finally {
        setIsLoading(false);
      }
    };

    loadNodes();
  }, []);

  useEffect(() => {
    let filtered = nodes;

    // Apply type filter
    if (typeFilter !== 'all') {
      filtered = filtered.filter((n) => n.node_type === typeFilter);
    }

    // Apply search filter
    if (search) {
      const searchLower = search.toLowerCase();
      filtered = filtered.filter((n) =>
        n.content.toLowerCase().includes(searchLower)
      );
    }

    setFilteredNodes(filtered);
    setPage(1);
  }, [nodes, typeFilter, search]);

  const handleDelete = async () => {
    if (!nodeToDelete) return;

    try {
      await api.deleteNode({ id: nodeToDelete.id });
      setNodes(nodes.filter((n) => n.id !== nodeToDelete.id));
      setDeleteModalOpen(false);
      setNodeToDelete(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete node');
    }
  };

  const handleCreate = async () => {
    if (!newNodeContent.trim()) return;

    try {
      setIsCreating(true);
      const response = await api.addNode({
        content: newNodeContent,
        node_type: newNodeType,
      });

      setNodes([response.node, ...nodes]);
      setCreateModalOpen(false);
      setNewNodeContent('');
      setNewNodeType('concept');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create node');
    } finally {
      setIsCreating(false);
    }
  };

  const paginatedNodes = filteredNodes.slice(
    (page - 1) * ITEMS_PER_PAGE,
    page * ITEMS_PER_PAGE
  );

  const totalPages = Math.ceil(filteredNodes.length / ITEMS_PER_PAGE);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Nodes</h1>
          <p className="text-gray-400 mt-1">
            {filteredNodes.length} {filteredNodes.length === 1 ? 'node' : 'nodes'}
          </p>
        </div>
        <Button onClick={() => setCreateModalOpen(true)}>+ Add Node</Button>
      </div>

      {/* Filters */}
      <div className="card">
        <div className="flex flex-wrap gap-4">
          <div className="flex-1 min-w-[200px]">
            <Input
              placeholder="Search nodes..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              fullWidth
            />
          </div>
          <div className="w-48">
            <Select
              options={[
                { value: 'all', label: 'All Types' },
                ...NODE_TYPES,
              ]}
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value)}
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
          {/* Nodes List */}
          {paginatedNodes.length === 0 ? (
            <div className="card text-center py-12">
              <p className="text-gray-500 text-lg">
                {search || typeFilter !== 'all'
                  ? 'No nodes match your filters.'
                  : 'No nodes yet. Create your first node!'}
              </p>
            </div>
          ) : (
            <div className="card overflow-hidden p-0">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-white/5">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Content
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Type
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Created
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Confidence
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/10">
                    {paginatedNodes.map((node) => (
                      <tr
                        key={node.id}
                        className="hover:bg-white/5 transition-colors cursor-pointer"
                        onClick={() => navigate(`/nodes/${node.id}`)}
                      >
                        <td className="px-6 py-4">
                          <div className="max-w-md">
                            <p className="text-white line-clamp-2">{node.content}</p>
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={clsx('badge', `badge-${node.node_type}`)}>
                            {node.node_type}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-400">
                          {new Date(node.meta.created_at).toLocaleDateString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-400">
                          {(node.meta.confidence * 100).toFixed(0)}%
                        </td>
                        <td
                          className="px-6 py-4 whitespace-nowrap text-right text-sm"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <button
                            onClick={() => navigate(`/nodes/${node.id}`)}
                            className="text-blue-400 hover:text-blue-300 mr-3"
                          >
                            View
                          </button>
                          <button
                            onClick={() => {
                              setNodeToDelete(node);
                              setDeleteModalOpen(true);
                            }}
                            className="text-red-400 hover:text-red-300"
                          >
                            Delete
                          </button>
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
                {Math.min(page * ITEMS_PER_PAGE, filteredNodes.length)} of{' '}
                {filteredNodes.length} nodes
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
        title="Create Node"
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
              Content
            </label>
            <textarea
              value={newNodeContent}
              onChange={(e) => setNewNodeContent(e.target.value)}
              placeholder="Enter node content..."
              rows={4}
              className="w-full px-4 py-2 rounded-lg bg-[#0f3460] border border-white/10 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#e94560] focus:border-transparent resize-vertical"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Type
            </label>
            <Select
              options={NODE_TYPES}
              value={newNodeType}
              onChange={(e) => setNewNodeType(e.target.value as NodeType)}
              fullWidth
            />
          </div>
        </div>
      </Modal>

      {/* Delete Confirmation */}
      <ConfirmModal
        isOpen={deleteModalOpen}
        onClose={() => {
          setDeleteModalOpen(false);
          setNodeToDelete(null);
        }}
        onConfirm={handleDelete}
        title="Delete Node"
        message={`Are you sure you want to delete this node? This action cannot be undone.`}
        confirmText="Delete"
        variant="danger"
      />
    </div>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
