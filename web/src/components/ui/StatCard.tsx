/**
 * Stat card component for dashboard metrics.
 */

import { Card } from '@/components/ui';
import { cn } from '@/lib/utils';
import type { LucideIcon } from 'lucide-react';

interface StatCardProps {
  title: string;
  value: string | number;
  icon: LucideIcon;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  color?: 'blue' | 'purple' | 'green' | 'orange' | 'red';
  onClick?: () => void;
}

const colorStyles = {
  blue: 'from-blue-500/20 to-blue-600/10 border-blue-500/30 hover:bg-blue-500/10',
  purple: 'from-purple-500/20 to-purple-600/10 border-purple-500/30 hover:bg-purple-500/10',
  green: 'from-green-500/20 to-green-600/10 border-green-500/30 hover:bg-green-500/10',
  orange: 'from-orange-500/20 to-orange-600/10 border-orange-500/30 hover:bg-orange-500/10',
  red: 'from-red-500/20 to-red-600/10 border-red-500/30 hover:bg-red-500/10',
};

export function StatCard({
  title,
  value,
  icon: Icon,
  trend,
  color = 'blue',
  onClick,
}: StatCardProps): JSX.Element {
  return (
    <Card
      className={cn(
        'relative overflow-hidden bg-gradient-to-br border transition-all',
        colorStyles[color],
        onClick && 'cursor-pointer hover:scale-[1.02]'
      )}
      onClick={onClick}
    >
      {/* Background decoration */}
      <div className="absolute -right-4 -top-4 text-8xl opacity-10">
        <Icon className="w-16 h-16" />
      </div>

      {/* Content */}
      <div className="relative p-6">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm font-medium text-gray-400 uppercase tracking-wide">
              {title}
            </p>
            <p className="mt-2 text-3xl font-bold text-white">{value}</p>
          </div>
          <div className="rounded-full bg-white/5 p-2">
            <Icon className="w-6 h-6 text-white" />
          </div>
        </div>

        {trend && (
          <div className="mt-4 flex items-center gap-2 text-sm">
            <span
              className={cn(
                'flex items-center gap-1',
                trend.isPositive ? 'text-green-400' : 'text-red-400'
              )}
            >
              {trend.isPositive ? '↑' : '↓'}
              {Math.abs(trend.value)}%
            </span>
            <span className="text-gray-500">vs last period</span>
          </div>
        )}
      </div>
    </Card>
  );
}

interface QuickActionProps {
  label: string;
  icon: LucideIcon;
  onClick: () => void;
  color?: 'blue' | 'purple' | 'green' | 'orange' | 'red';
}

export function QuickAction({
  label,
  icon: Icon,
  onClick,
  color = 'blue',
}: QuickActionProps): JSX.Element {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex flex-col items-center gap-2 p-4 rounded-xl bg-gradient-to-br border border-white/10',
        'hover:scale-105 transition-transform',
        colorStyles[color]
      )}
    >
      <div className="rounded-full bg-white/5 p-2">
        <Icon className="w-6 h-6 text-white" />
      </div>
      <span className="text-sm font-medium text-white">{label}</span>
    </button>
  );
}
