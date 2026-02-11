/**
 * Select component with label and error handling.
 */

import { SelectHTMLAttributes, forwardRef } from 'react';

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  fullWidth?: boolean;
  options: Array<{ value: string; label: string }>;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, fullWidth = false, options, className = '', ...props }, ref) => {
    return (
      <div className={clsx('flex flex-col gap-1.5', fullWidth && 'w-full')}>
        {label && (
          <label className="text-sm font-medium text-gray-300">{label}</label>
        )}
        <select
          ref={ref}
          className={clsx(
            'px-4 py-2 rounded-lg bg-[#0f3460] border border-white/10 text-white',
            'focus:outline-none focus:ring-2 focus:ring-[#e94560] focus:border-transparent',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            error && 'border-red-500 focus:ring-red-500',
            fullWidth && 'w-full',
            className
          )}
          {...props}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {error && <p className="text-sm text-red-400">{error}</p>}
      </div>
    );
  }
);

Select.displayName = 'Select';

interface SelectInputProps extends Omit<SelectProps, 'options'> {
  children: React.ReactNode;
}

export const SelectInput = forwardRef<HTMLSelectElement, SelectInputProps>(
  ({ label, error, fullWidth = false, children, className = '', ...props }, ref) => {
    return (
      <div className={clsx('flex flex-col gap-1.5', fullWidth && 'w-full')}>
        {label && (
          <label className="text-sm font-medium text-gray-300">{label}</label>
        )}
        <select
          ref={ref}
          className={clsx(
            'px-4 py-2 rounded-lg bg-[#0f3460] border border-white/10 text-white',
            'focus:outline-none focus:ring-2 focus:ring-[#e94560] focus:border-transparent',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            error && 'border-red-500 focus:ring-red-500',
            fullWidth && 'w-full',
            className
          )}
          {...props}
        >
          {children}
        </select>
        {error && <p className="text-sm text-red-400">{error}</p>}
      </div>
    );
  }
);

SelectInput.displayName = 'SelectInput';

function clsx(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}
