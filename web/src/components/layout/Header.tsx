/**
 * Header component with title, actions, and breadcrumb navigation.
 */

import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import { ChevronRight, Home } from 'lucide-react';

interface BreadcrumbItem {
  label: string;
  path?: string;
}

const ROUTE_BREADCRUMBS: Record<string, BreadcrumbItem[]> = {
  '/nodes': [{ label: 'Nodes', path: '/nodes' }],
  '/nodes/:id': [{ label: 'Nodes', path: '/nodes' }],
  '/edges': [{ label: 'Edges', path: '/edges' }],
  '/graph': [{ label: 'Graph', path: '/graph' }],
  '/query': [{ label: 'Query', path: '/query' }],
  '/traverse': [{ label: 'Traverse', path: '/traverse' }],
};

interface HeaderProps {
  title: string;
  actions?: React.ReactNode;
}

export function Header({ title, actions }: HeaderProps): JSX.Element {
  const [status, setStatus] = useState<'online' | 'offline' | 'checking'>('checking');
  const [time, setTime] = useState(new Date());
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    // Check API status
    const checkStatus = async () => {
      try {
        const response = await fetch('/health');
        setStatus(response.ok ? 'online' : 'offline');
      } catch {
        setStatus('offline');
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(interval);
  }, []);

  // Build breadcrumb items from current path
  const getBreadcrumbs = (): BreadcrumbItem[] => {
    const pathname = location.pathname;

    // Direct routes
    if (ROUTE_BREADCRUMBS[pathname]) {
      return ROUTE_BREADCRUMBS[pathname];
    }

    // Dynamic routes (e.g., /nodes/:id)
    if (pathname.startsWith('/nodes/')) {
      return [{ label: 'Nodes', path: '/nodes' }, { label: 'Node Details' }];
    }

    // Default to dashboard
    return [{ label: 'Dashboard', path: '/' }];
  };

  const breadcrumbs = getBreadcrumbs();

  return (
    <header className="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-[var(--color-surface)]">
      <div className="flex items-center gap-4 flex-1 min-w-0">
        {/* Breadcrumb Navigation */}
        <nav className="flex items-center gap-1" aria-label="Breadcrumb">
          <button
            onClick={() => navigate('/')}
            className="flex items-center gap-1 text-sm text-gray-400 hover:text-gray-300 transition-colors"
            title="Dashboard"
          >
            <Home className="h-4 w-4" />
          </button>

          {breadcrumbs.map((crumb, index) => (
            <div key={index} className="flex items-center gap-1">
              <ChevronRight className="h-4 w-4 text-gray-500" />
              {crumb.path ? (
                <button
                  onClick={() => navigate(crumb.path)}
                  className={cn(
                    'text-sm hover:text-gray-300 transition-colors',
                    location.pathname === crumb.path ? 'text-white font-medium' : 'text-gray-400'
                  )}
                >
                  {crumb.label}
                </button>
              ) : (
                <span className="text-sm text-white font-medium">{crumb.label}</span>
              )}
            </div>
          ))}
        </nav>

        {/* Page Title (hidden on small screens, visible in breadcrumb on large screens) */}
        <h1 className="text-xl font-semibold text-white hidden md:block">
          {title}
        </h1>
      </div>

      <div className="flex items-center gap-4">
        {/* API Status */}
        <div className="flex items-center gap-2">
          <span
            className={cn(
              'w-2 h-2 rounded-full',
              status === 'online' && 'bg-green-500',
              status === 'offline' && 'bg-red-500',
              status === 'checking' && 'bg-yellow-500 animate-pulse'
            )}
          />
          <span className="text-sm text-gray-400 capitalize hidden sm:inline">
            {status}
          </span>
        </div>

        {/* Time */}
        <div className="text-sm text-gray-400 hidden sm:block">
          {time.toLocaleTimeString()}
        </div>

        {/* Actions */}
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </header>
  );
}
