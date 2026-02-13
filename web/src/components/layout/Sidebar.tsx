/**
 * Simple collapsible sidebar for SYNTON-DB.
 */

import { useState, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import {
  Menu,
  X,
  BarChart3,
  Box,
  Link as LinkIcon,
  Network,
  Search,
  Route,
  Brain,
} from 'lucide-react';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui';

interface NavItem {
  path: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}

const navItems: NavItem[] = [
  { path: '/', label: 'Dashboard', icon: BarChart3 },
  { path: '/nodes', label: 'Nodes', icon: Box },
  { path: '/edges', label: 'Edges', icon: LinkIcon },
  { path: '/graph', label: 'Graph', icon: Network },
  { path: '/query', label: 'Query', icon: Search },
  { path: '/traverse', label: 'Traverse', icon: Route },
];

const SIDEBAR_STORAGE_KEY = 'syntondb_sidebar_collapsed';

export function Sidebar(): JSX.Element {
  const location = useLocation();
  const [isCollapsed, setIsCollapsed] = useState(() => {
    const stored = localStorage.getItem(SIDEBAR_STORAGE_KEY);
    return stored === 'true';
  });
  const [isMobileOpen, setIsMobileOpen] = useState(false);

  useEffect(() => {
    localStorage.setItem(SIDEBAR_STORAGE_KEY, isCollapsed.toString());
  }, [isCollapsed]);

  const toggleSidebar = () => setIsCollapsed(!isCollapsed);

  const NavItem = ({ item }: { item: NavItem }) => {
    const isActive = location.pathname === item.path ||
      (item.path !== '/' && location.pathname.startsWith(item.path));
    const Icon = item.icon;

    return (
      <TooltipProvider delayDuration={isCollapsed ? 0 : 1000}>
        <Tooltip>
          <TooltipTrigger asChild>
            <Link
              to={item.path}
              className={cn(
                'flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200',
                isActive
                  ? 'bg-[#e94560] text-white font-medium shadow-md'
                  : 'text-gray-300 hover:bg-white/5 hover:text-white',
                !isCollapsed && 'w-full justify-start'
              )}
            >
              <Icon className="w-5 h-5 flex-shrink-0" />
              {(!isCollapsed || isMobileOpen) && (
                <span className="font-medium">{item.label}</span>
              )}
            </Link>
          </TooltipTrigger>
          {isCollapsed && (
            <TooltipContent side="right">
              <p>{item.label}</p>
            </TooltipContent>
          )}
        </Tooltip>
      </TooltipProvider>
    );
  };

  return (
    <>
      <aside
        className={cn(
          'hidden md:flex flex-col border-r border-white/10 bg-[var(--color-surface)] transition-all duration-300',
          isCollapsed ? 'w-16' : 'w-[260px]'
        )}
      >
        <TooltipProvider delayDuration={isCollapsed ? 0 : 1000}>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={toggleSidebar}
                className={cn(
                  'absolute -right-3 top-4 z-10 flex h-8 w-8 items-center justify-center rounded-lg transition-all',
                  isCollapsed
                    ? 'bg-[#e94560] text-white'
                    : 'bg-white/10 text-gray-400 hover:bg-white/20'
                )}
              >
                {isCollapsed ? <Menu className="h-5 w-5" /> : <X className="h-5 w-5" />}
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">
              <p>{isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        <div
          className={cn(
            'p-6 border-b border-white/10 transition-opacity duration-200',
            isCollapsed && 'opacity-0'
          )}
        >
          <h1
            className={cn(
              'text-xl font-bold flex items-center gap-2 transition-all duration-200',
              isCollapsed ? 'justify-center' : ''
            )}
          >
            <Brain className="w-6 h-6 text-[#e94560]" />
            {(!isCollapsed || isMobileOpen) && (
              <span className="text-sm text-gray-400 ml-1">SYNTON-DB</span>
            )}
          </h1>
          {(!isCollapsed || isMobileOpen) && (
            <p className="text-xs text-gray-400 mt-1">Cognitive Database</p>
          )}
        </div>

        <nav
          className={cn(
            'flex-1 p-4 space-y-1 transition-all duration-200 overflow-hidden',
            isCollapsed && 'p-2'
          )}
        >
          {navItems.map((item) => (
            <NavItem key={item.path} item={item} />
          ))}
        </nav>

        <div
          className={cn(
            'p-4 border-t border-white/10 transition-opacity duration-200',
            isCollapsed && 'opacity-0'
          )}
        >
          {(!isCollapsed || isMobileOpen) && (
            <p className="text-xs text-gray-500 text-center">
              © 2025 SYNTON-DB Team
            </p>
          )}
        </div>
      </aside>

      {/* Mobile Menu Button - Using Dialog directly for simplicity */}
      <div className="md:hidden fixed bottom-4 right-4 z-40">
        <button
          onClick={() => setIsMobileOpen(true)}
          className="flex h-12 w-12 items-center justify-center rounded-full bg-[#e94560] text-white shadow-lg hover:scale-105"
        >
          <Menu className="h-6 w-6" />
        </button>
      </div>

      {/* Mobile Menu Dialog - Simplified version */}
      {isMobileOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/80 flex flex-col h-full"
          onClick={() => setIsMobileOpen(false)}
        >
          <div className="flex items-center justify-between p-6 border-b border-white/10">
            <h1 className="text-xl font-bold text-white flex items-center gap-2">
              <Brain className="w-6 h-6 text-[#e94560]" />
              <span className="text-sm text-gray-400">SYNTON-DB</span>
            </h1>
            <button
              onClick={() => setIsMobileOpen(false)}
              className="text-gray-400 hover:text-white"
            >
              <X className="h-5 w-5" />
            </button>
          </div>

          <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
            {navItems.map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  className="flex items-center gap-3 px-4 py-3 rounded-lg text-gray-300 hover:bg-white/5 hover:text-white"
                  onClick={() => setIsMobileOpen(false)}
                >
                  <Icon className="w-5 h-5" />
                  <span className="font-medium">{item.label}</span>
                </Link>
              );
            })}
          </nav>

          <div className="p-4 border-t border-white/10">
            <p className="text-xs text-gray-500 text-center">
              © 2025 SYNTON-DB Team
            </p>
          </div>
        </div>
      )}
    </>
  );
}
