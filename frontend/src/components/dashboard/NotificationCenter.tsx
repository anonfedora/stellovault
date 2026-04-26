"use client";

import { useMemo, useState } from "react";
import { Bell, BellOff, Check, AlertTriangle, Info, CheckCircle2 } from "lucide-react";
import type { DashboardNotification } from "@/hooks/useDashboard";

interface NotificationCenterProps {
  notifications: DashboardNotification[];
  onMarkRead: (id: string) => void;
  onClear: () => void;
}

const SEVERITY_ICON: Record<DashboardNotification["severity"], React.ComponentType<{ className?: string }>> = {
  info: Info,
  warning: AlertTriangle,
  critical: AlertTriangle,
  success: CheckCircle2,
};

const SEVERITY_STYLES: Record<DashboardNotification["severity"], string> = {
  info: "text-blue-600 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/20",
  warning: "text-amber-600 dark:text-amber-300 bg-amber-50 dark:bg-amber-900/20",
  critical: "text-red-600 dark:text-red-300 bg-red-50 dark:bg-red-900/20",
  success: "text-emerald-600 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-900/20",
};

export const NotificationCenter = ({
  notifications,
  onMarkRead,
  onClear,
}: NotificationCenterProps) => {
  const [showOnlyUnread, setShowOnlyUnread] = useState(false);

  const visible = useMemo(
    () =>
      showOnlyUnread
        ? notifications.filter((n) => !n.read)
        : notifications,
    [showOnlyUnread, notifications],
  );

  const unreadCount = notifications.filter((n) => !n.read).length;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Bell className="h-5 w-5 text-gray-500 dark:text-gray-300" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Notifications
          </h2>
          {unreadCount > 0 && (
            <span className="ml-1 inline-flex items-center justify-center h-5 min-w-5 px-1.5 rounded-full bg-blue-600 text-white text-[11px] font-semibold">
              {unreadCount}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setShowOnlyUnread((prev) => !prev)}
            className="text-xs font-medium text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
          >
            {showOnlyUnread ? "Show all" : "Unread only"}
          </button>
          <button
            type="button"
            onClick={onClear}
            className="inline-flex items-center gap-1 text-xs font-medium text-gray-500 dark:text-gray-400 hover:text-red-500"
            disabled={notifications.length === 0}
          >
            <BellOff className="h-3.5 w-3.5" />
            Clear
          </button>
        </div>
      </div>

      <ul className="flex-1 space-y-3 overflow-y-auto pr-1">
        {visible.length === 0 && (
          <li className="py-10 text-center text-sm text-gray-500 dark:text-gray-400">
            You&apos;re all caught up.
          </li>
        )}
        {visible.map((notification) => {
          const Icon = SEVERITY_ICON[notification.severity];
          return (
            <li
              key={notification.id}
              className={`rounded-lg border p-3 transition-colors ${
                notification.read
                  ? "border-gray-100 dark:border-gray-700 bg-gray-50/60 dark:bg-gray-900/40"
                  : "border-blue-100 dark:border-blue-900/40 bg-blue-50/40 dark:bg-blue-900/10"
              }`}
            >
              <div className="flex items-start gap-3">
                <span
                  className={`rounded-md p-2 ${SEVERITY_STYLES[notification.severity]}`}
                  aria-hidden
                >
                  <Icon className="h-4 w-4" />
                </span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                      {notification.title}
                    </p>
                    <time
                      className="text-xs text-gray-400 dark:text-gray-500 shrink-0"
                      dateTime={notification.createdAt}
                    >
                      {new Date(notification.createdAt).toLocaleString("en-US", {
                        month: "short",
                        day: "numeric",
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </time>
                  </div>
                  <p className="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
                    {notification.body}
                  </p>
                </div>
                {!notification.read && (
                  <button
                    type="button"
                    onClick={() => onMarkRead(notification.id)}
                    className="text-xs font-medium text-blue-600 dark:text-blue-300 hover:underline inline-flex items-center gap-1"
                    aria-label={`Mark ${notification.title} as read`}
                  >
                    <Check className="h-3.5 w-3.5" />
                    Mark
                  </button>
                )}
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
};
