import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Routes, Route, useLocation, Link } from 'react-router-dom';
import './index.css';

// Components
import { Header } from '@/components/layout';

// Pages
import { Dashboard } from '@/pages/Dashboard';
import { Nodes } from '@/pages/Nodes';
import { NodeDetail } from '@/pages/NodeDetail';
import { Edges } from '@/pages/Edges';
import { Graph } from '@/pages/Graph';
import { Query } from '@/pages/Query';
import { Traverse } from '@/pages/Traverse';

// Page title mapping
const PAGE_TITLES: Record<string, string> = {
  '/': 'Dashboard',
  '/nodes': 'Nodes',
  '/edges': 'Edges',
  '/graph': 'Graph Visualization',
  '/query': 'Query',
  '/traverse': 'Graph Traversal',
};

function getTitle(pathname: string): string {
  if (PAGE_TITLES[pathname]) {
    return PAGE_TITLES[pathname];
  }
  if (pathname.startsWith('/nodes/')) {
    return 'Node Details';
  }
  return 'Dashboard';
}

function Sidebar(): JSX.Element {
  const location = useLocation();

  const navItems = [
    { path: '/', label: 'Dashboard', icon: '📊' },
    { path: '/nodes', label: 'Nodes', icon: '🔷' },
    { path: '/edges', label: 'Edges', icon: '🔗' },
    { path: '/graph', label: 'Graph', icon: '🕸️' },
    { path: '/query', label: 'Query', icon: '🔍' },
    { path: '/traverse', label: 'Traverse', icon: '🚶' },
  ];

  return (
    <aside className="sidebar">
      <div className="p-6 border-b border-white/10">
        <h1 className="text-xl font-bold text-white flex items-center gap-2">
          <span className="text-2xl">🧠</span>
          <span>SYNTON-DB</span>
        </h1>
        <p className="text-xs text-gray-400 mt-1">Cognitive Database</p>
      </div>

      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path ||
            (item.path !== '/' && location.pathname.startsWith(item.path));

          return (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                isActive
                  ? 'bg-[#e94560] text-white font-medium'
                  : 'text-gray-300 hover:bg-white/5 hover:text-white'
              }`}
            >
              <span className="text-lg">{item.icon}</span>
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      <div className="p-4 border-t border-white/10">
        <p className="text-xs text-gray-500 text-center">
          © 2025 SYNTON-DB Team
        </p>
      </div>
    </aside>
  );
}

function AppContent(): JSX.Element {
  const location = useLocation();
  const title = getTitle(location.pathname);

  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        <Header title={title} key={location.pathname} />
        <main className="main-scroll">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/nodes" element={<Nodes />} />
            <Route path="/nodes/:id" element={<NodeDetail />} />
            <Route path="/edges" element={<Edges />} />
            <Route path="/graph" element={<Graph />} />
            <Route path="/query" element={<Query />} />
            <Route path="/traverse" element={<Traverse />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  </StrictMode>
);
