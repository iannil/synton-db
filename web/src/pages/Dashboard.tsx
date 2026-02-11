/**
 * Dashboard page with statistics and quick actions.
 */

import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { DatabaseStats, HealthResponse, Node } from '@/types/api';
import { StatCard, QuickAction } from '@/components/ui';
import { Card, CardContent, Badge, Skeleton } from '@/components/ui';
import { cn } from '@/lib/utils';

const NODE_TYPE_ICONS: Record<string, string> = {
  entity: '🏢',
  concept: '💡',
  fact: '✓',
  raw_chunk: '📄',
};

const NODE_TYPE_COLORS: Record<string, string> = {
  entity: 'bg-blue-500/20 text-blue-400',
  concept: 'bg-purple-500/20 text-purple-400',
  fact: 'bg-green-500/20 text-green-400',
  raw_chunk: 'bg-gray-500/20 text-gray-400',
};

export function Dashboard(): JSX.Element {
  const navigate = useNavigate();
  const [stats, setStats] = useState<DatabaseStats | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [recentNodes, setRecentNodes] = useState<Node[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadData = async () => {
      try {
        setIsLoading(true);
        const [statsData, healthData, nodesData] = await Promise.all([
          api.stats(),
          api.health(),
          api.getAllNodes(),
        ]);

        setStats(statsData);
        setHealth(healthData);

        // Get 5 most recent nodes
        const sortedNodes = nodesData
          .sort((a, b) => {
            const dateA = new Date(a.meta.created_at);
            const dateB = new Date(b.meta.created_at);
            return dateB.getTime() - dateA.getTime();
          })
          .slice(0, 5);

        setRecentNodes(sortedNodes);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Skeleton className="h-64 w-96" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-destructive text-lg">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Title */}
      <div>
        <h1 className="text-2xl font-bold text-white">Dashboard</h1>
        <p className="text-gray-400 mt-1">
          Welcome to SYNTON-DB Cognitive Database
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Nodes"
          value={stats?.node_count ?? 0}
          icon="🔷"
          color="blue"
          onClick={() => navigate('/nodes')}
        />
        <StatCard
          title="Total Edges"
          value={stats?.edge_count ?? 0}
          icon="🔗"
          color="purple"
          onClick={() => navigate('/edges')}
        />
        <StatCard
          title="Embedded Nodes"
          value={stats?.embedded_count ?? 0}
          icon="🧠"
          color="green"
        />
        <StatCard
          title="System Status"
          value={health?.status === 'healthy' ? 'Healthy' : 'Warning'}
          icon={health?.status === 'healthy' ? '✅' : '⚠️'}
          color={health?.status === 'healthy' ? 'green' : 'orange'}
        />
      </div>

      {/* Memory Stats */}
      {stats?.memory_stats && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            title="Active Nodes"
            value={stats.memory_stats.active_nodes}
            icon="⚡"
            color="green"
          />
          <StatCard
            title="Decayed Nodes"
            value={stats.memory_stats.decayed_nodes}
            icon="📉"
            color="orange"
          />
          <StatCard
            title="Avg Access Score"
            value={stats.memory_stats.average_score.toFixed(2)}
            icon="📊"
            color="blue"
          />
          <StatCard
            title="Memory Load"
            value={`${(stats.memory_stats.load_factor * 100).toFixed(1)}%`}
            icon="💾"
            color="purple"
          />
        </div>
      )}

      {/* Quick Actions */}
      <Card>
        <CardContent className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">Quick Actions</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            <QuickAction
              label="Add Node"
              icon="➕"
              color="blue"
              onClick={() => navigate('/nodes?action=create')}
            />
            <QuickAction
              label="Add Edge"
              icon="🔗"
              color="purple"
              onClick={() => navigate('/edges?action=create')}
            />
            <QuickAction
              label="Query"
              icon="🔍"
              color="green"
              onClick={() => navigate('/query')}
            />
            <QuickAction
              label="Graph View"
              icon="🕸️"
              color="orange"
              onClick={() => navigate('/graph')}
            />
          </div>
        </CardContent>
      </Card>

      {/* Recent Nodes */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Recent Nodes</h2>
            <button
              onClick={() => navigate('/nodes')}
              className="text-sm text-primary hover:underline"
            >
              View All →
            </button>
          </div>

          {recentNodes.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No nodes yet. Create your first node!</p>
          ) : (
            <div className="space-y-3">
              {recentNodes.map((node) => (
                <div
                  key={node.id}
                  className="flex items-center gap-4 p-3 rounded-lg bg-muted/50 hover:bg-muted/80 transition-colors cursor-pointer"
                  onClick={() => navigate(`/nodes/${node.id}`)}
                >
                  <div className={cn(
                    'w-10 h-10 rounded-full flex items-center justify-center',
                    NODE_TYPE_COLORS[node.node_type] || NODE_TYPE_COLORS.raw_chunk
                  )}>
                    <span className="text-lg">
                      {NODE_TYPE_ICONS[node.node_type] || '📄'}
                    </span>
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-white font-medium truncate">{node.content}</p>
                    <p className="text-sm text-gray-500">
                      {new Date(node.meta.created_at).toLocaleString()}
                    </p>
                  </div>
                  <Badge variant="secondary" className="text-xs">
                    {node.node_type}
                  </Badge>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
