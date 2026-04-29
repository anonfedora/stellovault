"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { AlertTriangle, ArrowLeft, Check, Flag, LockKeyhole, RefreshCcw } from "lucide-react";
import { EscrowTimeline } from "@/components/escrows/EscrowTimeline";
import { StatusBadge } from "@/components/escrows/StatusBadge";
import { useEscrows } from "@/hooks/useEscrows";

export default function EscrowDetailPage() {
  const params = useParams<{ id: string }>();
  const { getEscrowById, lastSyncAt } = useEscrows();
  const escrow = getEscrowById(decodeURIComponent(params.id));

  if (!escrow) {
    return (
      <main className="min-h-screen bg-gray-50 px-4 py-8 text-gray-950">
        <div className="mx-auto max-w-3xl rounded-lg border border-gray-200 bg-white p-6">
          <h1 className="text-2xl font-bold">Escrow not found</h1>
          <p className="mt-2 text-gray-600">This escrow ID is not available in the current workspace.</p>
          <Link href="/escrows" className="mt-4 inline-flex font-semibold text-blue-800">
            Back to escrows
          </Link>
        </div>
      </main>
    );
  }

  const canRelease = escrow.status === "inspection";
  const canDispute = !["released", "disputed"].includes(escrow.status);

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-28 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-6xl">
        <Link href="/escrows" className="inline-flex items-center gap-2 text-sm font-semibold text-blue-800">
          <ArrowLeft className="h-4 w-4" />
          Back to escrows
        </Link>

        <div className="mt-5 grid gap-5 lg:grid-cols-[1fr_360px]">
          <section className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm sm:p-6">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <StatusBadge status={escrow.status} />
                  <span className="text-sm font-medium text-gray-500">{escrow.id}</span>
                </div>
                <h1 className="mt-3 text-3xl font-bold">{escrow.title}</h1>
                <p className="mt-2 text-gray-600">
                  {escrow.buyer} buying from {escrow.seller}
                </p>
              </div>
              <p className="text-3xl font-bold text-blue-900">
                {escrow.amount.toLocaleString()} <span className="text-base text-gray-500">{escrow.asset}</span>
              </p>
            </div>

            <div className="mt-6 grid gap-3 sm:grid-cols-2">
              {[
                ["Buyer", escrow.buyer],
                ["Seller", escrow.seller],
                ["Delivery route", escrow.deliveryRoute],
                ["Oracle", escrow.oracle],
                ["Due date", new Date(escrow.dueDate).toLocaleDateString()],
                ["Last synced", new Date(lastSyncAt).toLocaleTimeString()],
              ].map(([label, value]) => (
                <div key={label} className="rounded-lg bg-gray-50 p-4">
                  <p className="text-xs font-semibold uppercase tracking-wide text-gray-500">{label}</p>
                  <p className="mt-1 font-semibold text-gray-950">{value}</p>
                </div>
              ))}
            </div>

            <div className="mt-8">
              <h2 className="mb-4 text-xl font-bold">Transaction timeline</h2>
              <EscrowTimeline events={escrow.events} />
            </div>
          </section>

          <aside className="space-y-4">
            <div className="rounded-lg border border-gray-200 bg-white p-5 shadow-sm">
              <h2 className="text-lg font-bold">Available actions</h2>
              <div className="mt-4 grid gap-3">
                <button
                  type="button"
                  disabled={!canRelease}
                  className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-emerald-600 px-4 font-semibold text-white disabled:bg-gray-200 disabled:text-gray-500"
                >
                  <Check className="h-4 w-4" />
                  Release funds
                </button>
                <button
                  type="button"
                  disabled={!canDispute}
                  className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg border border-red-200 px-4 font-semibold text-red-700 disabled:border-gray-200 disabled:text-gray-400"
                >
                  <Flag className="h-4 w-4" />
                  Open dispute
                </button>
                <button
                  type="button"
                  className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg border border-gray-300 px-4 font-semibold text-gray-800"
                >
                  <RefreshCcw className="h-4 w-4" />
                  Sync blockchain
                </button>
              </div>
            </div>

            <div className="rounded-lg border border-blue-100 bg-blue-50 p-5">
              <div className="flex gap-3">
                <LockKeyhole className="h-5 w-5 shrink-0 text-blue-800" />
                <p className="text-sm text-blue-950">
                  Funds remain locked until configured oracle checks and party approvals match the escrow terms.
                </p>
              </div>
            </div>

            {escrow.status === "disputed" && (
              <div className="rounded-lg border border-red-100 bg-red-50 p-5">
                <div className="flex gap-3">
                  <AlertTriangle className="h-5 w-5 shrink-0 text-red-700" />
                  <p className="text-sm text-red-950">
                    Dispute mode is active. Release and refund actions are paused until arbitration completes.
                  </p>
                </div>
              </div>
            )}
          </aside>
        </div>
      </div>
    </main>
  );
}
