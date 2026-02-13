import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Routes, Route, useLocation } from 'react-router-dom';
import './index.css';
import './App.css';

// Components
import { Header, Sidebar } from '@/components/layout';
import { Toaster } from '@/components/ui';

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
      <Toaster />
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
