import Link from "next/link";
import { ArrowRight, FileText, Lock, Unlock } from "lucide-react";
import type { Collateral } from "@/hooks/useCollateral";

const STATUS_STYLES: Record<string, string> = {
  LOCKED: "bg-blue-100 text-blue-800",
  RELEASED: "bg-green-100 text-green-800",
  LIQUIDATED: "bg-red-100 text-red-800",
};

const STATUS_ICONS: Record<string, React.ReactNode> = {
  LOCKED: <Lock className="h-3 w-3" />,
  RELEASED: <Unlock className="h-3 w-3" />,
  LIQUIDATED: <FileText className="h-3 w-3" />,
};

export function CollateralCard({ collateral }: { collateral: Collateral }) {
  const value = new Intl.NumberFormat("en", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  }).format(Number(collateral.amount));

  return (
    <Link
      href={`/collateral/${collateral.id}`}
      className="block rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition hover:border-blue-300 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 sm:p-5"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <span
              className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-semibold ${STATUS_STYLES[collateral.status] ?? "bg-gray-100 text-gray-700"}`}
            >
              {STATUS_ICONS[collateral.status]}
              {collateral.status}
            </span>
            <span className="text-xs text-gray-500">{collateral.assetType ?? collateral.assetCode}</span>
          </div>
          <p className="text-2xl font-bold text-blue-900">{value}</p>
          {collateral.description && (
            <p className="truncate text-sm text-gray-600">{collateral.description}</p>
          )}
          {collateral.issuer && (
            <p className="text-xs text-gray-400">Issuer: {collateral.issuer}</p>
          )}
        </div>
        <ArrowRight className="h-5 w-5 shrink-0 text-gray-400" />
      </div>
      <div className="mt-3 flex items-center justify-between text-xs text-gray-500">
        <span>{collateral.documents?.length ?? 0} document(s)</span>
        <span>{new Date(collateral.createdAt).toLocaleDateString()}</span>
      </div>
    </Link>
  );
}
