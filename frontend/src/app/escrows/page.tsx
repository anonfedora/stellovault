"use client";

import Link from "next/link";
import { Plus, RefreshCcw, Search, SlidersHorizontal } from "lucide-react";
import { useState } from "react";
import { EscrowCard } from "@/components/escrows/EscrowCard";
import { statusLabels } from "@/components/escrows/StatusBadge";
import type { EscrowFilters, EscrowStatus } from "@/hooks/useEscrows";
import { useEscrows } from "@/hooks/useEscrows";

const statuses: Array<"all" | EscrowStatus> = [
  "all",
  "draft",
  "funded",
  "in_transit",
  "inspection",
  "released",
  "disputed",
];

export default function EscrowsPage() {
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [filters, setFilters] = useState<EscrowFilters>({
    status: "all",
    party: "",
    dateRange: "all",
  });
  const { escrows, allEscrows, lastSyncAt } = useEscrows(filters);

  const activeCount = allEscrows.filter((escrow) =>
    ["funded", "in_transit", "inspection", "disputed"].includes(escrow.status),
  ).length;

  const filterControls = (
    <div className="grid gap-3 sm:grid-cols-3">
      <label className="grid gap-2 text-sm font-medium text-gray-700">
        Status
        <select
          value={filters.status}
          onChange={(event) =>
            setFilters((current) => ({
              ...current,
              status: event.target.value as EscrowFilters["status"],
            }))
          }
          className="min-h-12 rounded-lg border border-gray-300 bg-white px-3 text-base text-gray-950"
        >
          {statuses.map((status) => (
            <option key={status} value={status}>
              {status === "all" ? "All statuses" : statusLabels[status]}
            </option>
          ))}
        </select>
      </label>

      <label className="grid gap-2 text-sm font-medium text-gray-700">
        Party or ID
        <div className="relative">
          <Search className="pointer-events-none absolute left-3 top-3.5 h-4 w-4 text-gray-400" />
          <input
            value={filters.party}
            onChange={(event) =>
              setFilters((current) => ({ ...current, party: event.target.value }))
            }
            className="min-h-12 w-full rounded-lg border border-gray-300 px-10 text-base text-gray-950"
            placeholder="Search escrows"
          />
        </div>
      </label>

      <label className="grid gap-2 text-sm font-medium text-gray-700">
        Updated
        <select
          value={filters.dateRange}
          onChange={(event) =>
            setFilters((current) => ({
              ...current,
              dateRange: event.target.value as EscrowFilters["dateRange"],
            }))
          }
          className="min-h-12 rounded-lg border border-gray-300 bg-white px-3 text-base text-gray-950"
        >
          <option value="all">Any time</option>
          <option value="7d">Last 7 days</option>
          <option value="30d">Last 30 days</option>
        </select>
      </label>
    </div>
  );

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-sm font-semibold text-blue-800">Escrow operations</p>
            <h1 className="mt-2 text-3xl font-bold tracking-tight sm:text-4xl">
              Trade escrows
            </h1>
            <p className="mt-2 max-w-2xl text-gray-600">
              Monitor live status, filter by counterparty, and act on escrow milestones.
            </p>
          </div>
          <Link
            href="/escrows/new"
            className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-blue-900 px-4 font-semibold text-white"
          >
            <Plus className="h-4 w-4" />
            New escrow
          </Link>
        </div>

        <section className="mt-6 grid gap-3 sm:grid-cols-3">
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="text-sm text-gray-500">Total escrows</p>
            <p className="mt-1 text-2xl font-bold">{allEscrows.length}</p>
          </div>
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="text-sm text-gray-500">Active workflows</p>
            <p className="mt-1 text-2xl font-bold">{activeCount}</p>
          </div>
          <div className="rounded-lg border border-gray-200 bg-white p-4">
            <p className="flex items-center gap-2 text-sm text-gray-500">
              <RefreshCcw className="h-4 w-4" />
              Live sync
            </p>
            <p className="mt-1 text-sm font-semibold">
              {new Date(lastSyncAt).toLocaleTimeString()}
            </p>
          </div>
        </section>

        <div className="mt-6 hidden rounded-lg border border-gray-200 bg-white p-4 sm:block">
          {filterControls}
        </div>

        <button
          type="button"
          onClick={() => setShowMobileFilters(true)}
          className="mt-6 inline-flex min-h-12 w-full items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white px-4 font-semibold text-gray-800 sm:hidden"
        >
          <SlidersHorizontal className="h-4 w-4" />
          Filters
        </button>

        <section className="mt-6 grid gap-4">
          {escrows.map((escrow) => (
            <EscrowCard key={escrow.id} escrow={escrow} />
          ))}
          {escrows.length === 0 && (
            <div className="rounded-lg border border-dashed border-gray-300 bg-white p-8 text-center text-gray-600">
              No escrows match the current filters.
            </div>
          )}
        </section>
      </div>

      {showMobileFilters && (
        <div className="fixed inset-0 z-50 bg-black/40 sm:hidden" onClick={() => setShowMobileFilters(false)}>
          <div
            className="absolute inset-x-0 bottom-0 rounded-t-2xl bg-white p-4 shadow-2xl"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="mx-auto mb-4 h-1 w-12 rounded-full bg-gray-300" />
            {filterControls}
            <button
              type="button"
              onClick={() => setShowMobileFilters(false)}
              className="mt-4 min-h-12 w-full rounded-lg bg-blue-900 font-semibold text-white"
            >
              Apply filters
            </button>
          </div>
        </div>
      )}
    </main>
  );
}
