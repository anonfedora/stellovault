"use client";

import type { PortfolioSnapshot } from "@/hooks/useDashboard";

interface PortfolioOverviewProps {
  portfolio: PortfolioSnapshot;
  loading?: boolean;
}

const formatCurrency = (value: number) =>
  `$${Math.round(value).toLocaleString()}`;

export const PortfolioOverview = ({
  portfolio,
  loading = false,
}: PortfolioOverviewProps) => {
  const utilizationPct = Math.min(100, portfolio.utilization * 100);
  const repaidPct =
    portfolio.totalRepaid + portfolio.totalOutstanding === 0
      ? 0
      : (portfolio.totalRepaid /
          (portfolio.totalRepaid + portfolio.totalOutstanding)) *
        100;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Portfolio Overview
          </h2>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Snapshot of assets pledged versus liabilities outstanding.
          </p>
        </div>
        {loading && (
          <div
            className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"
            aria-label="Loading portfolio"
          />
        )}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <Tile
          label="Collateral pledged"
          value={formatCurrency(portfolio.totalCollateralValue)}
          tone="neutral"
        />
        <Tile
          label="Total principal"
          value={formatCurrency(portfolio.totalPrincipal)}
          tone="neutral"
        />
        <Tile
          label="Outstanding"
          value={formatCurrency(portfolio.totalOutstanding)}
          tone="warning"
        />
        <Tile
          label="Repaid"
          value={formatCurrency(portfolio.totalRepaid)}
          tone="success"
        />
      </div>

      <div className="space-y-4">
        <ProgressRow
          label="Repayment progress"
          percent={repaidPct}
          accent="bg-emerald-500"
          rightLabel={`${repaidPct.toFixed(1)}%`}
        />
        <ProgressRow
          label="Collateral utilization"
          percent={utilizationPct}
          accent="bg-blue-500"
          rightLabel={`${utilizationPct.toFixed(1)}%`}
        />
      </div>

      <div className="mt-6 grid grid-cols-3 gap-4">
        <CountPill
          label="Active"
          value={portfolio.activeLoanCount}
          tone="success"
        />
        <CountPill
          label="Pending"
          value={portfolio.pendingLoanCount}
          tone="warning"
        />
        <CountPill
          label="Defaulted"
          value={portfolio.defaultedLoanCount}
          tone="danger"
        />
      </div>
    </div>
  );
};

const TONE_STYLES = {
  neutral: "text-gray-900 dark:text-gray-100",
  success: "text-emerald-700 dark:text-emerald-400",
  warning: "text-amber-700 dark:text-amber-400",
  danger: "text-red-700 dark:text-red-400",
} as const;

const Tile = ({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone: keyof typeof TONE_STYLES;
}) => (
  <div className="rounded-lg border border-gray-100 dark:border-gray-700 bg-gray-50/60 dark:bg-gray-900/40 p-4">
    <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">
      {label}
    </p>
    <p className={`mt-2 text-2xl font-semibold ${TONE_STYLES[tone]}`}>{value}</p>
  </div>
);

const ProgressRow = ({
  label,
  percent,
  accent,
  rightLabel,
}: {
  label: string;
  percent: number;
  accent: string;
  rightLabel: string;
}) => (
  <div>
    <div className="flex items-center justify-between mb-1.5">
      <span className="text-sm text-gray-600 dark:text-gray-300">{label}</span>
      <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
        {rightLabel}
      </span>
    </div>
    <div className="h-2 w-full bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
      <div
        className={`h-full ${accent} transition-all duration-500`}
        style={{ width: `${Math.max(0, Math.min(100, percent))}%` }}
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(percent)}
      />
    </div>
  </div>
);

const PILL_TONES = {
  success: "bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-300",
  warning: "bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-300",
  danger: "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300",
} as const;

const CountPill = ({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone: keyof typeof PILL_TONES;
}) => (
  <div
    className={`rounded-lg px-3 py-2 text-center ${PILL_TONES[tone]}`}
  >
    <p className="text-xs font-medium uppercase tracking-wide">{label}</p>
    <p className="text-xl font-semibold mt-1">{value}</p>
  </div>
);
