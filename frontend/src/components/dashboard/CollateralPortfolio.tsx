"use client";

import { useMemo, useState } from "react";
import type { Loan } from "@/types";

interface CollateralPortfolioProps {
  loans: Loan[];
  loading?: boolean;
}

type AssetStatus = "active" | "pending" | "defaulted";

interface CollateralRow {
  id: string;
  assetType: string;
  collateralValue: number;
  principal: number;
  utilization: number;
  status: AssetStatus;
  maturityDate: Date;
}

const STATUS_BADGE: Record<AssetStatus, string> = {
  active:
    "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
  pending:
    "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
  defaulted:
    "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300",
};

const fmt = (n: number) => `$${Math.round(n).toLocaleString()}`;
const fmtPct = (n: number) => `${(n * 100).toFixed(1)}%`;

function SkeletonRow() {
  return (
    <tr className="animate-pulse">
      {[...Array(6)].map((_, i) => (
        <td key={i} className="px-4 py-3">
          <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4" />
        </td>
      ))}
    </tr>
  );
}

export function CollateralPortfolio({
  loans,
  loading = false,
}: CollateralPortfolioProps) {
  const [sortKey, setSortKey] = useState<keyof CollateralRow>("collateralValue");
  const [sortAsc, setSortAsc] = useState(false);

  const rows = useMemo<CollateralRow[]>(
    () =>
      loans.map((loan) => ({
        id: loan.id,
        assetType: loan.collateralAssetType,
        collateralValue: loan.collateralValue,
        principal: loan.principal,
        utilization:
          loan.collateralValue === 0 ? 0 : loan.principal / loan.collateralValue,
        status: loan.status.toLowerCase() as AssetStatus,
        maturityDate: loan.maturityDate,
      })),
    [loans],
  );

  const sorted = useMemo(() => {
    return [...rows].sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      const cmp =
        av instanceof Date && bv instanceof Date
          ? av.getTime() - bv.getTime()
          : typeof av === "number" && typeof bv === "number"
            ? av - bv
            : String(av).localeCompare(String(bv));
      return sortAsc ? cmp : -cmp;
    });
  }, [rows, sortKey, sortAsc]);

  const handleSort = (key: keyof CollateralRow) => {
    if (key === sortKey) {
      setSortAsc((v) => !v);
    } else {
      setSortKey(key);
      setSortAsc(false);
    }
  };

  const SortIcon = ({ col }: { col: keyof CollateralRow }) =>
    sortKey === col ? (
      <span className="ml-1 text-blue-500">{sortAsc ? "↑" : "↓"}</span>
    ) : (
      <span className="ml-1 text-gray-300 dark:text-gray-600">↕</span>
    );

  const headers: { key: keyof CollateralRow; label: string }[] = [
    { key: "id", label: "Loan ID" },
    { key: "assetType", label: "Asset Type" },
    { key: "collateralValue", label: "Collateral Value" },
    { key: "principal", label: "Principal" },
    { key: "utilization", label: "Utilization" },
    { key: "status", label: "Status" },
    { key: "maturityDate", label: "Maturity" },
  ];

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden">
      <div className="px-6 py-4 border-b border-gray-100 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          Collateral Portfolio
        </h2>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
          Tokenized assets pledged as collateral. Click column headers to sort.
        </p>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-gray-50 dark:bg-gray-900/40 text-left">
              {headers.map(({ key, label }) => (
                <th
                  key={key}
                  className="px-4 py-3 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 cursor-pointer select-none whitespace-nowrap hover:text-gray-700 dark:hover:text-gray-200"
                  onClick={() => handleSort(key)}
                  aria-sort={
                    sortKey === key
                      ? sortAsc
                        ? "ascending"
                        : "descending"
                      : "none"
                  }
                >
                  {label}
                  <SortIcon col={key} />
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
            {loading ? (
              <>
                <SkeletonRow />
                <SkeletonRow />
                <SkeletonRow />
              </>
            ) : sorted.length === 0 ? (
              <tr>
                <td
                  colSpan={headers.length}
                  className="px-4 py-10 text-center text-sm text-gray-500 dark:text-gray-400"
                >
                  No collateral assets found.
                </td>
              </tr>
            ) : (
              sorted.map((row) => (
                <tr
                  key={row.id}
                  className="hover:bg-gray-50 dark:hover:bg-gray-700/40 transition-colors"
                >
                  <td className="px-4 py-3 font-mono text-xs text-gray-700 dark:text-gray-300">
                    {row.id}
                  </td>
                  <td className="px-4 py-3 text-gray-900 dark:text-gray-100 font-medium capitalize">
                    {row.assetType.toLowerCase()}
                  </td>
                  <td className="px-4 py-3 text-gray-900 dark:text-gray-100">
                    {fmt(row.collateralValue)}
                  </td>
                  <td className="px-4 py-3 text-gray-900 dark:text-gray-100">
                    {fmt(row.principal)}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <div className="flex-1 h-1.5 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden min-w-[48px]">
                        <div
                          className="h-full bg-blue-500 rounded-full"
                          style={{
                            width: `${Math.min(100, row.utilization * 100)}%`,
                          }}
                        />
                      </div>
                      <span className="text-xs text-gray-600 dark:text-gray-400 shrink-0">
                        {fmtPct(row.utilization)}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium capitalize ${STATUS_BADGE[row.status] ?? STATUS_BADGE.pending}`}
                    >
                      {row.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                    {row.maturityDate.toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                    })}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
