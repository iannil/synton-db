/**
 * Stat card component for dashboard metrics.
 */

interface StatCardProps {
  title: string;
  value: string | number;
  icon: string;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  color?: 'blue' | 'purple' | 'green' | 'orange' | 'red';
  onClick?: () => void;
}

const colorStyles = {
  blue: 'from-blue-500/20 to-blue-600/10 border-blue-500/30',
  purple: 'from-purple-500/20 to-purple-600/10 border-purple-500/30',
  green: 'from-green-500/20 to-green-600/10 border-green-500/30',
  orange: 'from-orange-500/20 to-orange-600/10 border-orange-500/30',
  red: 'from-red-500/20 to-red-600/10 border-red-500/30',
};

export function StatCard({
  title,
  value,
  icon,
  trend,
  color = 'blue',
  onClick,
}: StatCardProps): JSX.Element {
  return (
    <div
      className={clsx(
        'card card-hover relative overflow-hidden bg-gradient-to-br',
        colorStyles[color],
        onClick && 'cursor-pointer'
      )}
      onClick={onClick}
    >
      {/* Background decoration */}
      <div className="absolute -right-4 -top-4 text-8xl opacity-10">{icon}</div>

      {/* Content */}
      <div className="relative">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm font-medium text-gray-400 uppercase tracking-wide">
              {title}
            </p>
            <p className="mt-2 text-3xl font-bold text-white">{value}</p>
          </div>
          <div className="text-3xl">{icon}</div>
        </div>

        {trend && (
          <div className="mt-4 flex items-center gap-2 text-sm">
            <span
              className={clsx(
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
    </div>
  );
}

interface QuickActionProps {
  label: string;
  icon: string;
  onClick: () => void;
  color?: 'blue' | 'purple' | 'green' | 'orange' | 'red';
}

export function QuickAction({
  label,
  icon,
  onClick,
  color = 'blue',
}: QuickActionProps): JSX.Element {
  return (
    <button
      onClick={onClick}
      className={clsx(
        'flex flex-col items-center gap-2 p-4 rounded-xl bg-gradient-to-br border border-white/10',
        'hover:scale-105 transition-transform',
        colorStyles[color]
      )}
    >
      <span className="text-3xl">{icon}</span>
      <span className="text-sm font-medium text-white">{label}</span>
    </button>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
