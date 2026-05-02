import { ReactNode } from 'react';

interface StatsCardProps {
  title: string;
  value?: string | number;
  change?: {
    value: number;
    label: string;
  };
  icon?: ReactNode;
  className?: string;
  loading?: boolean;
}

export const StatsCard = ({
  title,
  value,
  change,
  icon,
  className = '',
  loading = false,
}: StatsCardProps) => {
  if (loading) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg shadow p-6 animate-pulse ${className}`}>
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded w-2/3 mb-3" />
            <div className="h-7 bg-gray-200 dark:bg-gray-700 rounded w-1/2" />
          </div>
          <div className="w-8 h-8 bg-gray-200 dark:bg-gray-700 rounded-full" />
        </div>
      </div>
    );
  }

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg shadow p-6 ${className}`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-600 dark:text-gray-400">{title}</p>
          <p className="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-1">{value}</p>
          {change && (
            <p className={`text-sm mt-2 ${
              change.value >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
            }`}>
              {change.value >= 0 ? '+' : ''}{change.value}% {change.label}
            </p>
          )}
        </div>
        {icon && (
          <div className="text-gray-400 dark:text-gray-500">
            {icon}
          </div>
        )}
      </div>
    </div>
  );
};
