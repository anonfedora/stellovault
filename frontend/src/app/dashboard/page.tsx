"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Download, RefreshCw, Wifi, WifiOff } from "lucide-react";
import { StatsCard } from "@/components/dashboard/StatsCard";
import { PortfolioOverview } from "@/components/dashboard/PortfolioOverview";
import { RecentActivity } from "@/components/dashboard/RecentActivity";
import { QuickActions } from "@/components/dashboard/QuickActions";
import { MetricsChart } from "@/components/dashboard/MetricsChart";
import { NotificationCenter } from "@/components/dashboard/NotificationCenter";
import {
  WidgetCustomizer,
  type WidgetDefinition,
} from "@/components/dashboard/WidgetCustomizer";
import { LiquidityModal } from "@/components/dashboard/LiquidityModal";
import { QuickStartCard } from "@/components/onboarding/QuickStartCard";
import { useDashboard } from "@/hooks/useDashboard";
import { useWalletAuth } from "@/hooks/useWalletAuth";
import {
  exportDashboardReport,
  type ExportFormat,
} from "@/utils/dashboardExport";

const WIDGETS: WidgetDefinition[] = [
  {
    id: "portfolio",
    label: "Portfolio overview",
    description: "Headline assets, liabilities and ratios.",
  },
  {
    id: "metrics",
    label: "Performance chart",
    description: "Outstanding balance and repayments over time.",
  },
  {
    id: "activity",
    label: "Recent activity",
    description: "Latest loan, repayment and escrow events.",
  },
  {
    id: "actions",
    label: "Quick actions",
    description: "Shortcuts to common workflows.",
  },
  {
    id: "notifications",
    label: "Notification center",
    description: "Alerts and platform updates.",
  },
];

const STORAGE_KEY = "stellovault_dashboard_widgets";

const DEFAULT_VISIBILITY: Record<string, boolean> = WIDGETS.reduce(
  (acc, widget) => {
    acc[widget.id] = true;
    return acc;
  },
  {} as Record<string, boolean>,
);

const formatTime = (date: Date | null) =>
  date
    ? date.toLocaleTimeString("en-US", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      })
    : "—";

export default function DashboardPage() {
  const dashboard = useDashboard();
  const { publicKey } = useWalletAuth();
  const {
    portfolio,
    activity,
    notifications,
    metrics,
    loans,
    loading,
    error,
    connected,
    lastUpdated,
    refresh,
    markNotificationRead,
    clearNotifications,
  } = dashboard;

  const [visibleWidgets, setVisibleWidgets] = useState<Record<string, boolean>>(
    () => {
      if (typeof window === "undefined") return DEFAULT_VISIBILITY;
      try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return DEFAULT_VISIBILITY;
        const parsed = JSON.parse(raw) as Record<string, boolean>;
        return { ...DEFAULT_VISIBILITY, ...parsed };
      } catch {
        return DEFAULT_VISIBILITY;
      }
    },
  );
  const [exportFormat, setExportFormat] = useState<ExportFormat>("csv");
  const [liquidityOpen, setLiquidityOpen] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(visibleWidgets));
    } catch {
      // Ignore quota errors.
    }
  }, [visibleWidgets]);

  const summaryMetrics = useMemo(
    () => [
      {
        title: "Active loans",
        value: portfolio.activeLoanCount.toString(),
      },
      {
        title: "Collateral pledged",
        value: `$${Math.round(portfolio.totalCollateralValue).toLocaleString()}`,
      },
      {
        title: "Outstanding",
        value: `$${Math.round(portfolio.totalOutstanding).toLocaleString()}`,
      },
      {
        title: "Repaid",
        value: `$${Math.round(portfolio.totalRepaid).toLocaleString()}`,
      },
      {
        title: "Pending notifications",
        value: notifications.filter((n) => !n.read).length.toString(),
      },
    ],
    [portfolio, notifications],
  );

  const handleExport = useCallback(() => {
    const rows = loans.map((loan) => ({
      id: loan.id,
      borrower: loan.borrower,
      asset: loan.collateralAssetType,
      collateralValue: loan.collateralValue,
      principal: loan.principal,
      interestRate: loan.interestRate,
      status: loan.status,
      totalRepaid: loan.repayments.reduce((sum, r) => sum + r.amount, 0),
      maturityDate: loan.maturityDate.toISOString(),
    }));
    exportDashboardReport("stellovault-dashboard", rows, exportFormat);
  }, [loans, exportFormat]);

  const isVisible = (id: string) => visibleWidgets[id] !== false;

  return (
    <div className="container mx-auto px-4 py-8 space-y-6">
      <header className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            Dashboard Overview
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Real-time visibility into your trade finance deals, collateral, and
            settlements.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span
            className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium ${
              connected
                ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300"
                : "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-300"
            }`}
            title={connected ? "Live updates connected" : "Live updates offline"}
          >
            {connected ? (
              <Wifi className="h-3.5 w-3.5" />
            ) : (
              <WifiOff className="h-3.5 w-3.5" />
            )}
            {connected ? "Live" : "Offline"}
          </span>
          <span className="text-xs text-gray-500 dark:text-gray-400">
            Updated {formatTime(lastUpdated)}
          </span>
          <button
            type="button"
            onClick={() => refresh()}
            className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-gray-700 px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
            disabled={loading}
            aria-label="Refresh dashboard"
          >
            <RefreshCw
              className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </button>
          <select
            value={exportFormat}
            onChange={(e) => setExportFormat(e.target.value as ExportFormat)}
            className="rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-2 py-1.5 text-sm text-gray-700 dark:text-gray-200"
            aria-label="Export format"
          >
            <option value="csv">CSV</option>
            <option value="json">JSON</option>
          </select>
          <button
            type="button"
            onClick={handleExport}
            className="inline-flex items-center gap-1.5 rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700"
          >
            <Download className="h-4 w-4" />
            Export
          </button>
          <WidgetCustomizer
            widgets={WIDGETS}
            visibleWidgets={visibleWidgets}
            onChange={setVisibleWidgets}
          />
        </div>
      </header>

      {error && (
        <div className="rounded-lg border border-red-200 dark:border-red-900/40 bg-red-50 dark:bg-red-900/20 px-4 py-3 text-sm text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      <QuickStartCard />

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
        {summaryMetrics.map((metric) => (
          <StatsCard
            key={metric.title}
            title={metric.title}
            value={metric.value}
          />
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {isVisible("portfolio") && (
          <div className="lg:col-span-2">
            <PortfolioOverview portfolio={portfolio} loading={loading} />
          </div>
        )}
        {isVisible("actions") && (
          <div>
            <QuickActions onExport={handleExport} onLiquidity={() => setLiquidityOpen(true)} />
          </div>
        )}
      </div>

      {isVisible("metrics") && (
        <MetricsChart data={metrics} loading={loading} />
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {isVisible("activity") && (
          <RecentActivity items={activity} loading={loading} />
        )}
        {isVisible("notifications") && (
          <NotificationCenter
            notifications={notifications}
            onMarkRead={markNotificationRead}
            onClear={clearNotifications}
          />
        )}
      </div>

      <LiquidityModal
        isOpen={liquidityOpen}
        onClose={() => setLiquidityOpen(false)}
        publicKey={publicKey}
      />
    </div>
  );
}
