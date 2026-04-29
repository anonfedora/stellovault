"use client";

import { useEffect, useMemo } from "react";
import { ArrowLeft, FileText, Lock, Unlock } from "lucide-react";
import Link from "next/link";
import { ValuationChart } from "@/components/collateral/ValuationChart";
import { useCollateral } from "@/hooks/useCollateral";
import type { ValuationPoint } from "@/components/collateral/ValuationChart";

function generateMockHistory(amount: number): ValuationPoint[] {
  const points: ValuationPoint[] = [];
  const now = Date.now();
  for (let i = 29; i >= 0; i--) {
    points.push({
      date: new Date(now - i * 24 * 60 * 60 * 1000).toISOString(),
      value: amount * (0.9 + Math.random() * 0.2),
    });
  }
  return points;
}

export default function CollateralDetailPage({ params }: { params: { id: string } }) {
  const { collateral, ltv, loading, fetchCollateralById, fetchLTV } = useCollateral();

  useEffect(() => {
    fetchCollateralById(params.id);
    fetchLTV(params.id);
  }, [fetchCollateralById, fetchLTV, params.id]);

  const valuationHistory = useMemo(
    () => (collateral ? generateMockHistory(Number(collateral.amount)) : []),
    [collateral],
  );

  if (loading) {
    return (
      <main className="min-h-screen bg-gray-50 px-4 pt-6 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-4xl space-y-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-24 animate-pulse rounded-lg bg-gray-200" />
          ))}
        </div>
      </main>
    );
  }

  if (!collateral) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <p className="text-gray-500">Collateral not found.</p>
          <Link href="/collateral" className="mt-3 text-sm text-blue-700 hover:underline">
            Back to collateral
          </Link>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-4xl space-y-6">
        <Link href="/collateral" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to collateral
        </Link>

        {/* Header */}
        <div className="rounded-xl border border-gray-200 bg-white p-6">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <div className="flex items-center gap-2">
                {collateral.status === "LOCKED" ? (
                  <Lock className="h-5 w-5 text-blue-700" />
                ) : (
                  <Unlock className="h-5 w-5 text-green-600" />
                )}
                <span className="text-sm font-semibold text-gray-500">{collateral.assetType ?? collateral.assetCode}</span>
              </div>
              <p className="mt-2 text-3xl font-bold text-gray-900">
                ${Number(collateral.amount).toLocaleString()}
              </p>
              {collateral.description && (
                <p className="mt-1 text-gray-600">{collateral.description}</p>
              )}
              {collateral.issuer && (
                <p className="mt-1 text-sm text-gray-400">Issuer: {collateral.issuer}</p>
              )}
            </div>
            <span className={`rounded-full px-3 py-1 text-xs font-semibold ${collateral.status === "LOCKED" ? "bg-blue-100 text-blue-800" : collateral.status === "RELEASED" ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"}`}>
              {collateral.status}
            </span>
          </div>
          <div className="mt-4 grid grid-cols-2 gap-3 text-xs text-gray-500 sm:grid-cols-4">
            <div><p className="font-medium text-gray-700">Stellar Asset</p><p className="font-mono">{collateral.stellarAssetCode ?? "—"}</p></div>
            <div><p className="font-medium text-gray-700">Escrow</p><p className="font-mono truncate">{collateral.escrowId}</p></div>
            <div><p className="font-medium text-gray-700">Created</p><p>{new Date(collateral.createdAt).toLocaleDateString()}</p></div>
            <div><p className="font-medium text-gray-700">Updated</p><p>{new Date(collateral.updatedAt).toLocaleDateString()}</p></div>
          </div>
        </div>

        {/* LTV */}
        {ltv && (
          <div className="rounded-xl border border-gray-200 bg-white p-5">
            <h2 className="mb-4 text-sm font-semibold text-gray-700">Loan-to-Value (LTV)</h2>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4 text-sm">
              <div className="rounded-lg bg-gray-50 p-3">
                <p className="text-xs text-gray-400">Current LTV</p>
                <p className="text-xl font-bold text-gray-900">{ltv.currentLtv}%</p>
              </div>
              <div className="rounded-lg bg-gray-50 p-3">
                <p className="text-xs text-gray-400">Max LTV</p>
                <p className="text-xl font-bold text-gray-900">{ltv.maxLtvRatio}%</p>
              </div>
              <div className="rounded-lg bg-green-50 p-3">
                <p className="text-xs text-green-600">Available Credit</p>
                <p className="text-xl font-bold text-green-900">${ltv.availableCredit.toLocaleString()}</p>
              </div>
              <div className="rounded-lg bg-gray-50 p-3">
                <p className="text-xs text-gray-400">Outstanding Loans</p>
                <p className="text-xl font-bold text-gray-900">${ltv.totalLoanAmount.toLocaleString()}</p>
              </div>
            </div>
            <div className="mt-3 h-2 w-full rounded-full bg-gray-200">
              <div
                className="h-2 rounded-full bg-blue-600 transition-all"
                style={{ width: `${Math.min(100, ltv.currentLtv)}%` }}
              />
            </div>
          </div>
        )}

        {/* Valuation chart */}
        <ValuationChart data={valuationHistory} />

        {/* Documents */}
        <div className="rounded-xl border border-gray-200 bg-white p-5">
          <h2 className="mb-4 text-sm font-semibold text-gray-700">Documents</h2>
          {collateral.documents && collateral.documents.length > 0 ? (
            <ul className="space-y-2">
              {collateral.documents.map((doc) => (
                <li key={doc.hash} className="flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 px-3 py-2 text-sm">
                  <FileText className="h-4 w-4 text-gray-500" />
                  <span className="flex-1 truncate">{doc.name}</span>
                  <span className="font-mono text-xs text-gray-400">{doc.hash.slice(0, 12)}…</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-sm text-gray-400">No documents attached.</p>
          )}
        </div>

        {/* Metadata */}
        {collateral.metadataHash && (
          <div className="rounded-xl border border-gray-200 bg-white p-5">
            <h2 className="mb-2 text-sm font-semibold text-gray-700">On-chain Metadata</h2>
            <p className="break-all font-mono text-xs text-gray-500">{collateral.metadataHash}</p>
          </div>
        )}
      </div>
    </main>
  );
}
