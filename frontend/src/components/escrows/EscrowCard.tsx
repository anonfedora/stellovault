import Link from "next/link";
import { ArrowRight, CalendarDays, Route, Users } from "lucide-react";
import type { Escrow } from "@/hooks/useEscrows";
import { StatusBadge } from "./StatusBadge";

export function EscrowCard({ escrow }: { escrow: Escrow }) {
  const amount = new Intl.NumberFormat("en", {
    style: "currency",
    currency: escrow.asset === "USDC" ? "USD" : "USD",
    maximumFractionDigits: 0,
  }).format(escrow.amount);

  return (
    <Link
      href={`/escrows/${escrow.id}`}
      className="block rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition hover:border-blue-300 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 sm:p-5"
    >
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge status={escrow.status} />
            <span className="text-xs font-medium text-gray-500">{escrow.id}</span>
          </div>
          <h2 className="text-lg font-bold text-gray-950">{escrow.title}</h2>
          <p className="text-2xl font-bold text-blue-900">
            {amount} <span className="text-sm text-gray-500">{escrow.asset}</span>
          </p>
        </div>
        <ArrowRight className="hidden h-5 w-5 text-gray-400 sm:block" />
      </div>

      <div className="mt-5 grid gap-3 text-sm text-gray-600 sm:grid-cols-3">
        <div className="flex items-start gap-2">
          <Users className="mt-0.5 h-4 w-4 text-blue-700" />
          <span className="min-w-0">
            <span className="block truncate">{escrow.buyer}</span>
            <span className="block truncate text-gray-400">{escrow.seller}</span>
          </span>
        </div>
        <div className="flex items-center gap-2">
          <CalendarDays className="h-4 w-4 text-blue-700" />
          <span>Due {new Date(escrow.dueDate).toLocaleDateString()}</span>
        </div>
        <div className="flex items-center gap-2">
          <Route className="h-4 w-4 text-blue-700" />
          <span className="truncate">{escrow.deliveryRoute}</span>
        </div>
      </div>
    </Link>
  );
}
