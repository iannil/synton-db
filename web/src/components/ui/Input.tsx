/**
 * Input component with label and error handling.
 */

import { InputHTMLAttributes, forwardRef } from 'react';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  fullWidth?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, fullWidth = false, className = '', ...props }, ref) => {
    return (
      <div className={clsx('flex flex-col gap-1.5', fullWidth && 'w-full')}>
        {label && (
          <label className="text-sm font-medium text-gray-300">{label}</label>
        )}
        <input
          ref={ref}
          className={clsx(
            'px-4 py-2 rounded-lg bg-[#0f3460] border border-white/10 text-white placeholder-gray-500',
            'focus:outline-none focus:ring-2 focus:ring-[#e94560] focus:border-transparent',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            error && 'border-red-500 focus:ring-red-500',
            fullWidth && 'w-full',
            className
          )}
          {...props}
        />
        {error && <p className="text-sm text-red-400">{error}</p>}
      </div>
    );
  }
);

Input.displayName = 'Input';

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  fullWidth?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ label, error, fullWidth = false, className = '', ...props }, ref) => {
    return (
      <div className={clsx('flex flex-col gap-1.5', fullWidth && 'w-full')}>
        {label && (
          <label className="text-sm font-medium text-gray-300">{label}</label>
        )}
        <textarea
          ref={ref}
          className={clsx(
            'px-4 py-2 rounded-lg bg-[#0f3460] border border-white/10 text-white placeholder-gray-500',
            'focus:outline-none focus:ring-2 focus:ring-[#e94560] focus:border-transparent',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            'resize-vertical',
            error && 'border-red-500 focus:ring-red-500',
            fullWidth && 'w-full',
            className
          )}
          {...props}
        />
        {error && <p className="text-sm text-red-400">{error}</p>}
      </div>
    );
  }
);

Textarea.displayName = 'Textarea';

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
