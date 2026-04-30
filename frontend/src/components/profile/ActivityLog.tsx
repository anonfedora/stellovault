'use client';

import { useState, useEffect } from 'react';
import { History, ChevronDown, ChevronUp, Filter } from 'lucide-react';
import { ActivityLog as ActivityLogType } from '@/hooks/useProfile';
import { toast } from 'sonner';

interface ActivityLogProps {
  onFetch: (limit?: number) => Promise<ActivityLogType[]>;
}

export function ActivityLog({ onFetch }: ActivityLogProps) {
  const [activities, setActivities] = useState<ActivityLogType[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [filter, setFilter] = useState<'all' | 'security' | 'transaction' | 'settings'>('all');

  useEffect(() => {
    loadActivities();
  }, [filter]);

  const loadActivities = async () => {
    try {
      setIsLoading(true);
      const data = await onFetch(50);
      
      if (filter === 'all') {
        setActivities(data);
      } else {
        const filtered = data.filter((activity) => {
          const action = activity.action.toLowerCase();
          if (filter === 'security') return action.includes('login') || action.includes('logout') || action.includes('security');
          if (filter === 'transaction') return action.includes('transaction') || action.includes('payment') || action.includes('escrow');
          if (filter === 'settings') return action.includes('update') || action.includes('change') || action.includes('settings');
          return true;
        });
        setActivities(filtered);
      }
    } catch (error) {
      toast.error('Failed to load activity log');
    } finally {
      setIsLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minutes ago`;
    if (diffHours < 24) return `${diffHours} hours ago`;
    if (diffDays < 7) return `${diffDays} days ago`;
    
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  };

  const getActionIcon = (action: string) => {
    const actionLower = action.toLowerCase();
    if (actionLower.includes('login') || actionLower.includes('logout')) return '🔐';
    if (actionLower.includes('transaction') || actionLower.includes('payment')) return '💳';
    if (actionLower.includes('update') || actionLower.includes('change')) return '⚙️';
    if (actionLower.includes('escrow')) return '📦';
    if (actionLower.includes('wallet')) return '👛';
    return '📝';
  };

  const getActionColor = (action: string) => {
    const actionLower = action.toLowerCase();
    if (actionLower.includes('login') || actionLower.includes('logout')) return 'text-blue-600 bg-blue-50';
    if (actionLower.includes('transaction') || actionLower.includes('payment')) return 'text-green-600 bg-green-50';
    if (actionLower.includes('update') || actionLower.includes('change')) return 'text-purple-600 bg-purple-50';
    if (actionLower.includes('escrow')) return 'text-orange-600 bg-orange-50';
    return 'text-gray-600 bg-gray-50';
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-2">
          <History className="w-5 h-5 text-gray-600" />
          <h2 className="text-lg font-semibold text-gray-900">Activity Log</h2>
        </div>
        
        <div className="flex items-center space-x-2">
          <div className="relative">
            <Filter className="w-4 h-4 text-gray-400 absolute left-3 top-1/2 transform -translate-y-1/2" />
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as any)}
              className="pl-10 pr-4 py-2 text-sm border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 appearance-none cursor-pointer"
            >
              <option value="all">All Activities</option>
              <option value="security">Security</option>
              <option value="transaction">Transactions</option>
              <option value="settings">Settings</option>
            </select>
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="animate-pulse h-16 bg-gray-100 rounded-lg" />
          ))}
        </div>
      ) : activities.length === 0 ? (
        <p className="text-center text-gray-500 py-8">No activity recorded</p>
      ) : (
        <div className="space-y-3">
          {activities.map((activity) => (
            <div
              key={activity.id}
              className="border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <button
                onClick={() => setExpandedId(expandedId === activity.id ? null : activity.id)}
                className="w-full p-4 flex items-center justify-between text-left"
              >
                <div className="flex items-center space-x-4">
                  <span className="text-2xl">{getActionIcon(activity.action)}</span>
                  <div>
                    <p className="font-medium text-gray-900">{activity.action}</p>
                    <p className="text-sm text-gray-500">{formatDate(activity.timestamp)}</p>
                  </div>
                </div>
                
                <div className="flex items-center space-x-4">
                  {activity.ipAddress && (
                    <span className="text-xs text-gray-400 font-mono">{activity.ipAddress}</span>
                  )}
                  {expandedId === activity.id ? (
                    <ChevronUp className="w-4 h-4 text-gray-400" />
                  ) : (
                    <ChevronDown className="w-4 h-4 text-gray-400" />
                  )}
                </div>
              </button>

              {expandedId === activity.id && (
                <div className="px-4 pb-4 pt-0 border-t border-gray-100">
                  <div className="pt-3 space-y-2">
                    <div className="flex items-center space-x-2">
                      <span className="text-sm text-gray-500">Action ID:</span>
                      <code className="text-xs text-gray-600 font-mono">{activity.id}</code>
                    </div>
                    {activity.resourceId && (
                      <div className="flex items-center space-x-2">
                        <span className="text-sm text-gray-500">Resource ID:</span>
                        <code className="text-xs text-gray-600 font-mono">{activity.resourceId}</code>
                      </div>
                    )}
                    {activity.details && Object.keys(activity.details).length > 0 && (
                      <div>
                        <span className="text-sm text-gray-500 block mb-2">Details:</span>
                        <pre className="text-xs text-gray-600 bg-gray-50 p-3 rounded-lg overflow-x-auto">
                          {JSON.stringify(activity.details, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
