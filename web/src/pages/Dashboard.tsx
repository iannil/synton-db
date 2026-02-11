/**
 * Dashboard page with statistics and quick actions.
 */

import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '@/services/api';
import type { DatabaseStats, HealthResponse, Node } from '@/types/api';
import { StatCard, QuickAction } from '@/components/ui';

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
        <div className="spinner" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-400 text-lg">{error}</p>
        </div>
      </div>
    );
  }

  const nodeTypeCounts = stats
    ? recentNodes.reduce((acc, node) => {
        acc[node.node_type] = (acc[node.node_type] || 0) + 1;
        return acc;
      }, {} as Record<string, number>)
    : {};

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
      <div className="card">
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
      </div>

      {/* Recent Nodes */}
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Recent Nodes</h2>
          <button
            onClick={() => navigate('/nodes')}
            className="text-sm text-[#e94560] hover:text-[#d63850] transition-colors"
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
                className="flex items-center gap-4 p-3 rounded-lg bg-white/5 hover:bg-white/10 transition-colors cursor-pointer"
                onClick={() => navigate(`/nodes/${node.id}`)}
              >
                <div className={clsx('w-10 h-10 rounded-full flex items-center justify-center', {
                  'bg-blue-500/20': node.node_type === 'entity',
                  'bg-purple-500/20': node.node_type === 'concept',
                  'bg-green-500/20': node.node_type === 'fact',
                  'bg-gray-500/20': node.node_type === 'raw_chunk',
                })}>
                  <span className="text-lg">
                    {node.node_type === 'entity' && '🏢'}
                    {node.node_type === 'concept' && '💡'}
                    {node.node_type === 'fact' && '✓'}
                    {node.node_type === 'raw_chunk' && '📄'}
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-white font-medium truncate">{node.content}</p>
                  <p className="text-sm text-gray-500">
                    {new Date(node.meta.created_at).toLocaleString()}
                  </p>
                </div>
                <span className={clsx('badge', `badge-${node.node_type}`)}>
                  {node.node_type}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
