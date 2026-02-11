/**
 * Sidebar navigation component.
 */

import { Link, useLocation } from 'react-router-dom';
import { clsx } from 'clsx';

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { path: '/', label: 'Dashboard', icon: '📊' },
  { path: '/nodes', label: 'Nodes', icon: '🔷' },
  { path: '/edges', label: 'Edges', icon: '🔗' },
  { path: '/graph', label: 'Graph', icon: '🕸️' },
  { path: '/query', label: 'Query', icon: '🔍' },
  { path: '/traverse', label: 'Traverse', icon: '🚶' },
];

export function Sidebar(): JSX.Element {
  const location = useLocation();

  return (
    <aside className="sidebar">
      {/* Logo */}
      <div className="p-6 border-b border-white/10">
        <h1 className="text-xl font-bold text-white flex items-center gap-2">
          <span className="text-2xl">🧠</span>
          <span>SYNTON-DB</span>
        </h1>
        <p className="text-xs text-gray-400 mt-1">Cognitive Database</p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path ||
            (item.path !== '/' && location.pathname.startsWith(item.path));

          return (
            <Link
              key={item.path}
              to={item.path}
              className={clsx(
                'flex items-center gap-3 px-4 py-3 rounded-lg transition-colors',
                isActive
                  ? 'bg-[#e94560] text-white font-medium'
                  : 'text-gray-300 hover:bg-white/5 hover:text-white'
              )}
            >
              <span className="text-lg">{item.icon}</span>
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-white/10">
        <p className="text-xs text-gray-500 text-center">
          © 2025 SYNTON-DB Team
        </p>
      </div>
    </aside>
  );
}
