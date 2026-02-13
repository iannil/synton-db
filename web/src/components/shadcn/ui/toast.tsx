/**
 * Toast utilities using Sonner.
 *
 * Import Toaster from './toaster' and add it to your app.
 * Use toast() function from 'sonner' directly.
 */

import { toast as sonnerToast } from 'sonner';

type ToastProps = {
  message: string;
  type?: 'success' | 'error' | 'info' | 'warning';
};

export const toast = ({ message, type = 'info' }: ToastProps) => {
  switch (type) {
    case 'success':
      sonnerToast.success(message);
      break;
    case 'error':
      sonnerToast.error(message);
      break;
    case 'warning':
      sonnerToast.warning(message);
      break;
    default:
      sonnerToast(message);
  }
};

export { sonnerToast as toastFn };
