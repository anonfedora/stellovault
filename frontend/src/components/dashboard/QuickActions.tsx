"use client";

import Link from "next/link";
import {
  Banknote,
  ShieldCheck,
  Briefcase,
  ArrowUpRight,
  FileDown,
  Activity,
} from "lucide-react";

export interface QuickAction {
  label: string;
  href: string;
  description: string;
  Icon: React.ComponentType<{ className?: string }>;
  external?: boolean;
}

interface QuickActionsProps {
  actions?: QuickAction[];
  onExport?: () => void;
}

const DEFAULT_ACTIONS: QuickAction[] = [
  {
    label: "Originate loan",
    href: "/loans/new",
    description: "Pledge collateral and start a new financing deal.",
    Icon: Banknote,
  },
  {
    label: "Monitor escrows",
    href: "/escrows",
    description: "Review releases awaiting oracle confirmation.",
    Icon: ShieldCheck,
  },
  {
    label: "Manage collateral",
    href: "/collateral",
    description: "Inspect tokenised assets and fractional shares.",
    Icon: Briefcase,
  },
  {
    label: "Risk snapshot",
    href: "/risk",
    description: "Drill into the components driving your score.",
    Icon: Activity,
  },
];

export const QuickActions = ({
  actions = DEFAULT_ACTIONS,
  onExport,
}: QuickActionsProps) => {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Quick Actions
          </h2>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Shortcuts to the operations you use most.
          </p>
        </div>
        {onExport && (
          <button
            type="button"
            onClick={onExport}
            className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-gray-700 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <FileDown className="h-3.5 w-3.5" />
            Export
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {actions.map(({ label, href, description, Icon, external }) => (
          <Link
            key={label}
            href={href}
            target={external ? "_blank" : undefined}
            rel={external ? "noopener noreferrer" : undefined}
            className="group flex items-start gap-3 rounded-lg border border-gray-100 dark:border-gray-700 bg-gray-50/60 dark:bg-gray-900/40 p-4 hover:border-blue-300 dark:hover:border-blue-600 hover:bg-white dark:hover:bg-gray-800 transition-colors"
          >
            <span className="rounded-md bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-300 p-2">
              <Icon className="h-4 w-4" />
            </span>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {label}
                </p>
                <ArrowUpRight className="h-4 w-4 text-gray-400 group-hover:text-blue-500 transition-colors" />
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                {description}
              </p>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
};
