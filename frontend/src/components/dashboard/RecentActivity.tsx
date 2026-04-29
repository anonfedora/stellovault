"use client";

import { useMemo, useState } from "react";
import type { ActivityItem } from "@/hooks/useDashboard";

interface RecentActivityProps {
  items: ActivityItem[];
  loading?: boolean;
  maxItems?: number;
}

const CATEGORY_LABELS: Record<ActivityItem["category"], string> = {
  loan: "Loans",
  repayment: "Repayments",
  escrow: "Escrows",
  system: "System",
};

const STATUS_COLORS: Record<NonNullable<ActivityItem["status"]>, string> = {
  success: "bg-emerald-500",
  pending: "bg-amber-500",
  failed: "bg-red-500",
  info: "bg-blue-500",
};

export const RecentActivity = ({
  items,
  loading = false,
  maxItems = 8,
}: RecentActivityProps) => {
  const [filter, setFilter] = useState<ActivityItem["category"] | "all">("all");

  const filtered = useMemo(() => {
    const list = filter === "all" ? items : items.filter((i) => i.category === filter);
    return list.slice(0, maxItems);
  }, [items, filter, maxItems]);

  const filterButtons: { key: typeof filter; label: string }[] = [
    { key: "all", label: "All" },
    { key: "loan", label: CATEGORY_LABELS.loan },
    { key: "repayment", label: CATEGORY_LABELS.repayment },
    { key: "escrow", label: CATEGORY_LABELS.escrow },
    { key: "system", label: CATEGORY_LABELS.system },
  ];

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Recent Activity
          </h2>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Latest events across your loans, escrows and transactions.
          </p>
        </div>
        {loading && (
          <span
            className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"
            aria-label="Loading activity"
          />
        )}
      </div>

      <div className="flex flex-wrap gap-2 mb-4">
        {filterButtons.map((btn) => (
          <button
            key={btn.key}
            type="button"
            onClick={() => setFilter(btn.key)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
              filter === btn.key
                ? "bg-blue-600 text-white"
                : "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
            }`}
          >
            {btn.label}
          </button>
        ))}
      </div>

      <ul className="flex-1 divide-y divide-gray-100 dark:divide-gray-700 overflow-y-auto">
        {filtered.length === 0 && (
          <li className="py-10 text-center text-sm text-gray-500 dark:text-gray-400">
            No activity to display yet.
          </li>
        )}
        {filtered.map((item) => {
          const status = item.status ?? "info";
          return (
            <li key={item.id} className="py-3 flex items-start gap-3">
              <span
                className={`mt-1.5 w-2.5 h-2.5 rounded-full ${STATUS_COLORS[status]}`}
                aria-hidden
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                    {item.title}
                  </p>
                  <time
                    className="text-xs text-gray-400 dark:text-gray-500 shrink-0"
                    dateTime={item.timestamp}
                  >
                    {new Date(item.timestamp).toLocaleString("en-US", {
                      month: "short",
                      day: "numeric",
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                  </time>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
                  {item.description}
                </p>
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
};
