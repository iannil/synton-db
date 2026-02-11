/**
 * Nodes list page with filtering and pagination.
 */

import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { api } from '@/services/api';
import type { Node, NodeType } from '@/types/api';
import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Card,
  CardContent,
  Badge,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Alert,
  AlertDescription,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Label,
  Textarea,
  Skeleton,
} from '@/components/ui';
import { cn } from '@/lib/utils';
import { Trash2, Eye } from 'lucide-react';

const NODE_TYPES: Array<{ value: NodeType; label: string }> = [
  { value: 'entity', label: 'Entity' },
  { value: 'concept', label: 'Concept' },
  { value: 'fact', label: 'Fact' },
  { value: 'raw_chunk', label: 'Raw Chunk' },
];

const ITEMS_PER_PAGE = 20;

const NODE_TYPE_BADGE_COLORS: Record<string, string> = {
  entity: 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30',
  concept: 'bg-purple-500/20 text-purple-400 hover:bg-purple-500/30',
  fact: 'bg-green-500/20 text-green-400 hover:bg-green-500/30',
  raw_chunk: 'bg-gray-500/20 text-gray-400 hover:bg-gray-500/30',
};

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

  // Dialogs
  const [createDialogOpen, setCreateDialogOpen] = useState(
    searchParams.get('action') === 'create'
  );
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
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
      setDeleteDialogOpen(false);
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
      setCreateDialogOpen(false);
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
        <Button onClick={() => setCreateDialogOpen(true)}>+ Add Node</Button>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-[200px]">
              <Input
                placeholder="Search nodes..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            <div className="w-48">
              <Select value={typeFilter} onValueChange={setTypeFilter}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Types</SelectItem>
                  {NODE_TYPES.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Error */}
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Loading */}
      {isLoading ? (
        <div className="space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <Skeleton key={i} className="h-16 w-full" />
          ))}
        </div>
      ) : (
        <>
          {/* Nodes List */}
          {paginatedNodes.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <p className="text-gray-500 text-lg">
                  {search || typeFilter !== 'all'
                    ? 'No nodes match your filters.'
                    : 'No nodes yet. Create your first node!'}
                </p>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Content</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead>Confidence</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {paginatedNodes.map((node) => (
                    <TableRow
                      key={node.id}
                      className="cursor-pointer"
                      onClick={() => navigate(`/nodes/${node.id}`)}
                    >
                      <TableCell>
                        <div className="max-w-md">
                          <p className="text-white line-clamp-2">{node.content}</p>
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge
                          className={NODE_TYPE_BADGE_COLORS[node.node_type] || NODE_TYPE_BADGE_COLORS.raw_chunk}
                        >
                          {node.node_type}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-gray-400">
                        {new Date(node.meta.created_at).toLocaleDateString()}
                      </TableCell>
                      <TableCell className="text-gray-400">
                        {(node.meta.confidence * 100).toFixed(0)}%
                      </TableCell>
                      <TableCell className="text-right" onClick={(e) => e.stopPropagation()}>
                        <div className="flex justify-end gap-2">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => navigate(`/nodes/${node.id}`)}
                          >
                            <Eye className="w-4 h-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setNodeToDelete(node);
                              setDeleteDialogOpen(true);
                            }}
                          >
                            <Trash2 className="w-4 h-4 text-destructive" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </Card>
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
                  variant="outline"
                  size="sm"
                  disabled={page === 1}
                  onClick={() => setPage(page - 1)}
                >
                  Previous
                </Button>
                <Button
                  variant="outline"
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

      {/* Create Dialog */}
      <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Node</DialogTitle>
            <DialogDescription>
              Add a new node to the knowledge graph.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="content">Content</Label>
              <Textarea
                id="content"
                value={newNodeContent}
                onChange={(e) => setNewNodeContent(e.target.value)}
                placeholder="Enter node content..."
                rows={4}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="type">Type</Label>
              <Select value={newNodeType} onValueChange={(value) => setNewNodeType(value as NodeType)}>
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
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={isCreating}>
              {isCreating ? 'Creating...' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <Dialog open={deleteDialogOpen} onOpenChange={(open) => {
        setDeleteDialogOpen(open);
        if (!open) setNodeToDelete(null);
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Node</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this node? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setDeleteDialogOpen(false);
              setNodeToDelete(null);
            }}>
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
