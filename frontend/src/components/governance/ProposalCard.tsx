"use client";

import Link from "next/link";
import { Proposal } from "@/hooks/useGovernance";
import { useEffect, useState } from "react";

interface ProposalCardProps {
  proposal: Proposal;
}

export function ProposalCard({ proposal }: ProposalCardProps) {
  const [timeLeft, setTimeLeft] = useState<string>("");

  useEffect(() => {
    if (proposal.status !== "OPEN") return;

    const updateTimer = () => {
      const now = Date.now();
      const diff = proposal.expiresAt - now;

      if (diff <= 0) {
        setTimeLeft("Expired");
        return;
      }

      const days = Math.floor(diff / (1000 * 60 * 60 * 24));
      const hours = Math.floor(
        (diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
      );
      const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

      if (days > 0) {
        setTimeLeft(`${days}d ${hours}h remaining`);
      } else {
        setTimeLeft(`${hours}h ${minutes}m remaining`);
      }
    };

    updateTimer();
    const interval = setInterval(updateTimer, 60000);
    return () => clearInterval(interval);
  }, [proposal.expiresAt, proposal.status]);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "OPEN":
        return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
      case "PASSED":
        return "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400";
      case "REJECTED":
        return "bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-400";
      case "EXECUTED":
        return "bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-400";
      default:
        return "bg-zinc-100 text-zinc-700";
    }
  };

  return (
    <Link
      href={`/governance/${proposal.id}`}
      className="block group p-6 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 hover:shadow-xl transition-all duration-300"
    >
      <div className="flex justify-between items-start mb-4">
        <span
          className={`px-3 py-1 rounded-full text-xs font-bold tracking-tight ${getStatusBadge(proposal.status)}`}
        >
          {proposal.status}
        </span>
        <span className="text-xs font-medium text-zinc-400 dark:text-zinc-500">
          Created: {new Date(proposal.createdAt).toLocaleDateString()}
        </span>
      </div>

      <h3 className="text-xl font-bold mb-2 text-zinc-900 dark:text-zinc-100 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
        {proposal.title}
      </h3>

      <p className="text-zinc-600 dark:text-zinc-400 line-clamp-2 text-sm mb-6 leading-relaxed">
        {proposal.description}
      </p>

      <div className="flex justify-between items-center pt-4 border-t border-zinc-100 dark:border-zinc-800">
        <div className="flex flex-col">
          <span className="text-[10px] uppercase tracking-wider text-zinc-400 font-bold mb-0.5">
            Type
          </span>
          <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {proposal.type}
          </span>
        </div>

        {proposal.status === "OPEN" && (
          <div className="text-right">
            <span className="text-[10px] uppercase tracking-wider text-zinc-400 font-bold mb-0.5 block">
              Time Left
            </span>
            <span className="text-sm font-semibold text-blue-600 dark:text-blue-400">
              {timeLeft}
            </span>
          </div>
        )}
      </div>
    </Link>
  );
}
