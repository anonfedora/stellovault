"use client";

import { useParams } from "next/navigation";
import { useGovernance } from "@/hooks/useGovernance";
import { VoteTallyBar } from "@/components/governance/VoteTallyBar";
import { VoteButton } from "@/components/governance/VoteButton";
import Link from "next/link";
import { useMemo, useState, useEffect } from "react";

const STATUS_BADGE: Record<string, string> = {
  OPEN: "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-400",
  PASSED: "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400",
  REJECTED: "bg-rose-100 text-rose-700 dark:bg-rose-900/40 dark:text-rose-400",
  EXECUTED: "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400",
};

const STATUS_ICON: Record<string, string> = {
  OPEN: "🗳️",
  PASSED: "✅",
  REJECTED: "❌",
  EXECUTED: "⚙️",
};

function shortenAddress(addr: string) {
  if (addr.length <= 12) return addr;
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function formatVotes(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toLocaleString();
}

export default function ProposalDetailPage() {
  const params = useParams();
  const id = params?.id as string;
  const { proposals, vote, votingState, userVotes, wsConnected } = useGovernance();
  const [now] = useState(() => Date.now());

  const proposal = useMemo(
    () => proposals.find((p) => p.id === id) ?? null,
    [proposals, id],
  );

  const [currentTime, setCurrentTime] = useState(() => Date.now());

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(Date.now());
    }, 60000); // Update every minute
    return () => clearInterval(timer);
  }, []);

  if (!proposal) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-zinc-50 dark:bg-black">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4 text-zinc-900 dark:text-zinc-50">
            Proposal not found
          </h2>
          <Link href="/governance" className="text-blue-600 hover:underline">
            Back to Governance
          </Link>
        </div>
      </div>
    );
  }

  const userVote = userVotes[id];
  const hasVoted = Boolean(userVote);
  const isVoting = votingState.id === id && votingState.loading;
  const voteError = votingState.id === id ? votingState.error : null;

  const total = proposal.votes.for + proposal.votes.against + proposal.votes.abstain;
  const forPercent = total > 0 ? (proposal.votes.for / total) * 100 : 0;
  const quorumReached =
    proposal.votingPower && proposal.quorumRequired
      ? total / proposal.votingPower >= proposal.quorumRequired
      : true;

  const expiresDate = new Date(proposal.expiresAt);
  const timeLeft = (() => {
    const diff = proposal.expiresAt - now;
    if (diff <= 0) return "Expired";
    const days = Math.floor(diff / 86400000);
    const hours = Math.floor((diff % 86400000) / 3600000);
    return days > 0 ? `${days}d ${hours}h remaining` : `${hours}h remaining`;
  })();

  return (
    <div className="min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <Link
            href="/governance"
            className="inline-flex items-center gap-2 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors"
          >
            ← Back to Proposals
          </Link>
          {wsConnected && (
            <span className="flex items-center gap-1.5 text-xs font-bold text-emerald-600 dark:text-emerald-400">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
              Live updates
            </span>
          )}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Main content */}
          <div className="lg:col-span-2 space-y-6">
            <div className="bg-white dark:bg-zinc-900 p-8 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
              <div className="flex items-center gap-3 mb-6">
                <span className={`px-3 py-1 rounded-full text-xs font-bold tracking-tight ${STATUS_BADGE[proposal.status] ?? STATUS_BADGE.OPEN}`}>
                  {STATUS_ICON[proposal.status]} {proposal.status}
                </span>
                <span className="text-xs font-medium text-zinc-400">#{proposal.id}</span>
                <span className="ml-auto text-xs font-medium text-zinc-400 bg-zinc-100 dark:bg-zinc-800 px-2.5 py-1 rounded-full">
                  {proposal.type}
                </span>
              </div>

              <h1 className="text-3xl font-extrabold mb-4 text-zinc-900 dark:text-zinc-50 leading-tight">
                {proposal.title}
              </h1>

              <div className="flex flex-wrap items-center gap-6 mb-8 pb-8 border-b border-zinc-100 dark:border-zinc-800">
                <div className="flex items-center gap-3">
                  <div className="h-9 w-9 rounded-full bg-gradient-to-tr from-blue-500 to-emerald-500 shrink-0" />
                  <div>
                    <p className="text-[10px] font-bold text-zinc-400 uppercase tracking-widest">Creator</p>
                    <p className="font-mono text-sm font-semibold text-zinc-700 dark:text-zinc-300">
                      {shortenAddress(proposal.creator)}
                    </p>
                  </div>
                </div>
                <div>
                  <p className="text-[10px] font-bold text-zinc-400 uppercase tracking-widest">Created</p>
                  <p className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">
                    {new Date(proposal.createdAt).toLocaleDateString()}
                  </p>
                </div>
                <div>
                  <p className="text-[10px] font-bold text-zinc-400 uppercase tracking-widest">
                    {proposal.status === "OPEN" ? "Expires" : "Ended"}
                  </p>
                  <p className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">
                    {expiresDate.toLocaleDateString()}
                    {proposal.status === "OPEN" && (
                      <span className="ml-2 text-blue-600 dark:text-blue-400">({timeLeft})</span>
                    )}
                  </p>
                </div>
              </div>

              <p className="text-zinc-700 dark:text-zinc-300 whitespace-pre-wrap leading-relaxed">
                {proposal.description}
              </p>
            </div>
          </div>

          {/* Sidebar */}
          <aside className="space-y-6">
            {/* Vote results */}
            <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-50">Vote Results</h2>
                <span className="text-xs font-semibold text-zinc-400">{formatVotes(total)} votes</span>
              </div>

              <VoteTallyBar
                votes={proposal.votes}
                votingPower={proposal.votingPower}
                threshold={proposal.threshold}
                showCounts
              />

              <div className="mt-6 space-y-2.5 pt-4 border-t border-zinc-100 dark:border-zinc-800">
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500">Pass threshold</span>
                  <span className="font-semibold text-zinc-900 dark:text-zinc-100">
                    {((proposal.threshold ?? 0.667) * 100).toFixed(1)}% For
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500">Current For</span>
                  <span className={`font-semibold ${forPercent >= (proposal.threshold ?? 0.667) * 100 ? "text-emerald-500" : "text-zinc-900 dark:text-zinc-100"}`}>
                    {forPercent.toFixed(1)}%
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500">Quorum</span>
                  <span className={`font-semibold ${quorumReached ? "text-emerald-500" : "text-amber-500"}`}>
                    {quorumReached ? "Reached" : "Not reached"}
                  </span>
                </div>
              </div>
            </div>

            {/* Voting power */}
            {proposal.votingPower && (
              <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
                <h2 className="text-sm font-bold text-zinc-900 dark:text-zinc-50 mb-4">Voting Power</h2>
                <div className="space-y-2.5 text-sm">
                  <div className="flex justify-between">
                    <span className="text-zinc-500">Total eligible</span>
                    <span className="font-semibold text-zinc-900 dark:text-zinc-100">
                      {formatVotes(proposal.votingPower)} XLM
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-zinc-500">Your stake</span>
                    <span className="font-semibold text-zinc-900 dark:text-zinc-100">100,000 XLM</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-zinc-500">Your weight</span>
                    <span className="font-semibold text-blue-600 dark:text-blue-400">
                      {((100000 / proposal.votingPower) * 100).toFixed(2)}%
                    </span>
                  </div>
                </div>
                <p className="mt-4 text-[11px] text-zinc-400 leading-relaxed">
                  Vote weight is proportional to your on-chain staked XLM at proposal creation.
                </p>
              </div>
            )}

            {/* Cast vote */}
            {proposal.status === "OPEN" && (
              <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
                <h2 className="text-lg font-bold mb-2 text-zinc-900 dark:text-zinc-50">Cast Your Vote</h2>
                <p className="text-xs text-zinc-400 mb-6">
                  Signing with Freighter wallet confirms your vote on-chain.
                </p>

                {voteError && (
                  <div className="mb-4 p-3 rounded-xl bg-rose-50 dark:bg-rose-950/20 border border-rose-100 dark:border-rose-900/30 text-rose-600 dark:text-rose-400 text-sm font-medium">
                    {voteError}
                  </div>
                )}

                {hasVoted && (
                  <div className="mb-4 p-3 rounded-xl bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-100 dark:border-emerald-900/30 text-emerald-600 dark:text-emerald-400 text-sm font-medium">
                    You voted <span className="font-bold">{userVote}</span> on this proposal.
                  </div>
                )}

                <div className="space-y-3">
                  {(["For", "Against", "Abstain"] as const).map((type) => (
                    <VoteButton
                      key={type}
                      type={type}
                      onClick={() => vote(id, type)}
                      isLoading={isVoting}
                      disabled={hasVoted}
                      userVote={userVote}
                    />
                  ))}
                </div>
              </div>
            )}

            {/* Execution status */}
            {proposal.status !== "OPEN" && (
              <div className="bg-white dark:bg-zinc-900 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 shadow-sm">
                <h2 className="text-sm font-bold text-zinc-900 dark:text-zinc-50 mb-3">Execution Status</h2>
                <div className={`flex items-center gap-2 px-3 py-2 rounded-xl text-sm font-semibold ${STATUS_BADGE[proposal.status]}`}>
                  <span>{STATUS_ICON[proposal.status]}</span>
                  <span>{proposal.status}</span>
                </div>
                {proposal.status === "EXECUTED" && (
                  <p className="mt-3 text-xs text-zinc-400">
                    This proposal was passed and executed on-chain.
                  </p>
                )}
              </div>
            )}
          </aside>
        </div>
      </div>
    </div>
  );
}
