/**
 * Edges management page with filtering and creation.
 */

import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { api } from '@/services/api';
import type { Edge, Node, Relation } from '@/types/api';
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
  Slider,
  Skeleton,
  Badge,
} from '@/components/ui';
import { cn } from '@/lib/utils';

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
  const [createDialogOpen, setCreateDialogOpen] = useState(
    searchParams.get('action') === 'create'
  );
  const [sourceId, setSourceId] = useState('');
  const [targetId, setTargetId] = useState('');
  const [relation, setRelation] = useState<Relation>('similar_to');
  const [weight, setWeight] = useState<number[]>([1.0]);
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
              direction: 'Forward',
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
        weight: weight[0],
      });

      setEdges([...edges, response.edge]);
      setCreateDialogOpen(false);
      setSourceId('');
      setTargetId('');
      setRelation('similar_to');
      setWeight([1.0]);
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
        <Button onClick={() => setCreateDialogOpen(true)}>+ Add Edge</Button>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap gap-4">
            <div className="w-48">
              <Select value={relationFilter} onValueChange={setRelationFilter}>
                <SelectTrigger>
                  <SelectValue placeholder="All Relations" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Relations</SelectItem>
                  {RELATION_TYPES.map((r) => (
                    <SelectItem key={r} value={r}>
                      {r}
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
          {/* Edges List */}
          {paginatedEdges.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <p className="text-gray-500 text-lg">
                  {relationFilter !== 'all'
                    ? 'No edges match your filters.'
                    : 'No edges yet. Create your first edge!'}
                </p>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Source</TableHead>
                    <TableHead>Relation</TableHead>
                    <TableHead>Target</TableHead>
                    <TableHead>Weight</TableHead>
                    <TableHead>Created</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {paginatedEdges.map((edge, index) => (
                    <TableRow
                      key={`${edge.source}-${edge.target}-${edge.relation}-${index}`}
                    >
                      <TableCell>
                        <button
                          onClick={() => navigate(`/nodes/${edge.source}`)}
                          className="text-blue-400 hover:text-blue-300 truncate max-w-xs block"
                        >
                          {getNodeContent(edge.source)}
                        </button>
                      </TableCell>
                      <TableCell>
                        <Badge variant="secondary" className="bg-purple-500/20 text-purple-400 hover:bg-purple-500/30">
                          {edge.relation}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <button
                          onClick={() => navigate(`/nodes/${edge.target}`)}
                          className="text-blue-400 hover:text-blue-300 truncate max-w-xs block"
                        >
                          {getNodeContent(edge.target)}
                        </button>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <div className="w-16 h-2 bg-secondary rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary"
                              style={{ width: `${edge.weight * 100}%` }}
                            />
                          </div>
                          <span className="text-sm text-gray-400">
                            {edge.weight.toFixed(2)}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell className="text-gray-400">
                        {new Date(edge.created_at).toLocaleDateString()}
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
                {Math.min(page * ITEMS_PER_PAGE, filteredEdges.length)} of{' '}
                {filteredEdges.length} edges
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
            <DialogTitle>Create Edge</DialogTitle>
            <DialogDescription>
              Create a new relationship between two nodes.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="source">Source Node</Label>
              <select
                id="source"
                value={sourceId}
                onChange={(e) => setSourceId(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <option value="">Select a node...</option>
                {nodes.map((node) => (
                  <option key={node.id} value={node.id}>
                    [{node.node_type}] {node.content.slice(0, 50)}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="target">Target Node</Label>
              <select
                id="target"
                value={targetId}
                onChange={(e) => setTargetId(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <option value="">Select a node...</option>
                {nodes.map((node) => (
                  <option key={node.id} value={node.id}>
                    [{node.node_type}] {node.content.slice(0, 50)}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="relation">Relation</Label>
              <Select value={relation} onValueChange={(value) => setRelation(value as Relation)}>
                <SelectTrigger id="relation">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {RELATION_TYPES.map((r) => (
                    <SelectItem key={r} value={r}>
                      {r}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>Weight: {weight[0].toFixed(2)}</Label>
              <Slider
                value={weight}
                onValueChange={setWeight}
                min={0}
                max={1}
                step={0.01}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-gray-500">
                <span>0.0</span>
                <span>0.5</span>
                <span>1.0</span>
              </div>
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
    </div>
  );
}
