/**
 * Header component with title and actions.
 */

import { useState, useEffect } from 'react';

interface HeaderProps {
  title: string;
  actions?: React.ReactNode;
}

export function Header({ title, actions }: HeaderProps): JSX.Element {
  const [status, setStatus] = useState<'online' | 'offline' | 'checking'>('checking');
  const [time, setTime] = useState(new Date());

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

  return (
    <header className="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-[#16213e]">
      <div className="flex items-center gap-4">
        <h2 className="text-xl font-semibold text-white">{title}</h2>
      </div>

      <div className="flex items-center gap-4">
        {/* API Status */}
        <div className="flex items-center gap-2">
          <span
            className={clsx(
              'w-2 h-2 rounded-full',
              status === 'online' && 'bg-green-500',
              status === 'offline' && 'bg-red-500',
              status === 'checking' && 'bg-yellow-500 animate-pulse'
            )}
          />
          <span className="text-sm text-gray-400 capitalize">{status}</span>
        </div>

        {/* Time */}
        <div className="text-sm text-gray-400">
          {time.toLocaleTimeString()}
        </div>

        {/* Actions */}
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </header>
  );
}

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
