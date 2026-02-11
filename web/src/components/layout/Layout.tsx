/**
 * Main layout component with sidebar and content area.
 */

import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { Header } from './Header';

export function Layout(): JSX.Element {
  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-content">
        <Header title="Dashboard" />
        <main className="main-scroll">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
