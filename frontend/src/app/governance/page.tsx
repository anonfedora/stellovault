"use client";

import Link from "next/link";
import { useState, useMemo } from "react";
import { useGovernance, type ProposalStatus } from "@/hooks/useGovernance";
import { ProposalCard } from "@/components/governance/ProposalCard";
import { Footer, Navbar } from "@/components";

const STATUS_OPTIONS: Array<ProposalStatus | "ALL"> = [
  "ALL",
  "OPEN",
  "PASSED",
  "REJECTED",
  "EXECUTED",
];

const TYPE_OPTIONS = [
  "All Types",
  "Protocol Settings",
  "Reward Parameters",
  "Treasury Grant",
  "Community Initiative",
];

type SortOption = "newest" | "oldest" | "mostVotes";

export default function GovernancePage() {
  const { proposals, wsConnected } = useGovernance();
  const [statusFilter, setStatusFilter] = useState<ProposalStatus | "ALL">("ALL");
  const [typeFilter, setTypeFilter] = useState("All Types");
  const [sortBy, setSortBy] = useState<SortOption>("newest");

  const filtered = useMemo(() => {
    let list = [...proposals];

    if (statusFilter !== "ALL") {
      list = list.filter((p) => p.status === statusFilter);
    }
    if (typeFilter !== "All Types") {
      list = list.filter((p) => p.type === typeFilter);
    }

    list.sort((a, b) => {
      if (sortBy === "newest") return b.createdAt - a.createdAt;
      if (sortBy === "oldest") return a.createdAt - b.createdAt;
      // mostVotes
      const totalA = a.votes.for + a.votes.against + a.votes.abstain;
      const totalB = b.votes.for + b.votes.against + b.votes.abstain;
      return totalB - totalA;
    });

    return list;
  }, [proposals, statusFilter, typeFilter, sortBy]);

  return (
    <>
      <Navbar />
      <div className="mt-10 min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6 lg:px-8">
        <div className="max-w-4xl mx-auto">
          {/* Header */}
          <header className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-6 mb-10">
            <div>
              <div className="flex items-center gap-3 mb-1">
                <h1 className="text-4xl font-extrabold tracking-tight text-zinc-900 dark:text-zinc-50">
                  Governance
                </h1>
                {wsConnected && (
                  <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-400 text-xs font-bold">
                    <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
                    Live
                  </span>
                )}
              </div>
              <p className="text-lg text-zinc-600 dark:text-zinc-400">
                Shape the future of the DAO by voting on active proposals.
              </p>
            </div>

            <Link
              href="/governance/new"
              className="flex h-12 items-center justify-center gap-2 rounded-full bg-zinc-900 px-6 font-bold text-white transition-all hover:bg-zinc-800 active:scale-95 dark:bg-white dark:text-zinc-950 dark:hover:bg-zinc-100 shrink-0"
            >
              + Create Proposal
            </Link>
          </header>

          {/* Filters */}
          <div className="flex flex-col sm:flex-row gap-3 mb-8">
            {/* Status tabs */}
            <div className="flex gap-1 p-1 rounded-xl bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 overflow-x-auto">
              {STATUS_OPTIONS.map((s) => (
                <button
                  key={s}
                  onClick={() => setStatusFilter(s)}
                  className={[
                    "px-3 py-1.5 rounded-lg text-xs font-bold whitespace-nowrap transition-all",
                    statusFilter === s
                      ? "bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-50 shadow-sm"
                      : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200",
                  ].join(" ")}
                >
                  {s}
                </button>
              ))}
            </div>

            {/* Type filter */}
            <select
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value)}
              className="px-4 py-2 rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
            >
              {TYPE_OPTIONS.map((t) => (
                <option key={t}>{t}</option>
              ))}
            </select>

            {/* Sort */}
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortOption)}
              className="px-4 py-2 rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
            >
              <option value="newest">Newest First</option>
              <option value="oldest">Oldest First</option>
              <option value="mostVotes">Most Votes</option>
            </select>
          </div>

          {/* Count */}
          <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-4 font-medium">
            {filtered.length} proposal{filtered.length !== 1 ? "s" : ""}
          </p>

          {/* List */}
          <div className="grid grid-cols-1 gap-6">
            {filtered.length > 0 ? (
              filtered.map((proposal) => (
                <ProposalCard key={proposal.id} proposal={proposal} />
              ))
            ) : (
              <div className="text-center py-20 bg-white dark:bg-zinc-900 rounded-3xl border border-dashed border-zinc-200 dark:border-zinc-800">
                <p className="text-zinc-500 font-medium">No proposals match your filters.</p>
                <button
                  onClick={() => { setStatusFilter("ALL"); setTypeFilter("All Types"); }}
                  className="mt-3 text-sm text-blue-600 dark:text-blue-400 hover:underline font-medium"
                >
                  Clear filters
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
      <Footer />
    </>
  );
}
