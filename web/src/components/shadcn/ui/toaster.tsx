/**
 * Toaster component using Sonner.
 *
 * Add this component to your app root to enable toast notifications.
 */

import { Toaster as SonnerToaster } from 'sonner';

export function Toaster() {
  return (
    <SonnerToaster
      position="top-right"
      toastOptions={{
        className: 'bg-[#1e293b] border border-white/10 text-gray-200',
        descriptionClassName: 'text-gray-300',
        actionButtonStyle: {
          backgroundColor: '#e94560',
          color: 'white',
        },
      }}
    />
  );
}
